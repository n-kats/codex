# run-tui: test config injection

## Goal

Make it easy to launch `make run-tui` with a predictable, editable config for
manual testing.

## What changed

- Added a repo-tracked `sample_config.toml`.
- `sample_config.toml` now defaults to a manual-test profile that sets `default_permissions = "test_config"` and denies the lab files under `_custom_tests/not_allowed_*`, plus `CODEX_HOME/auth.json` at `/workspace/_cache/codex_home_debug/auth.json` and `_local/codex_homes/auth.json` for the default `make run-tui` flow.
- `make run-tui` now always runs the TUI with:
  - `--config-file /workspace/$(RUN_TUI_CONFIG)` (inside the Docker container)
  - Default: `RUN_TUI_CONFIG=sample_config.toml`
- `make run-tui RUN_TUI_CONFIG=path/to/config.toml` points at any repo-tracked config.
- TUI startup now threads the loader override through the final config reload as well, so the
  selected config stays in effect when startup warnings are collected.

## Notes

- This avoids generating config files under `_tmp/` so the test configuration is
  easy to edit and review in git.
- The checked-in fixture paths for manual permission checks are:
  - `_custom_tests/allowed_file`
  - `_custom_tests/not_allowed_file`
  - `_custom_tests/allowed_dir`
  - `_custom_tests/not_allowed_dir`
- `CODEX_HOME` still defaults to `_cache/codex_home_debug` for state/logs.
