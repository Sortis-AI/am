use clap::Subcommand;

use am_core::error::AmResult;
use am_core::output::Format;
use am_core::{output, profile};

#[derive(Debug, Subcommand)]
pub enum ProfileCmd {
    /// Publish profile metadata (NIP-01 kind:0)
    Set {
        /// Display name
        #[arg(long)]
        name: Option<String>,

        /// About / description
        #[arg(long)]
        about: Option<String>,

        /// Picture URL
        #[arg(long)]
        picture: Option<String>,

        /// Website URL
        #[arg(long)]
        website: Option<String>,
    },
}

pub async fn run(
    cmd: ProfileCmd,
    identity: Option<&str>,
    passphrase: Option<&str>,
    format: Format,
    verbosity: u8,
) -> AmResult<()> {
    match cmd {
        ProfileCmd::Set {
            name,
            about,
            picture,
            website,
        } => {
            let info = profile::set(
                identity,
                name.as_deref(),
                about.as_deref(),
                picture.as_deref(),
                website.as_deref(),
                passphrase,
                verbosity,
            )
            .await?;
            match format {
                Format::Json => output::print_json(&info)?,
                Format::Text => {
                    println!("Profile published for {}", info.npub);
                    if let Some(n) = &info.name {
                        println!("  name: {n}");
                    }
                    if let Some(a) = &info.about {
                        println!("  about: {a}");
                    }
                    if let Some(p) = &info.picture {
                        println!("  picture: {p}");
                    }
                    if let Some(w) = &info.website {
                        println!("  website: {w}");
                    }
                }
            }
        }
    }
    Ok(())
}
