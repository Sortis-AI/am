use clap::Args;

use am_core::error::AmResult;
use am_core::output::Format;
use am_core::{message, output};

#[derive(Debug, Args)]
pub struct SendArgs {
    /// Recipient npub (repeatable for group messages)
    #[arg(long, required = true)]
    pub to: Vec<String>,

    /// Message content (reads from stdin if omitted)
    pub message: Option<String>,
}

pub async fn run(
    args: SendArgs,
    identity: Option<&str>,
    passphrase: Option<&str>,
    format: Format,
    verbosity: u8,
) -> AmResult<()> {
    let result = message::send(
        identity,
        &args.to,
        args.message.as_deref(),
        passphrase,
        verbosity,
    )
    .await?;
    match format {
        Format::Json => output::print_json(&result)?,
        Format::Text => {
            let recipients = result.to.join(", ");
            println!("Sent message to {recipients}");
            if !result.failed.is_empty() {
                let failed = result.failed.join(", ");
                eprintln!("Failed to send to: {failed}");
            }
        }
    }
    Ok(())
}
