# Sashiko Development and CI Tasks

.PHONY: help check-pr check-integration check-all sob lint fmt test integration-test
.PHONY: docs docs-serve audit build clean

# Default target
.DEFAULT_GOAL := help

# List available commands
help:
	@echo "Available targets:"
	@echo ""
	@echo "  Development:"
	@echo "    build             - Build release binary"
	@echo "    fmt               - Auto-format Rust code"
	@echo "    lint              - Run all linters (clippy, fmt --check, yamllint)"
	@echo "    test              - Run unit tests"
	@echo "    docs              - Build rustdoc and assemble Pages site"
	@echo "    docs-serve        - Build docs and serve locally on :8787"
	@echo "    audit             - Run cargo-audit security scan (requires cargo-audit)"
	@echo "    clean             - Remove build artifacts"
	@echo ""
	@echo "  CI Suites:"
	@echo "    check-pr          - Run all PR checks (SOB, Lint, Unit Tests)"
	@echo "    check-integration - Run integration tests"
	@echo "    check-all         - Run the complete check suite (PR + Integration)"
	@echo ""
	@echo "  Utilities:"
	@echo "    sob               - Check Signed-off-by tags (RANGE=HEAD~1..HEAD)"
	@echo "    integration-test  - [Slow] Run integration tests using benchmarks"

# ── Development ──────────────────────────────────────────

# Build release binary
build:
	@cargo build --release

# Auto-format Rust code
fmt:
	@cargo fmt --all

# Run all linters (clippy, fmt --check, yamllint)
lint:
	@cargo clippy --all-targets --all-features --release -- -D warnings
	@cargo fmt --all -- --check
	@yamllint .

# Run unit tests
test:
	@cargo test --release

# Build rustdoc and assemble Pages site
docs:
	@./scripts/docs.sh

# Build docs and serve locally on :8787
docs-serve: docs
	@python3 -m http.server 8787 --directory target/pages

# Run cargo-audit security scan
audit:
	@cargo audit

# Remove build artifacts
clean:
	@cargo clean
	@rm -rf target/pages

# ── CI Suites ────────────────────────────────────────────

# [PR Suite] Run all checks required for a Pull Request (SOB, Lint, Unit Tests)
check-pr: sob lint test

# [Integration Suite] Run the full integration tests
check-integration: integration-test

# Run the complete check suite (PR + Integration)
check-all: check-pr check-integration

# ── Utilities ────────────────────────────────────────────

# Check Signed-off-by tags (default: HEAD~1..HEAD)
RANGE ?= HEAD~1..HEAD
sob:
	@./scripts/check-sob.sh "$(RANGE)"

# [Slow] Run integration tests using benchmarks
integration-test:
	@./scripts/integration-test.sh
