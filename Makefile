SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

.PHONY: \
	help fetch cache-dir \
	release-dir release \
	docker-build \
	fmt \
	build \
	run-tui run-tui-test test-tui \
	test-custom list-custom-tests \
	update-fixtures \
	lint-arg0 test-arg0 fix-arg0 \
	lint-cli test-cli fix-cli \
	clean clean-dry-run build-linux-sandbox test-core test-all test-almost all almost \
	write-config-schema \
	verify-all-custom verify-codex-home-cli-flag verify-additional-prompt-dirs-env verify-exec-command-default-login verify-linux-default-shell

.DEFAULT_GOAL := help

ROOT_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
ROOT_DIR_DOCKER := /workspace
CODEX_RS_DIR := $(ROOT_DIR)/codex-rs
CODEX_RS_DIR_DOCKER := $(ROOT_DIR_DOCKER)/codex-rs
CACHE_DIR := $(ROOT_DIR)/_cache
TEST_LOG_DIR := $(ROOT_DIR)/_tmp
RELEASE_DIR := $(ROOT_DIR)/_release
RELEASE_DIR_DOCKER := $(ROOT_DIR_DOCKER)/_release
CODEX_HOME ?= $(CACHE_DIR)/codex_home_debug
export CODEX_HOME
CODEX_MEMORIES_HOME ?= $(CACHE_DIR)/codex_memory_debug
RUST_BACKTRACE ?= 1
export RUST_BACKTRACE
DOCKER_DIR := $(ROOT_DIR)/docker
CODEX_DOCKER_IMAGE_NAME ?= codex-dev
CODEX_DOCKER_PLATFORM ?=
CODEX_DOCKER_CACHE_DIR ?= $(CACHE_DIR)/docker
DOCKER_RUN := $(ROOT_DIR)/scripts/docker_run.sh
RUN_TUI_CONFIG ?= sample_config.toml
ALMOST_SKIP_TESTS_FILE := $(ROOT_DIR)/skip_test_list.txt
SKIP_ALMOST_TESTS ?= $(strip $(shell awk 'NF && $$1 !~ /^#/ { print $$1 }' "$(ALMOST_SKIP_TESTS_FILE)"))
RUST_TOOLCHAIN := $(shell awk -F'"' '/^channel = / { print $$2; exit }' "$(CODEX_RS_DIR)/rust-toolchain.toml")

# Extra flags passed to `cargo test` (example: `make test-almost CARGO_TEST_FLAGS=--no-fail-fast`).
CARGO_TEST_FLAGS ?=

# Speed-first overrides for non-release Rust targets (disable: `make CARGO_FAST_BUILD=0 ...`).
CARGO_FAST_BUILD ?= 1
CARGO_FAST_DEBUG ?= 1
CARGO_FAST_CODEGEN_UNITS ?= 16
CARGO_FAST_LTO ?= off
CARGO_FAST_INCREMENTAL ?= false
CARGO_FAST_OPT_LEVEL ?= 0
CARGO_TEST_FAST_DEBUG ?= 0

ifeq ($(CARGO_FAST_BUILD),1)
CARGO_NON_RELEASE_EXPORTS := \
	export CARGO_PROFILE_DEV_DEBUG=$(CARGO_FAST_DEBUG); \
	export CARGO_PROFILE_TEST_DEBUG=$(CARGO_FAST_DEBUG); \
	export CARGO_PROFILE_DEV_CODEGEN_UNITS=$(CARGO_FAST_CODEGEN_UNITS); \
	export CARGO_PROFILE_TEST_CODEGEN_UNITS=$(CARGO_FAST_CODEGEN_UNITS); \
	export CARGO_PROFILE_DEV_LTO=$(CARGO_FAST_LTO); \
	export CARGO_PROFILE_TEST_LTO=$(CARGO_FAST_LTO); \
	export CARGO_PROFILE_DEV_INCREMENTAL=$(CARGO_FAST_INCREMENTAL); \
	export CARGO_PROFILE_TEST_INCREMENTAL=$(CARGO_FAST_INCREMENTAL); \
	export CARGO_PROFILE_DEV_OPT_LEVEL=$(CARGO_FAST_OPT_LEVEL); \
	export CARGO_PROFILE_TEST_OPT_LEVEL=$(CARGO_FAST_OPT_LEVEL); \
	export V8_FROM_SOURCE=1; \
