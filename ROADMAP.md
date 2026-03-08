# Roadmap

## Vision

The internet is about to fill up with agents. They will negotiate, coordinate, delegate, and transact — and they need a way to talk to each other that doesn't depend on a single company's API, a human clicking "approve" in a dashboard, or a server that can go down and take the whole network with it.

`am` is the messaging primitive for that world. A single binary. A keypair is your identity. A mesh of relays is your transport. NIP-17 gift wrapping means not even the relays know who's talking to whom. No accounts, no provisioning, no vendor lock-in. Generate a key, add a relay, send a message.

The bet is that agent communication will follow the same arc as human communication: it starts centralized (Slack webhooks, API callbacks, managed queues) and then someone builds the thing that's simple enough and decentralized enough that it becomes the default. That's what we're building.

The end state is a world where any agent — running on any infrastructure, built by any team — can reach any other agent with nothing more than its public key. No intermediary has the power to deplatform, surveil, or rate-limit the conversation. The protocol is the platform.

---

## v0.1.0 — Foundation (current)

Ship date: now.

The minimum viable messenger. Two agents can find each other by npub, exchange encrypted messages through Nostr relays, and parse the results programmatically.

- [x] Identity management (generate, import, show, list)
- [x] NIP-17 encrypted send (message arg or stdin pipe)
- [x] Listen for messages (NDJSON streaming + `--once` batch)
- [x] Relay management (add, remove, list)
- [x] Config management (show, set)
- [x] JSON output by default, `--format text` for humans
- [x] Typed exit codes (0-5)
- [x] Library + binary split (`am-core` / `am-cli`)
- [x] XDG-compliant data and config storage
- [x] Integration test suite
- [x] CI (build, test, clippy, fmt) + release workflow

---

## v0.2.0 — Hardening

Make it safe to run in less controlled environments. Make it useful for more than 1:1 chat.

### Key encryption at rest
Plaintext nsec files are fine when your agent runs in an isolated VM with disk encryption. They're not fine on a shared workstation. v0.2 adds optional passphrase-protected key storage (NIP-49), unlockable via `--passphrase` flag or `AM_PASSPHRASE` env var. Keys stored as `ncryptsec1...` instead of `nsec1...`. No interactive prompts — the automation constraint holds.

### Group messaging
NIP-17 supports multi-recipient gift wrapping. Expose it: `am send --to <npub1> --to <npub2> ...`. Each recipient gets their own gift-wrapped copy. No group management — just multi-recipient encryption. Agents that need rooms can build them on top.

### Profile publishing
`am profile set --name "Alice Bot" --about "Research agent" --picture <url> --website <url>` — publish a NIP-01 kind:0 metadata event for the identity. Allows human Nostr clients to display a readable name, avatar, and website next to an agent's npub instead of a raw key. Write-only: `am` publishes the profile, it does not fetch or display others'.

---

## v0.3.0 — Resilience

Make it hard to lose messages. Make it observable.

### Inbox relay publication
`am relay publish-inbox` — publish a kind:10050 event listing the relays where the identity receives messages. This lets senders discover where to deliver without out-of-band coordination. Prerequisite for real-world multi-agent deployments.

### File transfer
`am send --file <path> --to <npub>` — base64-encode small payloads into the message content with a structured envelope. Not a file hosting protocol — just a way to pass configs, credentials, and small artifacts between agents without leaving the encrypted channel.

### Contact aliases
`am contact add alice <npub1...>` — local alias book so agents (and humans) can use names instead of raw pubkeys. Stored in config. No NIP-02 contact list publication — that's a privacy leak for agents.

### Message persistence
Local SQLite store for sent and received messages. `am history` command. Deduplication for messages received across multiple relays. Agents that restart mid-conversation can catch up.

### Relay health and failover
`am relay list` shows connection status, latency, last-seen timestamps. Auto-reconnect on transient failures. Configurable relay sets per identity (some agents may need different relay strategies than others).

### Delivery confirmation
Best-effort delivery receipts. When a message is sent, track which relays accepted it. Optionally wait for at least N relay confirmations before returning success. Exit code 3 if no relay confirms.

### Structured logging
`RUST_LOG` already works via tracing. v0.3 adds `--log-file` and `--log-format json` for agents that need to ship logs to observability systems. Trace IDs on messages for distributed tracing across agent conversations.

---

## v0.4.0 — Composability

Make `am` a building block, not just a CLI.

### SDK and bindings
Publish `am-core` to crates.io. Add Python bindings (PyO3) and a C FFI layer. An agent written in any language can link against the library directly instead of shelling out to the CLI.

### Daemon mode
`am daemon` — long-running process that maintains relay connections and exposes a local Unix socket or HTTP API. Eliminates the connect/disconnect overhead of one-shot CLI invocations. Agents that send hundreds of messages per minute need this.

### Webhooks and triggers
`am listen --exec <command>` — pipe each incoming message to a command's stdin. `am listen --webhook <url>` — POST each message to a local HTTP endpoint. Turn `am` into an event-driven bridge between Nostr and any system.

### Message schemas
Optional structured message envelopes: `{ "type": "task.assign", "payload": { ... } }`. Schema validation via JSON Schema. Agents that speak a shared protocol can discover each other's capabilities and negotiate formats.

---

## v0.5.0 — Trust

Make it possible to verify who you're talking to.

### Identity verification
Out-of-band identity verification flows. An agent can sign a challenge to prove it controls a given npub. Web-of-trust signals: "I trust this npub because npub-X vouched for it." No central authority — trust is a graph, not a hierarchy.

### Rate limiting and filtering
`am listen --filter-from <npub>` — only accept messages from known senders. Configurable per-sender rate limits. Spam is an unsolved problem on Nostr for humans; for agents it's a DoS vector. Defense starts here.

### Audit log
Append-only local log of all cryptographic operations: key generation, message encryption, gift wrap creation. For environments where agents handle sensitive data and need to prove what happened.

---

## Long-term / Not yet scoped

These are directions, not commitments. They depend on how the ecosystem evolves.

- **Relay operator tools** — `am-relay` crate for running a lightweight relay optimized for agent traffic (small events, high throughput, no media)
- **Agent discovery protocol** — publish capabilities and routing information so agents can find each other by function rather than by pubkey
- **NIP-101 Alias Key Exchange** — support for key aliasing and geographic-coordinate-based key derivation (Hierarchical GeoKeys), enabling agents to manage multiple key aliases and establish hierarchical identity relationships
- **Multi-chain identity** — bridge Nostr keys to other identity systems (DID, KERI) for agents that operate across protocol boundaries
- **Offline-first sync** — CRDTs or append-only logs for agents that need to sync state, not just exchange messages
- **Hardware key support** — PKCS#11 or similar for agents running in HSM-backed environments

---

## Non-goals (permanent)

Some things `am` will never be:

- **A GUI application.** There are good Nostr clients for humans. This is for machines.
- **A relay server.** Relays are infrastructure. `am` is a client.
- **A NIP-04 implementation.** NIP-04 is broken. We don't ship broken crypto.
- **A general-purpose Nostr client.** We don't post notes, follow people, or manage social graphs. One job, done well.
- **A framework.** `am` is a tool. Frameworks tell you how to build your agent. `am` doesn't care how your agent works — it just delivers the message.
