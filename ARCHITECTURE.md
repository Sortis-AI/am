# Architecture

## Crate Layout

### am-core (library)

The core library. Other Rust projects can depend on this directly.

| Module | Purpose |
|--------|---------|
| `client.rs` | Shared relay client setup, connection, per-relay retry logic (`send_with_retry`), `RelayResult`/`RelayStatus` types |
| `identity.rs` | Keypair generation, import/export, bech32 encoding, NIP-49 encryption, file storage (0600 perms) |
| `message.rs` | NIP-17 send (single/multi-recipient, concurrent per-recipient) and receive via nostr-sdk Client |
| `profile.rs` | NIP-01 kind:0 metadata publishing (name, about, picture, website) with per-relay retry |
| `relay.rs` | Relay add/remove/list against TOML config |
| `config.rs` | XDG directory management, TOML config read/write |
| `output.rs` | JSON/text output formatting |
| `error.rs` | `AmError` enum with typed exit codes |

### am-cli (binary)

Thin CLI shell. Parses args via clap derive, delegates to am-core, formats output.

Binary name: `am`

## Data Storage

All data lives under XDG directories:

- **Config:** `$XDG_CONFIG_HOME/am/config.toml`
- **Identities:** `$XDG_DATA_HOME/am/identities/<name>.nsec` (plaintext `nsec1...` or encrypted `ncryptsec1...`)

Config schema (`Config` struct):
- `default_identity: Option<String>`
- `relays: Vec<String>`
- `format: Option<String>`

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `nostr-sdk` | 0.44 (nip49, nip59) | Nostr protocol, NIP-01/17/44/49/59 |
| `tokio` | 1 | Async runtime |
| `clap` | 4.5 (derive, env) | CLI parsing with env var support |
| `serde` + `serde_json` | 1 | Serialization |
| `toml` | 1 | Config format |
| `thiserror` | 2 | Error derives |
| `dirs` | 6 | XDG paths |
| `futures` | 0.3 | Async combinators (join_all for concurrent sends) |
| `tracing` + `tracing-subscriber` | 0.1/0.3 | Structured logging |

## Key Encryption (NIP-49)

Identity files can be stored encrypted using NIP-49 (`ncryptsec1...` prefix). Passphrase is provided via:
- `--passphrase <value>` global CLI flag
- `AM_PASSPHRASE` environment variable

Encryption uses `log_n = 16` (scrypt) and `KeySecurity::Medium`. Plaintext `nsec1...` files continue to work without a passphrase. The file prefix determines the format automatically.

## Error Model

`AmError` variants map to exit codes:
- 1: General / IO / JSON
- 2: Invalid arguments
- 3: Network / Nostr client
- 4: Crypto / key errors
- 5: Config / TOML

## Message Flow

### Send
1. Load identity keys from `$XDG_DATA_HOME/am/identities/<name>.nsec`
2. Load relay list from config
3. Connect nostr-sdk Client to relays (`client::connect`)
4. Build gift-wrap events per recipient (`EventBuilder::private_msg`)
5. Send all recipients concurrently via `tokio::spawn` + `client::send_with_retry` (3 attempts per relay)
6. Collect per-relay results (`RelayResult` with status, optional error/attempts based on verbosity)
7. Disconnect

### Listen (--once)
1. Load identity, connect to relays
2. `client.fetch_events()` with Kind::GiftWrap filter
3. `client.unwrap_gift_wrap()` each event
4. Output as JSON/text, disconnect

### Listen (streaming)
1. Load identity, connect to relays
2. `client.subscribe()` with Kind::GiftWrap filter
3. `client.handle_notifications()` — print NDJSON per message