else
CARGO_NON_RELEASE_EXPORTS :=
endif

ifeq ($(CARGO_FAST_BUILD),1)
CARGO_TEST_NON_RELEASE_EXPORTS := \
	export CARGO_PROFILE_DEV_DEBUG=$(CARGO_TEST_FAST_DEBUG); \
	export CARGO_PROFILE_TEST_DEBUG=$(CARGO_TEST_FAST_DEBUG); \
	export CARGO_PROFILE_DEV_CODEGEN_UNITS=$(CARGO_FAST_CODEGEN_UNITS); \
	export CARGO_PROFILE_TEST_CODEGEN_UNITS=$(CARGO_FAST_CODEGEN_UNITS); \
	export CARGO_PROFILE_DEV_LTO=$(CARGO_FAST_LTO); \
	export CARGO_PROFILE_TEST_LTO=$(CARGO_FAST_LTO); \
	export CARGO_PROFILE_DEV_INCREMENTAL=$(CARGO_FAST_INCREMENTAL); \
	export CARGO_PROFILE_TEST_INCREMENTAL=$(CARGO_FAST_INCREMENTAL); \
	export CARGO_PROFILE_DEV_OPT_LEVEL=$(CARGO_FAST_OPT_LEVEL); \
	export CARGO_PROFILE_TEST_OPT_LEVEL=$(CARGO_FAST_OPT_LEVEL); \
	export V8_FROM_SOURCE=1; \
else
CARGO_TEST_NON_RELEASE_EXPORTS :=
endif

cache-dir:
	@mkdir -p "$(CODEX_HOME)" "$(CODEX_MEMORIES_HOME)"

fetch:
	@git fetch --all
	@if git rev-parse --is-shallow-repository | grep -q true; then \
		git fetch fork-origin --prune --tags --force --unshallow; \
	else \
		git fetch fork-origin --prune --tags --force; \
	fi
	@if git rev-parse --verify --quiet custom >/dev/null && git rev-parse --verify --quiet fork-origin/main >/dev/null; then \
		set -- $$(git rev-list --left-right --count custom...fork-origin/main); \
		custom_ahead="$$1"; \
		main_ahead="$$2"; \
		printf '%s\n' "custom vs fork-origin/main: custom ahead $$custom_ahead, fork-origin/main ahead $$main_ahead"; \
	fi

release-dir:
	@mkdir -p "$(RELEASE_DIR)"

docker-build:
	@docker build $(if $(CODEX_DOCKER_PLATFORM),--platform $(CODEX_DOCKER_PLATFORM),) -t "$(CODEX_DOCKER_IMAGE_NAME)" -f "$(DOCKER_DIR)/Dockerfile" "$(ROOT_DIR)"

