use clap::Subcommand;

use am_core::error::AmResult;
use am_core::output::Format;
use am_core::{identity, output};

#[derive(Debug, Subcommand)]
pub enum IdentityCmd {
    /// Generate a new keypair
    Generate {
        /// Name for this identity
        #[arg(long)]
        name: Option<String>,
    },

    /// Show identity public key (or secret with --secret)
    Show {
        /// Also display the secret key (nsec)
        #[arg(long)]
        secret: bool,

        /// Identity name
        #[arg(long)]
        name: Option<String>,
    },

    /// Import an existing secret key
    Import {
        /// The nsec bech32-encoded secret key
        nsec: String,

        /// Name for this identity
        #[arg(long)]
        name: Option<String>,
    },

    /// List all identities
    List,

    /// Encrypt an existing identity with a passphrase (NIP-49)
    Encrypt {
        /// Identity name
        #[arg(long, default_value = "default")]
        name: String,
    },

    /// Decrypt an encrypted identity back to plaintext
    Decrypt {
        /// Identity name
        #[arg(long, default_value = "default")]
        name: String,
    },
}

pub async fn run(cmd: IdentityCmd, passphrase: Option<&str>, format: Format) -> AmResult<()> {
    match cmd {
        IdentityCmd::Generate { name } => {
            let info = identity::generate(name.as_deref(), passphrase)?;
            match format {
                Format::Json => output::print_json(&info)?,
                Format::Text => {
                    println!("Identity '{}' created", info.name);
                    println!("npub: {}", info.npub);
                    if info.encrypted {
                        println!("(encrypted with passphrase)");
                    }
                }
            }
        }
        IdentityCmd::Show { secret, name } => {
            let info = identity::show(name.as_deref(), secret, passphrase)?;
            match format {
                Format::Json => output::print_json(&info)?,
                Format::Text => {
                    println!("Name: {}", info.name);
                    println!("npub: {}", info.npub);
                    if let Some(nsec) = &info.nsec {
                        println!("nsec: {nsec}");
                    }
                    if info.encrypted {
                        println!("(encrypted)");
                    }
                }
            }
        }
        IdentityCmd::Import { nsec, name } => {
            let info = identity::import(&nsec, name.as_deref(), passphrase)?;
            match format {
                Format::Json => output::print_json(&info)?,
                Format::Text => {
                    println!("Identity '{}' imported", info.name);
                    println!("npub: {}", info.npub);
                    if info.encrypted {
                        println!("(encrypted with passphrase)");
                    }
                }
            }
        }
        IdentityCmd::List => {
            let identities = identity::list()?;
            match format {
                Format::Json => output::print_json(&identities)?,
                Format::Text => {
                    for id in &identities {
                        let suffix = if id.encrypted { " (encrypted)" } else { "" };
                        println!("{}: {}{suffix}", id.name, id.npub);
                    }
                }
            }
        }
        IdentityCmd::Encrypt { name } => {
            let pass = passphrase.ok_or_else(|| {
                am_core::error::AmError::Args("--passphrase is required for encryption".into())
            })?;
            let info = identity::encrypt_existing(&name, pass)?;
            match format {
                Format::Json => output::print_json(&info)?,
                Format::Text => println!("Identity '{}' encrypted", info.name),
            }
        }
        IdentityCmd::Decrypt { name } => {
            let pass = passphrase.ok_or_else(|| {
                am_core::error::AmError::Args("--passphrase is required for decryption".into())
            })?;
            let info = identity::decrypt_existing(&name, pass)?;
            match format {
                Format::Json => output::print_json(&info)?,
                Format::Text => println!("Identity '{}' decrypted", info.name),
            }
        }
    }
    Ok(())
}
