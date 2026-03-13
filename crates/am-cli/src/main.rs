use clap::Parser;
use tracing_subscriber::EnvFilter;

mod commands;

use commands::{Cli, run};

#[tokio::main]
async fn main() {
    let no_dna = std::env::var("NO_DNA").is_ok_and(|v| !v.is_empty());

    let mut filter = EnvFilter::from_default_env();
    if no_dna && std::env::var("RUST_LOG").is_err() {
        filter = EnvFilter::new("debug");
    }

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let is_json = matches!(cli.format, commands::OutputFormat::Json) || no_dna;

    if let Err(e) = run(cli).await {
        let code = e.exit_code();
        if is_json {
            let _ = am_core::output::eprint_json(&serde_json::json!({ "error": e.to_string() }));
        } else {
            eprintln!("error: {e}");
        }
        std::process::exit(code);
    }
}