clean: docker-build
	@# Make runs recipes with `-u` (nounset). Set a placeholder so the host shell expands
	@# `$CARGO_TARGET_DIR` into `${CARGO_TARGET_DIR}`, leaving the real expansion for the container.
	@export CARGO_TARGET_DIR='$${CARGO_TARGET_DIR}'; \
	$(call run_docker,if [ -d "$$CARGO_TARGET_DIR" ]; then shopt -s dotglob nullglob; rm -rf -- "$$CARGO_TARGET_DIR"/*; shopt -u dotglob nullglob; else mkdir -p "$$CARGO_TARGET_DIR"; fi; cd "$(CODEX_RS_DIR_DOCKER)" && if [ -d target ]; then shopt -s dotglob nullglob; rm -rf -- target/*; shopt -u dotglob nullglob; else mkdir -p target; fi)

clean-dry-run: docker-build
	@# Print the directories and entries that `make clean` would remove.
	@export CARGO_TARGET_DIR='$${CARGO_TARGET_DIR}'; \
	$(call run_docker,echo "CARGO_TARGET_DIR=$$CARGO_TARGET_DIR"; if [ -d "$$CARGO_TARGET_DIR" ]; then echo "[would remove] $$CARGO_TARGET_DIR/*"; find "$$CARGO_TARGET_DIR" -mindepth 1 -maxdepth 1 -print | sort; else echo "[missing] $$CARGO_TARGET_DIR"; fi; cd "$(CODEX_RS_DIR_DOCKER)" && echo "WORKSPACE_TARGET=$$(pwd)/target" && if [ -d target ]; then echo "[would remove] $$(pwd)/target/*"; find target -mindepth 1 -maxdepth 1 -print | sort; else echo "[missing] $$(pwd)/target"; fi)

define run_test_logged
	@mkdir -p "$(TEST_LOG_DIR)"; \
	log_file="$${LOG_FILE:-$(TEST_LOG_DIR)/$(1)_test_result.txt}"; \
	printf "%s\n" "log: $$log_file"; \
	LOG_FILE="$$log_file" LOG_APPEND="$${LOG_APPEND:-0}" CODEX_SHELL_STARTUP_FILES="$${CODEX_SHELL_STARTUP_FILES:-clean}" \
	CODEX_DOCKER_IMAGE_NAME="$(CODEX_DOCKER_IMAGE_NAME)" CODEX_DOCKER_PLATFORM="$(CODEX_DOCKER_PLATFORM)" \
	CODEX_DOCKER_CACHE_DIR="$(CODEX_DOCKER_CACHE_DIR)" DOCKER_RUN="$(DOCKER_RUN)" \
	bash "$(ROOT_DIR)/scripts/run_logged.sh" "$(1)" "$(2)"
endef

define run_docker
	CODEX_DOCKER_IMAGE_NAME="$(CODEX_DOCKER_IMAGE_NAME)" \
	CODEX_DOCKER_PLATFORM="$(CODEX_DOCKER_PLATFORM)" \
	CODEX_DOCKER_CACHE_DIR="$(CODEX_DOCKER_CACHE_DIR)" \
	bash "$(DOCKER_RUN)" "$(1)"
endef

define run_targets_continue
	@status=0; \
	for target in $(1); do \
		$(MAKE) $$target || status=$$?; \
	done; \
	exit $$status
endef

define run_targets_continue_logged
	@mkdir -p "$(TEST_LOG_DIR)"; \
	log_file="$(TEST_LOG_DIR)/$(1)_test_result.txt"; \
	printf "%s\n" "log: $$log_file"; \
	/bin/bash -lc 'set -o pipefail; LOG_FILE="'"$$log_file"'"; : > "$$LOG_FILE"; status=0; for target in $(2); do printf "%s\n" "==> $$target ($$(date -Is))" | tee -a "$$LOG_FILE"; LOG_APPEND=1 $(MAKE) --no-print-directory "$$target" LOG_FILE="$$LOG_FILE" 2>&1 | tee -a "$$LOG_FILE"; rc=$${PIPESTATUS[0]}; if [ $$rc -ne 0 ]; then status=$$rc; fi; done; exit $$status'
endef

help:
	@printf "%s\n" "Common commands:" \
		"  (Make targets run with CODEX_HOME=$$PWD/_cache/codex_home_debug by default)" \
		"  (run-tui uses CODEX_MEMORIES_HOME=$$PWD/_cache/codex_memory_debug by default)" \
		"  (Docker image: $(CODEX_DOCKER_IMAGE_NAME), cache: $(CODEX_DOCKER_CACHE_DIR))" \
		"  (Docker target dir: set CODEX_DOCKER_TARGET_DIR in .env)" \
		"  (Test targets also tee logs to $$PWD/_tmp/*_test_result.txt)" \
		"  (Test targets default CODEX_SHELL_STARTUP_FILES=clean; override with 'make CODEX_SHELL_STARTUP_FILES=default ...')" \
		"  (Pass extra cargo flags with CARGO_TEST_FLAGS=...; example: 'make test-almost CARGO_TEST_FLAGS=--no-fail-fast')" \
		"  (Non-release Rust builds are speed-first by default; disable with 'make CARGO_FAST_BUILD=0 ...')" \
		"" \
		"  make fetch            # git fetch --all + fork-origin tags + custom/main distance" \
		"  make docker-build     # Build the Docker image for build/test" \
		"  make fmt              # Format Rust" \
		"  make build            # Build codex (CLI entrypoint)" \
		"  make run-tui          # Run codex TUI (uses RUN_TUI_CONFIG)" \
		"  make run-tui-test     # Alias for run-tui" \
		"  make release          # Build Linux release tarball into ./_release" \
		"" \
		"  make verify-all-custom# Run all custom verifications (no auto-fix)" \
		"  make verify-codex-home-cli-flag # Verify --codex-home customization" \
		"  make verify-additional-prompt-dirs-env # Verify CODEX_ADDITIONAL_PROMPT_DIRS customization" \
		"  make verify-exec-command-default-login # Verify exec_command default login behavior" \
		"  make verify-linux-default-shell # Verify Linux: zsh login shell is controllable (no user dotfiles)" \
		"" \
		"  make run-tui          # Run codex TUI" \
		"  make test-tui         # Run TUI tests" \
		"  make test-custom      # Run custom-only tests (cargo test custom__)" \
		"  make list-custom-tests# List custom__ test names from Rust sources" \
		"  make test-core        # Run codex-core tests (builds linux sandbox first on Linux)" \
		"  make test-all         # Run full Rust test suite (all features)" \
		"  make test-almost      # Run tests skipping known flaky cases (default features)" \
		"  make update-fixtures  # Regenerate config schema + accept snapshots" \
		"  make all              # Run format + test-all" \
		"  make almost           # Run format + test-almost" \
		"  make clean            # Remove Rust build artifacts (mounted CARGO_TARGET_DIR + codex-rs/target)" \
		"  make clean-dry-run    # Print what clean would remove"

# Formatting
fmt: cache-dir docker-build
	$(call run_test_logged,fmt,cd "$(CODEX_RS_DIR_DOCKER)" && cargo +nightly fmt)

write-config-schema: cache-dir docker-build
	$(call run_test_logged,write_config_schema,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo run -p codex-core --bin codex-write-config-schema)

update-fixtures: cache-dir docker-build
	@# One command to update generated artifacts that are committed to the repo.
	$(MAKE) --no-print-directory write-config-schema
	$(call run_docker,cd "$(CODEX_RS_DIR_DOCKER)/tui" && cargo insta accept)

# Build
build: cache-dir docker-build
	$(call run_test_logged,build_cli,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo build -p codex-cli --bin codex)

release: cache-dir release-dir docker-build
	$(call run_test_logged,release,/bin/bash "$(ROOT_DIR_DOCKER)/scripts/release_linux.sh" "$(ROOT_DIR_DOCKER)" "$(CODEX_RS_DIR_DOCKER)" "$(RELEASE_DIR_DOCKER)")

# Lint / test helpers (no auto-fix)
lint-arg0: cache-dir docker-build
	$(call run_docker,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo clippy -p codex-arg0 --all-features --tests)

test-arg0: cache-dir docker-build
	$(call run_docker,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test -p codex-arg0 --lib)

fix-arg0: cache-dir docker-build
	$(call run_docker,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && just fix -p codex-arg0)

lint-cli: cache-dir docker-build
	$(call run_docker,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo clippy -p codex-cli --all-features --tests)

test-cli: cache-dir docker-build
	$(call run_docker,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test -p codex-cli --bin codex)

fix-cli: cache-dir docker-build
	$(call run_docker,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && just fix -p codex-cli)

build-linux-sandbox: cache-dir docker-build
	@# Some test suites expect `codex-linux-sandbox` and its bundled bwrap to exist as standalone binaries.
	@# `cargo test -p codex-core` does not necessarily build them, so build them explicitly on Linux.
	@if [ "$$(uname -s)" = "Linux" ]; then \
		$(call run_docker,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo build -p codex-linux-sandbox -p codex-bwrap); \
	fi

test-core: build-linux-sandbox docker-build
	$(call run_test_logged,test_core,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test $(CARGO_TEST_FLAGS) -p codex-core)

test-all: build-linux-sandbox docker-build
	$(call run_test_logged,test_all,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test $(CARGO_TEST_FLAGS) --all-features)

all:
	$(call run_targets_continue_logged,all,fmt test-all)

test-almost: build-linux-sandbox docker-build
	@# `--all-features` tends to blow up the build matrix and `target/` size; keep `test-all` for that.
	$(call run_test_logged,test_almost,$(CARGO_TEST_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test $(CARGO_TEST_FLAGS) -- $(foreach test,$(SKIP_ALMOST_TESTS),--skip $(test)))

almost:
	$(call run_targets_continue_logged,almost,fmt test-almost)

# Custom verifications
verify-all-custom:
	$(call run_targets_continue_logged,verify_all_custom,verify-codex-home-cli-flag verify-additional-prompt-dirs-env verify-exec-command-default-login verify-linux-default-shell)

verify-codex-home-cli-flag:
	$(call run_targets_continue_logged,verify_codex_home_cli_flag,fmt lint-arg0 test-arg0 lint-cli test-cli)

verify-additional-prompt-dirs-env: cache-dir docker-build
	$(call run_test_logged,verify_additional_prompt_dirs_parse,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test -p codex-core --lib custom__追加プロンプトディレクトリ__カンマ区切りと相対パスを解決できる)
	$(call run_test_logged,verify_additional_prompt_dirs_discover,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test -p codex-core --lib custom__追加プロンプトディレクトリ__同名は後勝ちで名前順に並ぶ)

verify-exec-command-default-login: build-linux-sandbox docker-build
	$(call run_test_logged,verify_exec_command_default_login,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test -p codex-app-server --test all suite::v2::turn_start::command_execution_notifications_include_process_id)
	$(call run_test_logged,verify_exec_command_default_login_derive_exec_args,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test -p codex-core --lib shell_tests::derive_exec_args)
	$(call run_test_logged,verify_exec_command_default_login_shell_startup_files,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test -p codex-core --lib custom__シェル起動ファイル__)
	$(call run_test_logged,verify_exec_command_default_login_cli_flag,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test -p codex-cli --bin codex custom__shell_startup_files_cli_flag__)

verify-linux-default-shell: cache-dir docker-build
	$(call run_test_logged,verify_linux_shell_detect_zsh,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test -p codex-core --lib shell::tests::test_current_shell_detects_zsh)
	$(call run_test_logged,verify_linux_bash_snapshot_sections,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test -p codex-core --lib shell_snapshot::tests::linux_bash_snapshot_includes_sections)
	$(call run_test_logged,verify_linux_sh_snapshot_sections,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test -p codex-core --lib shell_snapshot::tests::linux_sh_snapshot_includes_sections)
	$(call run_test_logged,verify_linux_snapshot_file_lifecycle,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test -p codex-core --lib shell_snapshot::tests::try_new_creates_and_deletes_snapshot_file)

# TUI helpers
run-tui: cache-dir docker-build
	@CODEX_MEMORIES_HOME="$(CODEX_MEMORIES_HOME)" \
	$(call run_docker,$(CARGO_NON_RELEASE_EXPORTS) cargo +$(RUST_TOOLCHAIN) run --manifest-path "$(CODEX_RS_DIR_DOCKER)/Cargo.toml" -p codex-cli --bin codex -- --config-file "$(ROOT_DIR_DOCKER)/$(RUN_TUI_CONFIG)")

run-tui-test: docker-build
	@$(MAKE) --no-print-directory run-tui

test-tui: cache-dir docker-build
	$(call run_test_logged,test_tui,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test -p codex-tui)

test-custom: build-linux-sandbox docker-build
	$(call run_test_logged,test_custom,$(CARGO_NON_RELEASE_EXPORTS) cd "$(CODEX_RS_DIR_DOCKER)" && cargo test custom__)

list-custom-tests:
	@mkdir -p "$(TEST_LOG_DIR)"; \
	log_file="$(TEST_LOG_DIR)/list_custom_tests_test_result.txt"; \
	printf "%s\n" "log: $$log_file"; \
	{ \
		printf "==> list_custom_tests (%s)\n" "$$(date -Is)"; \
		cd "$(CODEX_RS_DIR)"; \
		rg --no-heading --line-number -g '*.rs' '(^|[[:space:]])(async[[:space:]]+)?fn[[:space:]]+custom__[^[:space:](]+' \
			| sed -E 's#^([^:]+):([0-9]+):.*fn[[:space:]]+(custom__[^[:space:](]+).*#\1:\2 \3#' \
			| sort -u; \
	} | tee "$$log_file"
