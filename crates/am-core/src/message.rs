use std::io::{self, Read};

use nostr_sdk::prelude::*;
use serde::Serialize;

use crate::client::{self, RelayResult};
use crate::config::load_config;
use crate::error::{AmError, AmResult};
use crate::identity::load_keys;

#[derive(Debug, Serialize)]
pub struct SentMessage {
    pub to: Vec<String>,
    pub content: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failed: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub relays: Vec<RelayResult>,
}

#[derive(Debug, Serialize)]
pub struct ReceivedMessage {
    pub from: String,
    pub content: String,
    pub timestamp: u64,
}

pub async fn send(
    identity: Option<&str>,
    to_npubs: &[String],
    message: Option<&str>,
    passphrase: Option<&str>,
    verbosity: u8,
) -> AmResult<SentMessage> {
    let keys = load_keys(identity.unwrap_or("default"), passphrase)?;
    let config = load_config()?;

    if config.relays.is_empty() {
        return Err(AmError::Network(
            "no relays configured; run `am relay add <url>`".into(),
        ));
    }

    if to_npubs.is_empty() {
        return Err(AmError::Args(
            "at least one --to recipient is required".into(),
        ));
    }

    let content = match message {
        Some(m) => m.to_string(),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf.trim_end().to_string()
        }
    };

    if content.is_empty() {
        return Err(AmError::Args("message is empty".into()));
    }

    let mut recipients = Vec::new();
    for npub in to_npubs {
        let pk = PublicKey::from_bech32(npub).map_err(|e| AmError::Crypto(e.to_string()))?;
        recipients.push((npub.clone(), pk));
    }

    let client = client::connect(keys, &config.relays).await?;

    // Build gift-wrap events (sequential — each needs unique encryption)
    let signer = client
        .signer()
        .await
        .map_err(|e| AmError::Crypto(e.to_string()))?;

    let our_pk = signer
        .get_public_key()
        .await
        .map_err(|e| AmError::Crypto(e.to_string()))?;

    let mut wrapped_events = Vec::new();
    for (npub, pk) in &recipients {
        // NIP-17: the p-tag set defines the chat room. Clients like 0xchat
        // match rooms by p-tags, so we must include ALL participants —
        // every other recipient plus the sender — so the room identity
        // is consistent across all participants.
        //
        // We build the rumor manually instead of using EventBuilder::private_msg
        // because the builder strips p-tags matching the author by default.
        // Group chats require the sender's own p-tag for room identity.
        let extra_tags: Vec<Tag> = if recipients.len() > 1 {
            let mut tags: Vec<Tag> = recipients
                .iter()
                .filter(|(_, other_pk)| other_pk != pk)
                .map(|(_, other_pk)| Tag::public_key(*other_pk))
                .collect();
            tags.push(Tag::public_key(our_pk));
            tags
        } else {
            vec![]
        };
        let rumor = EventBuilder::private_msg_rumor(*pk, &content)
            .tags(extra_tags)
            .allow_self_tagging()
            .build(our_pk);
        let event = EventBuilder::gift_wrap(&signer, pk, rumor, [])
            .await
            .map_err(|e| AmError::Crypto(e.to_string()))?;
        wrapped_events.push((npub.clone(), event));
    }

    // Send all events concurrently with retry
    let mut handles = Vec::new();
    for (npub, event) in wrapped_events {
        let client = client.clone();
        let relays = config.relays.clone();
        handles.push(tokio::spawn(async move {
            let (relay_results, succeeded) =
                client::send_with_retry(&client, &event, &relays, 3, verbosity).await;
            (npub, relay_results, !succeeded.is_empty())
        }));
    }

    let join_results = futures::future::join_all(handles).await;

    let mut sent = Vec::new();
    let mut failed = Vec::new();
    let mut all_relay_results = Vec::new();

    for result in join_results {
        match result {
            Ok((npub, relay_results, any_success)) => {
                if any_success {
                    sent.push(npub);
                } else {
                    failed.push(npub);
                }
                // Merge relay results (dedup by relay URL, keep worst status)
                for rr in relay_results {
                    if let Some(existing) = all_relay_results
                        .iter_mut()
                        .find(|r: &&mut RelayResult| r.relay == rr.relay)
                    {
                        // If any send to this relay failed, mark it failed
                        if matches!(rr.status, client::RelayStatus::Failed) {
                            existing.status = client::RelayStatus::Failed;
                            existing.error = rr.error;
                        }
                    } else {
                        all_relay_results.push(rr);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("send task panicked: {e}");
            }
        }
    }

    client.disconnect().await;

    if sent.is_empty() {
        return Err(AmError::Network("failed to send to all recipients".into()));
    }

    Ok(SentMessage {
        to: sent,
        content,
        failed,
        relays: all_relay_results,
    })
}

pub async fn listen(
    identity: Option<&str>,
    since: Option<u64>,
    limit: Option<usize>,
    once: bool,
    timeout_secs: u64,
    passphrase: Option<&str>,
    _verbosity: u8,
) -> AmResult<Vec<ReceivedMessage>> {
    let keys = load_keys(identity.unwrap_or("default"), passphrase)?;
    let config = load_config()?;

    if config.relays.is_empty() {
        return Err(AmError::Network(
            "no relays configured; run `am relay add <url>`".into(),
        ));
    }

    let client = client::connect(keys, &config.relays).await?;

    let filter = {
        let our_pk = client
            .public_key()
            .await
            .map_err(|e| AmError::Crypto(e.to_string()))?;
        let mut f = Filter::new().kind(Kind::GiftWrap).pubkey(our_pk);
        if let Some(ts) = since {
            f = f.since(Timestamp::from(ts));
        }
        if let Some(l) = limit {
            f = f.limit(l);
        }
        f
    };

    if once {
        let events = client
            .fetch_events(filter, std::time::Duration::from_secs(timeout_secs))
            .await
            .map_err(|e| AmError::Network(e.to_string()))?;

        let mut messages = Vec::new();
        for event in events.into_iter() {
            if let Ok(unwrapped) = client.unwrap_gift_wrap(&event).await {
                messages.push(ReceivedMessage {
                    from: unwrapped.sender.to_bech32().unwrap_or_default(),
                    content: unwrapped.rumor.content.clone(),
                    timestamp: unwrapped.rumor.created_at.as_secs(),
                });
            }
        }
        messages.sort_by_key(|m| m.timestamp);
        client.disconnect().await;
        Ok(messages)
    } else {
        client
            .subscribe(filter, None)
            .await
            .map_err(|e| AmError::Network(e.to_string()))?;

        client
            .handle_notifications(|notification| async {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind == Kind::GiftWrap {
                        if let Ok(unwrapped) = client.unwrap_gift_wrap(&event).await {
                            let msg = ReceivedMessage {
                                from: unwrapped.sender.to_bech32().unwrap_or_default(),
                                content: unwrapped.rumor.content.clone(),
                                timestamp: unwrapped.rumor.created_at.as_secs(),
                            };
                            if let Ok(json) = serde_json::to_string(&msg) {
                                println!("{json}");
                            }
                        }
                    }
                }
                Ok(false)
            })
            .await
            .map_err(|e| AmError::Network(e.to_string()))?;

        Ok(vec![])
    }
}
