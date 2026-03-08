use clap::Parser;
use tracing_subscriber::EnvFilter;

mod commands;

use commands::{Cli, run};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        let code = e.exit_code();
        let format = am_core::output::Format::Json; // TODO: propagate from cli
        match format {
            am_core::output::Format::Json => {
                let _ = am_core::output::print_json(&serde_json::json!({ "error": e.to_string() }));
            }
            am_core::output::Format::Text => {
                eprintln!("error: {e}");
            }
        }
        std::process::exit(code);
    }
}
