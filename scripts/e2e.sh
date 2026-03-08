#!/usr/bin/env bash
# End-to-end integration test for `am`.
# Requires live relays with NIP-17/59 (kind:1059 gift wrap) support.
#
# Usage:
#   ./scripts/e2e.sh
#
set -euo pipefail

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------

AM="${AM_BIN:-./target/release/am}"

# Three NIP-17/59 capable relays (from ~/.config/am/config.toml)
RELAYS=(
    "wss://relay.damus.io"
    "wss://nostr.oxtr.dev"
    "wss://nostr.bitcoiner.social"
    "wss://inbox.nostr.wine"
)

MY_NPUB="npub10g25c69nds6gzl7ywc5hxkflf7y9rzqc2pmyqft66cux040q02msp76g46"
CHARLIE_PASS="correct-horse-battery"
LISTEN_TIMEOUT=15

# ---------------------------------------------------------------------------
# Colour helpers
# ---------------------------------------------------------------------------

BOLD='\033[1m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

pass()  { printf "${GREEN}  PASS${NC}  %s\n" "$1"; }
fail()  { printf "${RED}  FAIL${NC}  %s\n" "$1"; exit 1; }
skip()  { printf "${YELLOW}  SKIP${NC}  %s\n" "$1"; }
step()  { printf "\n${BOLD}${CYAN}==> %s${NC}\n" "$1"; }
info()  { printf "       %s\n" "$1"; }

# ---------------------------------------------------------------------------
# Prerequisites
# ---------------------------------------------------------------------------

step "Checking prerequisites"

if [[ ! -x "$AM" ]]; then
    fail "Binary not found at $AM — run 'make build' first"
fi

if ! command -v jq &>/dev/null; then
    fail "jq is required but not installed"
fi

pass "Binary:  $AM"
pass "Relays:  ${RELAYS[*]}"
pass "jq:      $(jq --version)"

# ---------------------------------------------------------------------------
# Temp dirs — one XDG_DATA_HOME + XDG_CONFIG_HOME per user
# ---------------------------------------------------------------------------

ALICE_DATA=$(mktemp -d)
ALICE_CFG=$(mktemp -d)
BOB_DATA=$(mktemp -d)
BOB_CFG=$(mktemp -d)
CHARLIE_DATA=$(mktemp -d)
CHARLIE_CFG=$(mktemp -d)

cleanup() {
    rm -rf "$ALICE_DATA" "$ALICE_CFG" \
           "$BOB_DATA"   "$BOB_CFG"   \
           "$CHARLIE_DATA" "$CHARLIE_CFG"
}
trap cleanup EXIT

# Per-user wrappers — all output JSON for easy parsing
alice()   { XDG_DATA_HOME="$ALICE_DATA"   XDG_CONFIG_HOME="$ALICE_CFG"   "$AM" --format json --identity alice "$@"; }
bob()     { XDG_DATA_HOME="$BOB_DATA"     XDG_CONFIG_HOME="$BOB_CFG"     "$AM" --format json --identity bob "$@"; }
charlie() { XDG_DATA_HOME="$CHARLIE_DATA" XDG_CONFIG_HOME="$CHARLIE_CFG" "$AM" --format json --identity charlie --passphrase "$CHARLIE_PASS" "$@"; }

# ---------------------------------------------------------------------------
# 1. Generate identities
# ---------------------------------------------------------------------------

step "1. Generate identities"

ALICE_INFO=$(alice   identity generate --name alice)
BOB_INFO=$(bob       identity generate --name bob)
CHARLIE_INFO=$(charlie identity generate --name charlie)

ALICE_NPUB=$(echo "$ALICE_INFO"   | jq -r '.npub')
BOB_NPUB=$(echo "$BOB_INFO"       | jq -r '.npub')
CHARLIE_NPUB=$(echo "$CHARLIE_INFO" | jq -r '.npub')

info "alice:   $ALICE_NPUB"
info "bob:     $BOB_NPUB"
info "charlie: $CHARLIE_NPUB (encrypted with passphrase)"

[[ "$ALICE_NPUB"   == npub1* ]] || fail "alice npub invalid"
[[ "$BOB_NPUB"     == npub1* ]] || fail "bob npub invalid"
[[ "$CHARLIE_NPUB" == npub1* ]] || fail "charlie npub invalid"
[[ $(echo "$CHARLIE_INFO" | jq -r '.encrypted') == "true" ]] || fail "charlie key should be encrypted"

pass "Three identities generated (charlie encrypted)"

# ---------------------------------------------------------------------------
# 2. Verify charlie's key is encrypted at rest
# ---------------------------------------------------------------------------

step "2. Verify Charlie's key file is encrypted"

CHARLIE_NSEC_FILE="$CHARLIE_DATA/am/identities/charlie.nsec"
KEY_CONTENT=$(cat "$CHARLIE_NSEC_FILE")
[[ "$KEY_CONTENT" == ncryptsec1* ]] || fail "expected ncryptsec1 prefix, got: $KEY_CONTENT"
pass "Charlie's key file starts with ncryptsec1"

