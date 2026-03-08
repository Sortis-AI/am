use serde::Serialize;

use crate::config::{load_config, save_config};
use crate::error::{AmError, AmResult};

#[derive(Debug, Serialize)]
pub struct RelayInfo {
    pub url: String,
}

pub fn add(url: &str) -> AmResult<()> {
    let mut config = load_config()?;
    if config.relays.iter().any(|r| r == url) {
        return Err(AmError::Config(format!("relay '{url}' already exists")));
    }
    config.relays.push(url.to_string());
    save_config(&config)?;
    Ok(())
}

pub fn remove(url: &str) -> AmResult<()> {
    let mut config = load_config()?;
    let before = config.relays.len();
    config.relays.retain(|r| r != url);
    if config.relays.len() == before {
        return Err(AmError::Config(format!("relay '{url}' not found")));
    }
    save_config(&config)?;
    Ok(())
}

pub fn list() -> AmResult<Vec<RelayInfo>> {
    let config = load_config()?;
    Ok(config
        .relays
        .into_iter()
        .map(|url| RelayInfo { url })
        .collect())
}
