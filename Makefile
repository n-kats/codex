.PHONY: \
	help cache-dir tmp-dir \
	fmt \
	build \
	run-tui test-tui \
	insta-pending-tui insta-show-tui insta-accept-tui \
	lint-arg0 test-arg0 fix-arg0 \
	lint-cli test-cli fix-cli \
	clean build-linux-sandbox test-core test-all test-almost all almost \
	verify-all-custom verify-codex-home-cli-flag verify-tui-enter-newline-ctrl-enter-send verify-additional-prompt-dirs-env verify-exec-command-default-login verify-linux-default-shell verify-command-exec-worker-user

.DEFAULT_GOAL := help

ROOT_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
CODEX_RS_DIR := $(ROOT_DIR)/codex-rs
CACHE_DIR := $(ROOT_DIR)/_cache
TMP_DIR := $(ROOT_DIR)/_tmp
CODEX_HOME ?= $(CACHE_DIR)/codex_home
export CODEX_HOME

SKIP_ALMOST_TESTS ?= \
	view_image_tool_attaches_local_image \
	approval_matrix_covers_all_modes

cache-dir:
	@mkdir -p "$(CODEX_HOME)"

tmp-dir:
	@mkdir -p "$(TMP_DIR)"

clean:
	cd "$(CODEX_RS_DIR)" && cargo clean

define run_test_logged
	@mkdir -p "$(TMP_DIR)"; \
	log_file="$${LOG_FILE:-$(TMP_DIR)/$(1)_test_result.txt}"; \
	printf "%s\n" "log: $$log_file"; \
	LOG_FILE="$$log_file" LOG_APPEND="$${LOG_APPEND:-0}" CODEX_SHELL_STARTUP_FILES="$${CODEX_SHELL_STARTUP_FILES:-clean}" /bin/bash -lc 'set -o pipefail; mode="$${LOG_APPEND:-0}"; { printf "%s\n" "==> $(1) ($$(date -Is))"; } | { if [ "$$mode" = "1" ]; then tee -a "$$LOG_FILE"; else tee "$$LOG_FILE"; fi; } >/dev/null; if [ "$$mode" = "1" ]; then $(2) 2>&1 | tee -a "$$LOG_FILE"; else $(2) 2>&1 | tee "$$LOG_FILE"; fi'
endef

define run_targets_continue
	@status=0; \
	for target in $(1); do \
		$(MAKE) $$target || status=$$?; \
	done; \
	exit $$status
endef

define run_targets_continue_logged
	@mkdir -p "$(TMP_DIR)"; \
	log_file="$(TMP_DIR)/$(1)_test_result.txt"; \
	printf "%s\n" "log: $$log_file"; \
	/bin/bash -lc 'set -o pipefail; LOG_FILE="'"$$log_file"'"; : > "$$LOG_FILE"; status=0; for target in $(2); do printf "%s\n" "==> $$target ($$(date -Is))" | tee -a "$$LOG_FILE"; LOG_APPEND=1 $(MAKE) --no-print-directory "$$target" LOG_FILE="$$LOG_FILE" 2>&1 | tee -a "$$LOG_FILE"; rc=$${PIPESTATUS[0]}; if [ $$rc -ne 0 ]; then status=$$rc; fi; done; exit $$status'
endef

