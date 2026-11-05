# 20260601 rebase custom report

- `--codex-home` / `--codex-memory`
  - 実装: `codex-rs/cli/src/main.rs:96`, `codex-rs/cli/src/main.rs:101`, `codex-rs/cli/src/main.rs:792`
  - テスト: `codex-rs/cli/src/custom_tests.rs:24`, `codex-rs/cli/src/custom_tests.rs:31`, `codex-rs/cli/src/custom_tests.rs:77`, `codex-rs/cli/src/custom_tests.rs:100`
  - 検証: `cargo test -p codex-cli --test custom_tests`

- `--config` / `--no-config`
  - 実装: `codex-rs/cli/src/main.rs:112`, `codex-rs/cli/src/main.rs:168`
  - テスト: `codex-rs/cli/src/custom_tests.rs:13`, `codex-rs/cli/src/custom_tests.rs:168`
  - 検証: `cargo test -p codex-cli --test custom_tests`

- `CODEX_ADDITIONAL_PROMPT_DIRS`
  - 実装: `codex-rs/tui/src/custom_prompts.rs:10`, `codex-rs/tui/src/custom_prompts.rs:65`
  - テスト: `codex-rs/tui/src/chatwidget/tests/slash_commands.rs:159`, `codex-rs/tui/src/chatwidget/tests/slash_commands.rs:216`
  - 検証: `cargo test -p codex-tui slash_commands`

- `custom.user_shell.no_inject`
  - 実装: `codex-rs/core/src/config/mod.rs:1113`, `codex-rs/core/src/tasks/user_shell.rs:135`, `codex-rs/core/src/tasks/user_shell.rs:369`
  - テスト: `codex-rs/core/src/config/config_tests.rs:219`, `codex-rs/core/tests/suite/custom_user_shell_cmd.rs:75`
  - 検証: `cargo test -p codex-core custom__user_shell_no_inject__bang_result_not_recorded_locally`

- `custom.exec.worker_user`
  - 実装: `codex-rs/core/src/config/mod.rs:1346`, `codex-rs/core/src/tools/runtimes/apply_patch.rs:354`
  - テスト: `codex-rs/core/tests/suite/custom_exec_command_worker_user.rs:96`, `codex-rs/core/tests/suite/custom_exec_command_worker_user.rs:234`, `codex-rs/core/tests/suite/custom_exec_command_worker_user.rs:347`
  - 検証: `cargo test -p codex-core custom__exec_worker_user__exec_command_tty_false_runs_as_worker_user`

- `custom.agents_md` / `/custom-agents`
  - 実装: `codex-rs/core/src/session/handlers.rs:108`, `codex-rs/tui/src/chatwidget/slash_dispatch.rs:41`, `codex-rs/tui/src/chatwidget/slash_dispatch.rs:58`
  - テスト: `codex-rs/tui/src/chatwidget/tests/slash_commands.rs:159`, `codex-rs/core/src/session/tests.rs:3403`
  - 検証: `cargo test -p codex-tui slash_commands`

- `custom.theme.diff`
  - 実装: `codex-rs/tui/src/render/highlight.rs:1296`
  - テスト: `codex-rs/tui/src/render/highlight.rs:1303`
  - 検証: `cargo test -p codex-tui highlight`

- `shell snapshot` の exports redaction
  - 実装: `codex-rs/core/src/shell_snapshot.rs:37`, `codex-rs/core/src/shell_snapshot.rs:319`
  - テスト: `codex-rs/core/src/shell_snapshot_tests.rs:128`, `codex-rs/core/tests/suite/shell_snapshot.rs:392`
  - 検証: `cargo test -p codex-core shell_snapshot`

- `update_check` の custom 版バージョン suffix
  - 実装: `codex-rs/tui/src/update_versions.rs:81`, `codex-rs/tui/src/updates.rs:230`
  - テスト: `codex-rs/tui/src/updates.rs:238`
  - 検証: `cargo test -p codex-tui update_check`

- custom 専用テスト分離
  - 実装: `codex-rs/cli/src/custom_tests.rs:13`, `codex-rs/core/tests/suite/custom_exec_command_worker_user.rs:96`, `codex-rs/core/tests/suite/custom_user_shell_cmd.rs:29`
  - テスト: `codex-rs/cli/src/custom_tests.rs:53`, `codex-rs/cli/src/custom_tests.rs:63`
  - 検証: `cargo test -p codex-cli --test custom_tests`

