.PHONY: build clean test test-unit test-e2e clippy fmt fmt-check

BINARY := target/release/am
SCRIPTS := scripts

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

build:
	cargo build --release

# Remove only the release binary, keeping incremental build cache
clean:
	rm -f $(BINARY)

# Full cargo clean — removes all build artifacts
clean-all:
	cargo clean

# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

# Unit + integration tests (no relay required)
test-unit:
	cargo test

# End-to-end test against three live NIP-17/59 relays (requires built binary)
test-e2e: build
	$(SCRIPTS)/e2e.sh

# Run both
test: test-unit test-e2e

# ---------------------------------------------------------------------------
# Lint / format
# ---------------------------------------------------------------------------

clippy:
	cargo clippy --all-targets

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check
