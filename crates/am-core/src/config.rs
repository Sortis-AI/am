use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{AmError, AmResult};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default_identity: Option<String>,

    #[serde(default)]
    pub relays: Vec<String>,

    #[serde(default)]
    pub format: Option<String>,
}

pub fn data_dir() -> AmResult<PathBuf> {
    let base = dirs::data_dir()
        .ok_or_else(|| AmError::Config("cannot determine data directory".into()))?;
    Ok(base.join("am"))
}

pub fn config_dir() -> AmResult<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| AmError::Config("cannot determine config directory".into()))?;
    Ok(base.join("am"))
}

pub fn config_path() -> AmResult<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn identity_dir() -> AmResult<PathBuf> {
    Ok(data_dir()?.join("identities"))
}

pub fn ensure_dirs() -> AmResult<()> {
    fs::create_dir_all(config_dir()?)?;
    fs::create_dir_all(identity_dir()?)?;
    Ok(())
}

pub fn load_config() -> AmResult<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents = fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

pub fn save_config(config: &Config) -> AmResult<()> {
    ensure_dirs()?;
    let contents = toml::to_string_pretty(config)?;
    fs::write(config_path()?, contents)?;
    Ok(())
}
