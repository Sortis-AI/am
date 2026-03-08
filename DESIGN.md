# Design

## Core Decisions

### NIP-17 only
No NIP-04 backward compatibility. Clean break for metadata privacy (NIP-59 gift wrapping).

### JSON output by default
Agents parse JSON. Humans get `--format text`. All output is structured.

### Library + binary split
`am-core` is a standalone library. `am-cli` is a thin shell. Other Rust projects embed `am-core` directly.

### No interactive prompts
Everything via flags and stdin. Agents can't type passwords.

### Key storage
Keys stored as nsec files with 0600 permissions by default. Optional NIP-49 passphrase encryption (`ncryptsec1...`) via `--passphrase` flag or `AM_PASSPHRASE` env var.

### TOML config
Human-editable when needed. Typed. Rust-native.

## Scope Boundaries (NOT building)

- No GUI, no web interface, no relay server
- No NIP-04 support (deprecated)
- No file transfer (v0.3)
- No contact lists (v0.3)
- No inbox relay publication (v0.3)

## Roadmap

### v0.1.0 (current)
- Identity management (generate, import, show, list)
- Relay management (add, remove, list)
- Send encrypted DMs (NIP-17)
- Listen for messages (streaming + batch)
- Config management
- JSON + text output formats

### v0.2.0
- Key encryption at rest (NIP-49 `ncryptsec1` storage)
- Group messaging (multi-recipient `--to`)
- Profile publishing (NIP-01 kind:0)

### v0.2.1
- Per-relay retry with configurable attempts (3 by default)
- Per-relay JSON output (`relays` array with status/error/attempts)
- Verbosity levels wired through (`-v` adds error details and attempt counts)
- Concurrent per-recipient sends (group messages send in parallel)
- Shared client module eliminates connection boilerplate

### v0.3.0
- Public posts (NIP-01 kind:1 short text notes)
- Inbox relay publication (kind:10050)
- File transfer
- Contact aliases
- Message persistence
- Relay health monitoring (builds on per-relay status from v0.2.1)
- Delivery confirmation
- Structured logging

## Distribution

Target platforms: Linux x86_64/aarch64, macOS x86_64/aarch64.
CI: GitHub Actions (build + test + clippy + fmt).
Release: cross-compiled static binaries.