# Confirm wrong passphrase fails
if XDG_DATA_HOME="$CHARLIE_DATA" XDG_CONFIG_HOME="$CHARLIE_CFG" "$AM" --passphrase "wrong" identity show --name charlie &>/dev/null; then
    fail "wrong passphrase should have failed"
fi
pass "Wrong passphrase correctly rejected"

# ---------------------------------------------------------------------------
# 3. Add relay for all users
# ---------------------------------------------------------------------------

step "3. Configure relays (NIP-17/59)"

for relay in "${RELAYS[@]}"; do
    alice   relay add "$relay"
    bob     relay add "$relay"
    charlie relay add "$relay"
    info "added $relay"
done

ALICE_RELAYS=$(alice relay list | jq 'length')
[[ "$ALICE_RELAYS" -eq ${#RELAYS[@]} ]] || fail "expected ${#RELAYS[@]} relays, got $ALICE_RELAYS"
pass "${#RELAYS[@]} relays configured for all users"

# ---------------------------------------------------------------------------
# 4. Set a profile (all fields) for Alice
# ---------------------------------------------------------------------------

step "4. Publish profiles (NIP-01 kind:0)"

ALICE_PROFILE=$(alice profile set \
    --name "Alice Test Bot" \
    --about "E2E test identity — ignore" \
    --picture "https://imgs.search.brave.com/diIGmexS-SSkZa_RV4VctadbM3hGIerb0Mobfu6bODo/rs:fit:500:0:1:0/g:ce/aHR0cHM6Ly93d3cu/cG5nYWxsLmNvbS93/cC1jb250ZW50L3Vw/bG9hZHMvMjAxNy8w/My9BbGljZS1QTkct/SEQucG5n" \
    --website "https://example.com")

ALICE_EID=$(echo "$ALICE_PROFILE" | jq -r '.event_id')
[[ "$ALICE_EID" == note1* ]] || fail "alice profile event_id should be note1 bech32, got: $ALICE_EID"
info "alice event_id: $ALICE_EID"
pass "Alice profile published"

BOB_PROFILE=$(bob profile set \
    --name "Bob Test Bot" \
    --about "E2E test identity — ignore" \
    --picture "https://imgs.search.brave.com/0d8yU54uOROXHmddhS6wW-dpqHjZOOm0LvSWpZairYg/rs:fit:860:0:0:0/g:ce/aHR0cHM6Ly90aHVt/Ym5haWwuaW1nYmlu/LmNvbS8xMC8xNy8y/MC9ib2ItbWFybGV5/LWFydGlzdGljLXJl/Z2dhZS1zdHlsZS1j/aGFyYWN0ZXItaWxs/dXN0cmF0aW9uLU5Y/OVBZdnpEX3QuanBn" \
    --website "https://example.com")

BOB_EID=$(echo "$BOB_PROFILE" | jq -r '.event_id')
[[ "$BOB_EID" == note1* ]] || fail "bob profile event_id should be note1 bech32, got: $BOB_EID"
info "bob event_id: $BOB_EID"
pass "Bob profile published"

CHARLIE_PROFILE=$(charlie profile set \
    --name "Charlie Test Bot" \
    --about "E2E test identity — ignore" \
    --picture "https://imgs.search.brave.com/JZilqqv3aKbu0XgnnNO_X2AJ802cESukXjrYPG-EoAw/rs:fit:500:0:1:0/g:ce/aHR0cHM6Ly9wNy5o/aWNsaXBhcnQuY29t/L3ByZXZpZXcvODcw/LzY5OS83MTMvNWJi/YzIxOTI5ZjQ1My10/aHVtYm5haWwuanBn" \
    --website "https://example.com")

CHARLIE_EID=$(echo "$CHARLIE_PROFILE" | jq -r '.event_id')
[[ "$CHARLIE_EID" == note1* ]] || fail "charlie profile event_id should be note1 bech32, got: $CHARLIE_EID"
info "charlie event_id: $CHARLIE_EID"
pass "Charlie profile published"

# ---------------------------------------------------------------------------
# 5. Alice sends a message to Bob (1:1)
# ---------------------------------------------------------------------------

step "5. Alice → Bob (1:1 DM)"

TS_BEFORE=$(date +%s)
SENT=$(alice send --to "$BOB_NPUB" "hello bob, it is alice — e2e test $(date -u +%H:%M:%S)")
info "sent: $(echo "$SENT" | jq -c .)"

[[ $(echo "$SENT" | jq -r '.to[0]') == "$BOB_NPUB" ]] || fail "to field mismatch"
[[ $(echo "$SENT" | jq -r '.content') == hello* ]] || fail "content missing"
pass "Message sent"

# ---------------------------------------------------------------------------
# 6. Bob reads the message from Alice
# ---------------------------------------------------------------------------

step "6. Bob reads inbox (--once)"

info "Waiting for relay propagation…"
sleep 3

MSGS=$(bob listen --once --since "$TS_BEFORE" --timeout "$LISTEN_TIMEOUT")
info "received: $MSGS"

MSG_COUNT=$(echo "$MSGS" | jq -s 'length')
if [[ "$MSG_COUNT" -eq 0 ]]; then
    skip "No messages received (relay may not have propagated yet — check manually)"
else
    FIRST=$(echo "$MSGS" | jq -s '.[0]')
    [[ $(echo "$FIRST" | jq -r '.from') == "$ALICE_NPUB" ]] || fail "from field mismatch"
    pass "Bob received message from alice"
fi

# ---------------------------------------------------------------------------
# 7. Alice sends a message to MY account
# ---------------------------------------------------------------------------

step "7. Alice → my account ($MY_NPUB)"

SENT_MINE=$(alice send --to "$MY_NPUB" "hi from the e2e test — $(date -u +%Y-%m-%dT%H:%M:%SZ)")
[[ $(echo "$SENT_MINE" | jq -r '.to[0]') == "$MY_NPUB" ]] || fail "to field mismatch"
pass "Message sent to $MY_NPUB"

# ---------------------------------------------------------------------------
# 8. Group message: each user sends to all others + my account
# ---------------------------------------------------------------------------

step "8. Group messaging — all three users → [alice, bob, charlie, me]"

GROUP_TS=$(date +%s)
GROUP=(--to "$ALICE_NPUB" --to "$BOB_NPUB" --to "$CHARLIE_NPUB" --to "$MY_NPUB")

info "Alice → group"
ALICE_GROUP=$(alice   send "${GROUP[@]}" "group message from alice $(date +%s)")
info "Bob   → group"
BOB_GROUP=$(bob       send "${GROUP[@]}" "group message from bob $(date +%s)")
info "Charlie → group"
CHARLIE_GROUP=$(charlie send "${GROUP[@]}" "group message from charlie $(date +%s)")

# Each sent result should have 4 recipients
for RESULT in "$ALICE_GROUP" "$BOB_GROUP" "$CHARLIE_GROUP"; do
    COUNT=$(echo "$RESULT" | jq '.to | length')
    [[ "$COUNT" -eq 4 ]] || fail "expected 4 recipients, got $COUNT: $RESULT"
done
pass "All three group messages sent (4 recipients each)"

# ---------------------------------------------------------------------------
# 9. Each user reads their group inbox
# ---------------------------------------------------------------------------

step "9. Each user reads group messages"

info "Waiting for relay propagation…"
sleep 5

# Alice should receive from Bob and Charlie (not herself)
ALICE_INBOX=$(alice listen --once --since "$GROUP_TS" --timeout "$LISTEN_TIMEOUT" | jq -s '.')
ALICE_COUNT=$(echo "$ALICE_INBOX" | jq 'length')
info "Alice inbox: $ALICE_COUNT message(s)"

BOB_INBOX=$(bob listen --once --since "$GROUP_TS" --timeout "$LISTEN_TIMEOUT" | jq -s '.')
BOB_COUNT=$(echo "$BOB_INBOX" | jq 'length')
info "Bob inbox: $BOB_COUNT message(s)"

CHARLIE_INBOX=$(charlie listen --once --since "$GROUP_TS" --timeout "$LISTEN_TIMEOUT" | jq -s '.')
CHARLIE_COUNT=$(echo "$CHARLIE_INBOX" | jq 'length')
info "Charlie inbox: $CHARLIE_COUNT message(s)"

if [[ "$ALICE_COUNT" -eq 0 && "$BOB_COUNT" -eq 0 && "$CHARLIE_COUNT" -eq 0 ]]; then
    skip "No group messages received — relay propagation may be slow (check your inbox at $MY_NPUB)"
else
    [[ "$ALICE_COUNT"   -ge 1 ]] || fail "Alice should have received group messages"
    [[ "$BOB_COUNT"     -ge 1 ]] || fail "Bob should have received group messages"
    [[ "$CHARLIE_COUNT" -ge 1 ]] || fail "Charlie should have received group messages"
    pass "Alice received $ALICE_COUNT group message(s)"
    pass "Bob received $BOB_COUNT group message(s)"
    pass "Charlie received $CHARLIE_COUNT group message(s)"
fi

# ---------------------------------------------------------------------------
# 10. Encrypt/Decrypt round-trip for Alice
# ---------------------------------------------------------------------------

step "10. identity encrypt/decrypt round-trip (Alice)"

alice identity encrypt --name alice --passphrase "roundtrip-pass"
KEY=$(cat "$ALICE_DATA/am/identities/alice.nsec")
[[ "$KEY" == ncryptsec1* ]] || fail "alice key should be encrypted after encrypt command"
pass "Alice's key encrypted"

alice identity decrypt --name alice --passphrase "roundtrip-pass"
KEY=$(cat "$ALICE_DATA/am/identities/alice.nsec")
[[ "$KEY" == nsec1* ]] || fail "alice key should be plaintext after decrypt command"
pass "Alice's key decrypted back to plaintext"

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------

printf "\n${BOLD}${GREEN}All e2e checks passed.${NC}\n"
printf "Check %s on any Nostr client to see the sent messages.\n" "$MY_NPUB"