help:
	@printf "%s\n" "Common commands:" \
		"  (Make targets run with CODEX_HOME=$$PWD/_cache/codex_home by default)" \
		"  (Test targets also tee logs to $$PWD/_tmp/*_test_result.txt)" \
		"  (Test targets default CODEX_SHELL_STARTUP_FILES=clean; override with 'make CODEX_SHELL_STARTUP_FILES=default ...')" \
		"" \
		"  make fmt              # Format Rust" \
		"  make build            # Build codex (CLI entrypoint)" \
		"" \
		"  make verify-all-custom# Run all custom verifications (no auto-fix)" \
		"  make verify-codex-home-cli-flag # Verify --codex-home customization" \
		"  make verify-tui-enter-newline-ctrl-enter-send # Verify TUI Enter newline / Ctrl+Enter send" \
		"  make verify-additional-prompt-dirs-env # Verify CODEX_ADDITIONAL_PROMPT_DIRS customization" \
		"  make verify-exec-command-default-login # Verify exec_command default login behavior" \
		"  make verify-linux-default-shell # Verify Linux: zsh login shell is controllable (no user dotfiles)" \
		"  make verify-command-exec-worker-user # Verify custom.exec.* worker-only command spawning" \
		"" \
		"  make run-tui          # Run codex TUI" \
		"  make test-tui         # Run TUI tests" \
		"  make test-core        # Run codex-core tests (builds linux sandbox first on Linux)" \
		"  make test-all         # Run full Rust test suite (all features)" \
		"  make test-almost      # Run tests skipping known flaky cases" \
		"  make all              # Run format + test-all" \
		"  make almost           # Run format + test-almost (logs to _tmp/almost_test_result.txt)" \
		"  make clean            # Remove Rust build artifacts (codex-rs/target)" \
		"  make insta-pending-tui# List pending TUI snapshots" \
		"  make insta-show-tui FILE=...  # Show a specific .snap.new file" \
		"  make insta-accept-tui # Accept all pending TUI snapshots"

# Formatting
fmt: cache-dir
	$(call run_test_logged,fmt,cd "$(CODEX_RS_DIR)" && cargo +nightly fmt)

# Build
build: cache-dir
	$(call run_test_logged,build_cli,cd "$(CODEX_RS_DIR)" && cargo build -p codex-cli --bin codex)

# Lint / test helpers (no auto-fix)
lint-arg0: cache-dir
	cd "$(CODEX_RS_DIR)" && cargo clippy -p codex-arg0 --all-features --tests

test-arg0: cache-dir
	cd "$(CODEX_RS_DIR)" && cargo test -p codex-arg0 --lib

fix-arg0: cache-dir
	just fix -p codex-arg0

lint-cli: cache-dir
	cd "$(CODEX_RS_DIR)" && cargo clippy -p codex-cli --all-features --tests

test-cli: cache-dir
	cd "$(CODEX_RS_DIR)" && cargo test -p codex-cli --bin codex

fix-cli: cache-dir
	just fix -p codex-cli

build-linux-sandbox: cache-dir
	@# Some test suites expect `codex-linux-sandbox` to exist as a standalone binary.
	@# `cargo test -p codex-core` does not necessarily build it, so build it explicitly on Linux.
	@if [ "$$(uname -s)" = "Linux" ]; then \
		cd "$(CODEX_RS_DIR)" && cargo build -p codex-linux-sandbox; \
	fi

test-core: build-linux-sandbox
	$(call run_test_logged,test_core,cd "$(CODEX_RS_DIR)" && cargo test -p codex-core)

test-all: build-linux-sandbox
	$(call run_test_logged,test_all,cd "$(CODEX_RS_DIR)" && cargo test --all-features)

all:
	$(call run_targets_continue_logged,all,fmt test-all)

test-almost: build-linux-sandbox
	$(call run_test_logged,test_almost,cd "$(CODEX_RS_DIR)" && cargo test --all-features -- $(foreach test,$(SKIP_ALMOST_TESTS),--skip $(test)))

almost:
	$(call run_targets_continue_logged,almost,fmt test-almost)

# Custom verifications
verify-all-custom:
	$(call run_targets_continue_logged,verify_all_custom,verify-codex-home-cli-flag verify-tui-enter-newline-ctrl-enter-send verify-additional-prompt-dirs-env verify-exec-command-default-login verify-linux-default-shell verify-command-exec-worker-user)

verify-codex-home-cli-flag:
	$(call run_targets_continue_logged,verify_codex_home_cli_flag,fmt lint-arg0 test-arg0 lint-cli test-cli)

