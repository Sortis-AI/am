use thiserror::Error;

#[derive(Debug, Error)]
pub enum AmError {
    #[error("general error: {0}")]
    General(String),

    #[error("invalid arguments: {0}")]
    Args(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("toml deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("toml serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("nostr client error: {0}")]
    Nostr(#[from] nostr_sdk::client::Error),

    #[error("nostr key error: {0}")]
    NostrKey(#[from] nostr_sdk::key::Error),
}

impl AmError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AmError::General(_) | AmError::Io(_) | AmError::Json(_) => 1,
            AmError::Args(_) => 2,
            AmError::Network(_) | AmError::Nostr(_) => 3,
            AmError::Crypto(_) | AmError::NostrKey(_) => 4,
            AmError::Config(_) | AmError::TomlDe(_) | AmError::TomlSer(_) => 5,
        }
    }
}

pub type AmResult<T> = Result<T, AmError>;
