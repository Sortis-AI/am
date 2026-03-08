# Map

## Actors

- **Agent A** — sender, holds keypair, sends NIP-17 gift-wrapped DMs
- **Agent B** — receiver, holds keypair, subscribes to Kind:1059 (GiftWrap) events
- **Nostr Relays** — dumb pipes, store/forward encrypted events

## Data Flow

### 1:1 Send
```
Agent A                    Relays                    Agent B
  |                          |                          |
  |-- send_private_msg() --> |                          |
  |   (NIP-17 gift wrap)     |                          |
  |                          | <-- subscribe(GiftWrap)--|
  |                          |-- event notification --> |
  |                          |   unwrap_gift_wrap()     |
  |                          |                          |
```

### Group Send (multi-recipient)
```
Agent A                    Relays              Agent B, Agent C
  |                          |                      |      |
  |-- send_private_msg(B) -> |                      |      |
  |-- send_private_msg(C) -> |                      |      |
  |   (one gift wrap each)   |                      |      |
  |                          | -- event(B) -------> |      |
  |                          | -- event(C) --------------> |
```

## Port Map

No network ports. CLI only. Communication is relay-mediated via WebSocket.

## File Layout

| Path | Purpose |
|------|---------|
| `$XDG_CONFIG_HOME/am/config.toml` | Relay list, default identity, format pref |
| `$XDG_DATA_HOME/am/identities/<name>.nsec` | Secret keys — `nsec1...` (plaintext) or `ncryptsec1...` (NIP-49 encrypted), 0600 perms |

## File Layout (project root)

| Path | Purpose |
|------|---------|
| `Makefile` | Build, test, lint targets |
| `scripts/e2e.sh` | End-to-end integration test (requires live relay + `jq`) |
| `crates/am-core/` | Library crate |
| `crates/am-cli/` | Binary crate (`am`) |
| `crates/am-cli/tests/` | Integration tests (no relay required) |

## Module Dependency

```
am-cli
  └── am-core
        ├── client   (shared relay connection, send_with_retry, RelayResult)
        ├── identity (nostr-sdk Keys, NIP-49 EncryptedSecretKey)
        ├── message  (nostr-sdk Client, concurrent multi-recipient send, uses client)
        ├── profile  (nostr-sdk Metadata, kind:0, uses client)
        ├── relay    (config read/write)
        ├── config   (dirs, toml, fs)
        ├── output   (serde_json)
        └── error    (thiserror)
```