verify-tui-enter-newline-ctrl-enter-send: cache-dir
	$(call run_test_logged,verify_tui_enter_newline,cd "$(CODEX_RS_DIR)" && cargo test -p codex-tui --lib enter_inserts_newline_instead_of_submitting)
	$(call run_test_logged,verify_tui_ctrl_enter_send,cd "$(CODEX_RS_DIR)" && cargo test -p codex-tui --lib ctrl_enter_submits_single_line_text)
	$(call run_test_logged,verify_tui_slash_tab_ctrl_enter,cd "$(CODEX_RS_DIR)" && cargo test -p codex-tui --lib slash_tab_then_ctrl_enter_dispatches_builtin_command)
	$(call run_test_logged,verify_tui2_enter_newline,cd "$(CODEX_RS_DIR)" && cargo test -p codex-tui2 --lib enter_inserts_newline_instead_of_submitting)
	$(call run_test_logged,verify_tui2_ctrl_enter_send,cd "$(CODEX_RS_DIR)" && cargo test -p codex-tui2 --lib ctrl_enter_submits_single_line_text)
	$(call run_test_logged,verify_tui2_slash_tab_ctrl_enter,cd "$(CODEX_RS_DIR)" && cargo test -p codex-tui2 --lib slash_tab_then_ctrl_enter_dispatches_builtin_command)

verify-additional-prompt-dirs-env: cache-dir
	$(call run_test_logged,verify_additional_prompt_dirs_parse,cd "$(CODEX_RS_DIR)" && cargo test -p codex-core --lib custom_prompts::tests::parse_additional_prompts_dirs_splits_and_resolves_relative_paths)
	$(call run_test_logged,verify_additional_prompt_dirs_discover,cd "$(CODEX_RS_DIR)" && cargo test -p codex-core --lib custom_prompts::tests::discover_prompts_in_dirs_last_write_wins_and_sorts)

verify-exec-command-default-login: build-linux-sandbox
	$(call run_test_logged,verify_exec_command_default_login,cd "$(CODEX_RS_DIR)" && cargo test -p codex-app-server --test all suite::v2::turn_start::command_execution_notifications_include_process_id)

verify-linux-default-shell: cache-dir
	$(call run_test_logged,verify_linux_shell_detect_zsh,cd "$(CODEX_RS_DIR)" && cargo test -p codex-core --lib shell::tests::test_current_shell_detects_zsh)
	$(call run_test_logged,verify_linux_bash_snapshot_sections,cd "$(CODEX_RS_DIR)" && cargo test -p codex-core --lib shell_snapshot::tests::linux_bash_snapshot_includes_sections)
	$(call run_test_logged,verify_linux_sh_snapshot_sections,cd "$(CODEX_RS_DIR)" && cargo test -p codex-core --lib shell_snapshot::tests::linux_sh_snapshot_includes_sections)
	$(call run_test_logged,verify_linux_snapshot_file_lifecycle,cd "$(CODEX_RS_DIR)" && cargo test -p codex-core --lib shell_snapshot::tests::try_new_creates_and_deletes_snapshot_file)

verify-command-exec-worker-user: cache-dir
	$(call run_test_logged,verify_command_exec_worker_user,cd "$(CODEX_RS_DIR)" && cargo test -p codex-core --lib config::custom_exec_tests)
	$(call run_test_logged,verify_command_exec_worker_user_exec_command_sudo,cd "$(CODEX_RS_DIR)" && cargo test -p codex-core --lib prepare_pty_command_)

# TUI helpers
run-tui: cache-dir
	cd "$(CODEX_RS_DIR)" && cargo run -p codex-cli --bin codex

test-tui: cache-dir
	$(call run_test_logged,test_tui,cd "$(CODEX_RS_DIR)" && cargo test -p codex-tui)

test-tui2: cache-dir
	$(call run_test_logged,test_tui2,cd "$(CODEX_RS_DIR)" && cargo test -p codex-tui2)

insta-pending-tui: cache-dir
	cd "$(CODEX_RS_DIR)" && cargo insta pending-snapshots -p codex-tui

insta-show-tui: cache-dir
	cd "$(CODEX_RS_DIR)" && cargo insta show -p codex-tui $(FILE)

insta-accept-tui: cache-dir
	cd "$(CODEX_RS_DIR)" && cargo insta accept -p codex-tui

insta-pending-tui2: cache-dir
	cd "$(CODEX_RS_DIR)" && cargo insta pending-snapshots -p codex-tui2

insta-show-tui2: cache-dir
	cd "$(CODEX_RS_DIR)" && cargo insta show -p codex-tui2 $(FILE)

insta-accept-tui2: cache-dir
	cd "$(CODEX_RS_DIR)" && cargo insta accept -p codex-tui2
