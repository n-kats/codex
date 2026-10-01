# 20260227 rebase custom report

- `--codex-home` / `CODEX_HOME` 切り替え
  - 実装: `codex-rs/cli/src/main.rs:72`, `codex-rs/arg0/src/lib.rs:190`
  - テスト: `codex-rs/arg0/src/custom_tests.rs:8`, `codex-rs/cli/src/custom_tests.rs:7`
  - 検証: `make verify-codex-home-cli-flag`

- `--config <FILE>` / `--no-config`
  - 実装: `codex-rs/cli/src/main.rs:78`
  - テスト: `codex-rs/core/src/config_loader/tests.rs`（既存）, `codex-rs/cli/src/main.rs:1148`（既存CLIテスト）
  - 検証: `make test-core`

- `--agents-md` と `/custom-agents`
  - 実装: `codex-rs/cli/src/main.rs:109`
  - テスト: `codex-rs/core/src/codex/custom_tests.rs:7`, `codex-rs/tui/src/chatwidget/custom_tests.rs:28`
  - 検証: `make verify-all-custom`

- `CODEX_ADDITIONAL_PROMPT_DIRS`
  - 実装: `codex-rs/core/src/custom_prompts.rs:8`
  - テスト: `codex-rs/core/src/custom_prompts/custom_tests.rs:11`
  - 検証: `make verify-additional-prompt-dirs-env`

- TUI 入力仕様（Enter改行 / Ctrl+Enter, Ctrl+J送信）
  - 実装: `codex-rs/tui/src/public_widgets/composer_input.rs:5`
  - テスト: `codex-rs/tui/src/chatwidget/tests.rs:6960`（関連）, `codex-rs/tui/src/chatwidget/custom_tests.rs:89`
  - 検証: `make verify-tui-enter-newline-ctrl-enter-send`

- 更新チェック（`x.y.z-custom-*` の比較）
  - 実装: `codex-rs/tui/src/updates.rs:242`
  - テスト: `codex-rs/tui/src/updates/custom_tests.rs:7`
  - 検証: `make verify-all-custom`

- shell startup files 制御（`CODEX_SHELL_STARTUP_FILES=clean`）
  - 実装: `codex-rs/core/src/shell_startup_files.rs:6`, `codex-rs/cli/src/main.rs:97`
  - テスト: `codex-rs/core/src/shell_startup_files/custom_tests.rs:25`
  - 検証: `make verify-exec-command-default-login`

- Linux デフォルトシェル優先順（bash優先）
  - 実装: `codex-rs/core/src/shell.rs:265`
  - テスト: `codex-rs/core/tests/suite/user_shell_cmd.rs`（既存）
  - 検証: `make verify-linux-default-shell`

- モデル実行コマンドの worker ユーザー分離（`custom.exec.*`）
  - 実装: `codex-rs/core/src/config/mod.rs:1029`, `codex-rs/core/src/config/mod.rs:1088`, `codex-rs/core/src/spawn.rs:12`
  - テスト: `codex-rs/core/tests/suite/custom_user_shell_cmd.rs:14`
  - 検証: `make verify-command-exec-worker-user`

- UserShell とモデル実行で環境ポリシー分離
  - 実装: `codex-rs/core/src/config/mod.rs:1073`, `codex-rs/core/src/config/mod.rs:2123`
  - テスト: `codex-rs/core/tests/suite/custom_user_shell_cmd.rs:14`
  - 検証: `make verify-command-exec-worker-user`

- shell snapshot の `exports` 秘匿
  - 実装: `codex-rs/core/src/shell_snapshot.rs:244`
  - テスト: `codex-rs/core/src/shell_snapshot.rs:740`
  - 検証: `make test-core`

- tool parallelism テスト安定化（時間判定依存の低減）
  - 実装: `codex-rs/core/tests/suite/tool_parallelism.rs:79`
  - テスト: `codex-rs/core/tests/suite/tool_parallelism.rs:79`
  - 検証: `make test-core`

- execve-wrapper の PATH 解決 / 絶対パス化
  - 実装: `codex-rs/shell-escalation/src/unix/escalate_client.rs:122`
  - テスト: `codex-rs/exec/tests/suite/sandbox.rs:193`（関連）
  - 検証: `cd codex-rs && cargo test -p codex-exec-server --test all suite::accept_elicitation::accept_elicitation_for_prompt_rule`

- custom 専用テスト分離（`custom__` 命名）
  - 実装: `codex-rs/core/src/config/custom_tests.rs:13`, `codex-rs/tui/src/diff_render/custom_tests.rs:51`
  - テスト: `codex-rs/core/src/project_doc/custom_tests.rs:9`
  - 検証: `cargo test custom__`

- Makefile 運用（`cargo +nightly fmt`, verify群, almost, _tmpログ）
  - 実装: `Makefile:135`, `Makefile:194`, `Makefile:190`, `Makefile:104`
  - テスト: `Makefile` 実行ログ（`_tmp/*_test_result.txt`）
  - 検証: `make almost`, `make verify-all-custom`

- NOTICE 追記
  - 実装: `NOTICE:3`
  - テスト: なし
  - 検証: `rg -n "Modifications Copyright" NOTICE`
