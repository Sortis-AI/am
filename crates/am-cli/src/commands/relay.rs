use clap::Subcommand;

use am_core::error::AmResult;
use am_core::output::Format;
use am_core::{output, relay};

#[derive(Debug, Subcommand)]
pub enum RelayCmd {
    /// Add a relay
    Add {
        /// Relay websocket URL (wss://...)
        url: String,
    },

    /// Remove a relay
    Remove {
        /// Relay websocket URL
        url: String,
    },

    /// List configured relays
    List,
}

pub async fn run(cmd: RelayCmd, format: Format) -> AmResult<()> {
    match cmd {
        RelayCmd::Add { url } => {
            relay::add(&url)?;
            match format {
                Format::Json => output::print_json(&serde_json::json!({ "added": url }))?,
                Format::Text => println!("Added relay: {url}"),
            }
        }
        RelayCmd::Remove { url } => {
            relay::remove(&url)?;
            match format {
                Format::Json => output::print_json(&serde_json::json!({ "removed": url }))?,
                Format::Text => println!("Removed relay: {url}"),
            }
        }
        RelayCmd::List => {
            let relays = relay::list()?;
            match format {
                Format::Json => output::print_json(&relays)?,
                Format::Text => {
                    for r in &relays {
                        println!("{}", r.url);
                    }
                }
            }
        }
    }
    Ok(())
}
