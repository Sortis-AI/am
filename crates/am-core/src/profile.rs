use nostr_sdk::prelude::*;
use serde::Serialize;

use crate::client::{self, RelayResult};
use crate::config::load_config;
use crate::error::{AmError, AmResult};
use crate::identity::load_keys;

#[derive(Debug, Serialize)]
pub struct ProfileInfo {
    pub npub: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    pub event_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub relays: Vec<RelayResult>,
}

pub async fn set(
    identity: Option<&str>,
    name: Option<&str>,
    about: Option<&str>,
    picture: Option<&str>,
    website: Option<&str>,
    passphrase: Option<&str>,
    verbosity: u8,
) -> AmResult<ProfileInfo> {
    if name.is_none() && about.is_none() && picture.is_none() && website.is_none() {
        return Err(AmError::Args(
            "at least one of --name, --about, --picture, or --website is required".into(),
        ));
    }

    let keys = load_keys(identity.unwrap_or("default"), passphrase)?;
    let config = load_config()?;

    if config.relays.is_empty() {
        return Err(AmError::Network(
            "no relays configured; run `am relay add <url>`".into(),
        ));
    }

    let mut metadata = Metadata::new();
    if let Some(n) = name {
        metadata.name = Some(n.to_string());
    }
    if let Some(a) = about {
        metadata.about = Some(a.to_string());
    }
    if let Some(p) = picture {
        metadata.picture = Some(p.to_string());
    }
    if let Some(w) = website {
        metadata.website = Some(w.to_string());
    }

    let npub = keys.public_key().to_bech32().unwrap_or_default();
    let client = client::connect(keys, &config.relays).await?;

    // Build and sign the metadata event, then send with retry
    let event = client
        .sign_event_builder(EventBuilder::metadata(&metadata))
        .await
        .map_err(|e| AmError::Network(e.to_string()))?;

    let event_id = event.id.to_bech32().unwrap_or_default();

    let (relay_results, _succeeded) =
        client::send_with_retry(&client, &event, &config.relays, 3, verbosity).await;

    client.disconnect().await;

    Ok(ProfileInfo {
        npub,
        name: name.map(String::from),
        about: about.map(String::from),
        picture: picture.map(String::from),
        website: website.map(String::from),
        event_id,
        relays: relay_results,
    })
}
