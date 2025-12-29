.PHONY: help fmt fmt-stable test-tui fix-tui insta-pending-tui insta-show-tui insta-accept-tui run-tui

CODEX_RS_DIR := codex-rs

help:
	@printf "%s\n" "Common commands:" \
		"  make fmt              # Format Rust (nightly rustfmt)" \
		"  make fmt-stable       # Format Rust (stable; may ignore some config)" \
		"  make run-tui          # Run codex TUI" \
		"  make test-tui         # Run TUI tests" \
		"  make fix-tui          # Apply clippy fixes for TUI (interactive use)" \
		"  make insta-pending-tui# List pending TUI snapshots" \
		"  make insta-show-tui FILE=...  # Show a specific .snap.new file" \
		"  make insta-accept-tui # Accept all pending TUI snapshots"

fmt:
	cd $(CODEX_RS_DIR) && RUSTUP_TOOLCHAIN=nightly just fmt

fmt-stable:
	cd $(CODEX_RS_DIR) && just fmt

run-tui:
	cd $(CODEX_RS_DIR) && cargo run -p codex-tui --bin codex-tui

test-tui:
	cd $(CODEX_RS_DIR) && cargo test -p codex-tui

fix-tui:
	cd $(CODEX_RS_DIR) && just fix -p codex-tui

insta-pending-tui:
	cd $(CODEX_RS_DIR) && cargo insta pending-snapshots -p codex-tui

insta-show-tui:
	cd $(CODEX_RS_DIR) && cargo insta show -p codex-tui $(FILE)

insta-accept-tui:
	cd $(CODEX_RS_DIR) && cargo insta accept -p codex-tui
