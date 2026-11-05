# 20260324 rebase custom report

- CLI / config surfaces
  - 実装: `codex-rs/cli/src/main.rs:76-98`, `codex-rs/cli/src/main.rs:120-123`, `codex-rs/cli/src/main.rs:651-667`, `codex-rs/cli/src/main.rs:1275-1295`, `codex-rs/core/src/config/mod.rs:161`, `codex-rs/core/src/config/mod.rs:219`, `codex-rs/core/src/config/mod.rs:3129-3134`, `codex-rs/core/src/config_loader/mod.rs:185-203`
  - テスト: `codex-rs/cli/src/custom_tests.rs:14-187`, `codex-rs/core/src/config/custom_tests.rs:226`
  - 検証: この環境では `cargo` / `just` が使えないため未実行。手元では `cd codex-rs && cargo test -p codex-cli && cargo test -p codex-core` を確認する。

- Project doc / prompt discovery
  - 実装: `codex-rs/core/src/project_doc.rs:135`, `codex-rs/core/src/project_doc.rs:186-194`, `codex-rs/core/src/custom_prompts.rs:1-43`, `codex-rs/tui/src/lib.rs:353-508`, `codex-rs/tui_app_server/src/app_command.rs:67`, `codex-rs/tui_app_server/src/app_command.rs:189-203`
  - テスト: `codex-rs/core/src/project_doc/custom_tests.rs:2-24`, `codex-rs/core/src/custom_prompts/custom_tests.rs:1-39`
  - 検証: この環境では未実行。手元では `cd codex-rs && cargo test -p codex-core` を確認する。

- Execution isolation
  - 実装: `codex-rs/core/src/config/mod.rs:335`, `codex-rs/core/src/config/mod.rs:2458-2464`, `codex-rs/core/src/config/mod.rs:2513-2539`, `codex-rs/core/src/config/mod.rs:2827-2829`, `codex-rs/core/src/config/custom.rs:13-218`, `codex-rs/core/src/spawn/run_as.rs:1-246`, `codex-rs/core/src/tasks/user_shell.rs:177`, `codex-rs/core/src/tasks/user_shell.rs:332`
  - テスト: `codex-rs/core/tests/suite/custom_exec_command_worker_user.rs:29-197`, `codex-rs/core/tests/suite/custom_user_shell_cmd.rs:14-81`
  - 検証: この環境では未実行。手元では `cd codex-rs && cargo test -p codex-core` を確認する。

- TUI input and update check
  - 実装: `codex-rs/tui/src/app.rs:5268`, `codex-rs/tui/src/app.rs:5350`, `codex-rs/tui/src/app.rs:5431`, `codex-rs/tui/src/app.rs:5506`, `codex-rs/tui/src/app.rs:5664`, `codex-rs/tui/src/app.rs:5747`, `codex-rs/tui/src/app.rs:5903`, `codex-rs/tui/src/app.rs:7995`, `codex-rs/tui/src/updates.rs:40`, `codex-rs/tui/src/updates.rs:123-172`, `codex-rs/tui/src/updates.rs:237-252`
  - テスト: `codex-rs/tui/src/chatwidget/custom_tests.rs:1-80`, `codex-rs/tui/src/updates.rs:237-252`
  - 検証: この環境では未実行。手元では `cd codex-rs && cargo test -p codex-tui` を確認する。

- TUI diff theme
  - 実装: `codex-rs/core/src/config/mod.rs:448`, `codex-rs/core/src/config/mod.rs:450`, `codex-rs/core/src/config/mod.rs:2466-2478`, `codex-rs/core/src/config/mod.rs:2982-2983`, `codex-rs/tui/src/diff_render.rs:325`
  - テスト: `codex-rs/tui/src/diff_render/custom_tests.rs:1-98`
  - 検証: この環境では未実行。手元では `cd codex-rs && cargo test -p codex-tui` を確認する。

- Shell snapshot hygiene
  - 実装: `codex-rs/core/src/shell_snapshot.rs:207`, `codex-rs/core/src/shell_snapshot.rs:244-263`, `codex-rs/core/src/shell_snapshot.rs:394-569`
  - テスト: `codex-rs/core/src/shell_snapshot_tests.rs:72-375`
  - 検証: この環境では未実行。手元では `cd codex-rs && cargo test -p codex-core` を確認する。
