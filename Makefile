.PHONY: help fmt fmt-stable test-tui fix-tui insta-pending-tui insta-show-tui insta-accept-tui run-tui verify-codex-home-cli-flag verify-tui-enter-newline-ctrl-enter-send cache-dir

.DEFAULT_GOAL := help

CODEX_RS_DIR := codex-rs
CACHE_DIR := $(CURDIR)/_cache
CODEX_HOME ?= $(CACHE_DIR)/codex_home
export CODEX_HOME

cache-dir:
	@mkdir -p "$(CODEX_HOME)"

help:
	@printf "%s\n" "Common commands:" \
		"  (Make targets run with CODEX_HOME=$$PWD/_cache/codex_home by default)" \
		"  make fmt              # Format Rust (nightly rustfmt)" \
		"  make fmt-stable       # Format Rust (stable; may ignore some config)" \
		"  make verify-codex-home-cli-flag # Verify --codex-home customization" \
		"  make verify-tui-enter-newline-ctrl-enter-send # Verify TUI Enter newline / Ctrl+Enter send" \
		"  make run-tui          # Run codex TUI" \
		"  make test-tui         # Run TUI tests" \
		"  make fix-tui          # Apply clippy fixes for TUI (interactive use)" \
		"  make insta-pending-tui# List pending TUI snapshots" \
		"  make insta-show-tui FILE=...  # Show a specific .snap.new file" \
		"  make insta-accept-tui # Accept all pending TUI snapshots"

fmt: cache-dir
	cd $(CODEX_RS_DIR) && RUSTUP_TOOLCHAIN=nightly just fmt

fmt-stable: cache-dir
	cd $(CODEX_RS_DIR) && just fmt

verify-codex-home-cli-flag: fmt
	cd $(CODEX_RS_DIR) && just fix -p codex-arg0
	cd $(CODEX_RS_DIR) && cargo test -p codex-arg0
	cd $(CODEX_RS_DIR) && just fix -p codex-cli
	cd $(CODEX_RS_DIR) && cargo test -p codex-cli

verify-tui-enter-newline-ctrl-enter-send: fmt
	$(MAKE) fix-tui
	$(MAKE) test-tui

run-tui: cache-dir
	cd $(CODEX_RS_DIR) && cargo run -p codex-tui --bin codex-tui

test-tui: cache-dir
	cd $(CODEX_RS_DIR) && cargo test -p codex-tui

fix-tui: cache-dir
	cd $(CODEX_RS_DIR) && just fix -p codex-tui

insta-pending-tui: cache-dir
	cd $(CODEX_RS_DIR) && cargo insta pending-snapshots -p codex-tui

insta-show-tui: cache-dir
	cd $(CODEX_RS_DIR) && cargo insta show -p codex-tui $(FILE)

insta-accept-tui: cache-dir
	cd $(CODEX_RS_DIR) && cargo insta accept -p codex-tui
