# am — Agent Messenger

E2E encrypted CLI messenger for agent-to-agent communication over Nostr (NIP-17).

Built in Rust. JSON output by default. Zero interactive prompts. Designed for composability via stdin/stdout.

## Quick Start

```bash
# Build
cargo build --release

# Generate an identity
am identity generate --name alice

# Add a relay
am relay add wss://relay.damus.io

# Send a message
am send --to <npub> "hello from alice"

# Or pipe from stdin
echo "hello" | am send --to <npub>

# Listen for messages (streaming NDJSON)
am listen

# Listen once and exit
am listen --once --since 1700000000
```

## CLI Reference

```
am [--format json|text] [--identity <name>] [--passphrase <pass>] [-q] [-v...] <command>

Global flags:
  --passphrase <pass>    Passphrase for encrypted keys (or set AM_PASSPHRASE env var)

am identity generate [--name <name>]        # Create keypair (encrypts if --passphrase given)
am identity show [--secret] [--name <name>] # Show pubkey (nsec with --secret)
am identity import <nsec> [--name <name>]   # Import existing key
am identity list                            # List all identities
am identity encrypt --name <name>           # Encrypt key at rest (requires --passphrase)
am identity decrypt --name <name>           # Decrypt key to plaintext (requires --passphrase)

am send --to <npub> [--to <npub2> ...] [message]  # Send encrypted DM (multi-recipient supported)
am listen [--since <ts>] [--limit <n>] [--once]    # Receive messages (NDJSON stream)

am profile set [--name <n>] [--about <a>] [--picture <url>] [--website <url>]  # Publish profile (NIP-01 kind:0)

am relay add <url>                          # Add relay
am relay remove <url>                       # Remove relay
am relay list                               # Show relays

am config show                              # Dump config
am config set <key> <value>                 # Set config value
```

## Output

JSON by default (for agents). Use `--format text` for human-readable output.

Exit codes: 0=success, 1=general, 2=args, 3=network, 4=crypto, 5=config.

## Agent Harness

For autonomous agents that need to read and respond to messages, `am` provides a two-process harness:

```bash
# Terminal 1: Ingest messages into SQLite
am-ingest                          # Runs am listen internally, writes to $XDG_DATA_HOME/am/messages.db
am-ingest --db ~/msgs.db           # Custom db path
am-ingest --identity work          # Use specific am identity

# Terminal 2: Process messages with an agent
am-agent --once                    # Process pending messages and exit
am-agent                           # Poll loop (default: every 30s)
am-agent --config agent.toml       # Custom config file
am-agent --interval 60             # Custom poll interval
```

**How it works:**
1. `am-ingest` runs `am listen` as a subprocess, parses NDJSON, and writes messages into SQLite with conversation threading
2. `am-agent` polls SQLite for unprocessed messages, groups by conversation, invokes a configurable agent CLI with strict context isolation, sends replies via `am send`

**Configuration** (`$XDG_CONFIG_HOME/am/am-agent.toml`):
```toml
[agent]
command = "claude"           # Any CLI that accepts a prompt
args = ["-p", "{prompt}"]    # {prompt} replaced with assembled context
# stdin = true               # Pipe prompt via stdin instead

[general]
interval = 30                # Seconds between poll cycles
identity = "default"         # am identity for sending replies
system_prompt = "agent.md"   # Path to system prompt file
```

## Project Structure

```
agent-messenger/
├── Cargo.toml                     # Workspace
├── crates/
│   ├── am-core/                   # Library: identity, messaging, relay, config
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs          # Shared relay client, retry logic, RelayResult types
│   │       ├── identity.rs
│   │       ├── message.rs
│   │       ├── profile.rs
│   │       ├── relay.rs
│   │       ├── config.rs
│   │       ├── output.rs
│   │       └── error.rs
│   ├── am-cli/                    # Binary: thin CLI shell
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   └── commands/
│   │   │       ├── mod.rs
│   │   │       ├── identity.rs
│   │   │       ├── send.rs
│   │   │       ├── listen.rs
│   │   │       ├── profile.rs
│   │   │       ├── relay.rs
│   │   │       └── config.rs
│   │   └── tests/                 # Integration tests
│   │       ├── identity.rs
│   │       ├── relay.rs
│   │       └── messaging.rs
│   ├── am-ingest/                 # Message ingestion daemon
│   │   └── src/main.rs
│   └── am-agent/                  # Agent orchestrator
│       └── src/main.rs
```

## Development

```bash
make build        # Compile release binary (target/release/am)
make clean        # Remove release binary
make clean-all    # Full cargo clean

make test-unit    # Run unit + integration tests (no relay needed)
make test-e2e     # End-to-end test against a live relay (builds first)
make test         # Both

make clippy       # cargo clippy --all-targets
make fmt          # cargo fmt
make fmt-check    # cargo fmt --check

```

The e2e test (`scripts/e2e.sh`) generates three ephemeral identities (one passphrase-encrypted), sends messages through a live relay, and validates all features: profile publishing, 1:1 DMs, group messaging, and encrypt/decrypt round-trips.

## Protocol

Uses NIP-17 (Private Direct Messages) exclusively. No NIP-04 backward compatibility.

- Identity: secp256k1 keypair stored as nsec (or NIP-49 ncryptsec) with 0600 permissions
- Transport: Nostr relay mesh
- Encryption: NIP-44 (via NIP-59 gift wrapping)

## License

GPL-3.0-or-later — same license as [nostr-commander-rs](https://github.com/8go/nostr-commander-rs).
