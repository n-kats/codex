# run-tui: test config injection

## Goal

Make it easy to launch `make run-tui` with a predictable, editable config for
manual testing (for example `custom.exec.worker_user` + unified exec).

## What changed

- Added a repo-tracked `sample_config.toml`.
- `make run-tui` now always runs the TUI with:
  - `--config /workspace/$(RUN_TUI_CONFIG)` (inside the Docker container)
  - Default: `RUN_TUI_CONFIG=sample_config.toml`
- `make run-tui RUN_TUI_CONFIG=path/to/config.toml` points at any repo-tracked config.
- TUI startup now threads the loader override through the final config reload as well, so the
  selected config stays in effect when startup warnings are collected.

## Notes

- This avoids generating config files under `_tmp/` so the test configuration is
  easy to edit and review in git.
- `CODEX_HOME` still defaults to `_cache/codex_home_debug` for state/logs.
- `make run-tui` does not attempt to set file capabilities. If `[custom.exec]` is enabled, the
  command runner tries direct `setuid`/`setgid` first and falls back to `sudo -n -u "#UID" -g
  "#GID" -- env -i ...` when direct switching is not permitted. When needed (e.g. arg0 dispatch),
  it preserves argv0 via a `/tmp/codex-argv0/<argv0>` symlink.
- When enabling `[custom.exec]` in this fork, set `[shell_environment_policy].inherit = "core"`
  (the default `"all"` is rejected for safety).
