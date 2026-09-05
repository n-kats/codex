# 20260226 Rebase Custom Report

## range-diff

- 実行:
  - `git range-diff "$(git merge-base fork-origin/main tmp-rebase)"..tmp-rebase "$(git merge-base fork-origin/main custom)"..custom`
- 出力:
  - `_tmp/range-diff/20260226-rebase-range-diff.txt`
- 目視確認:
  - 変化コミットは `!` が 1 件（`custom changes`）のみ。

## カスタム仕様チェック

- TUI 入力（Enter 改行、Ctrl+Enter/Ctrl+J 送信、Tab は `!` 入力時非送信）
  - 実装: `codex-rs/tui/src/bottom_pane/chat_composer.rs:2366`, `codex-rs/tui/src/bottom_pane/chat_composer.rs:2381`
  - テスト: `codex-rs/tui/src/bottom_pane/chat_composer.rs:5613`
  - 検証: `make verify-tui-enter-newline-ctrl-enter-send`

- カスタム版バージョンの更新判定
  - 実装: `codex-rs/tui/src/updates.rs:123`, `codex-rs/tui/src/updates.rs:172`
  - テスト: `codex-rs/tui/src/updates.rs:258`
  - 検証: `cargo test -p codex-tui updates`

- `--codex-home` / `CODEX_HOME`
  - 実装: `codex-rs/cli/src/main.rs:74`
  - テスト: `なし`
  - 検証: `make verify-codex-home-cli-flag`

- `CODEX_ADDITIONAL_PROMPT_DIRS`
  - 実装: `codex-rs/core/src/custom_prompts.rs:8`
  - テスト: `codex-rs/core/src/custom_prompts.rs:24`
  - 検証: `make verify-additional-prompt-dirs-env`

- `CODEX_SHELL_STARTUP_FILES`
  - 実装: `codex-rs/core/src/shell_startup_files.rs:6`
  - テスト: `codex-rs/core/src/shell_startup_files.rs:17`
  - 検証: `make verify-exec-command-default-login`

- `custom.exec.*`（worker user 固定実行）
  - 実装: `codex-rs/core/src/config/mod.rs:1012`, `codex-rs/core/src/spawn.rs:12`, `codex-rs/core/src/tools/runtimes/mod.rs:13`
  - テスト: `codex-rs/core/src/config/mod.rs:2013`
  - 検証: `make verify-command-exec-worker-user`

- exec-server（PATH 解決 / EscalateRequest.file 絶対化）
  - 実装: `codex-rs/shell-escalation/src/unix/escalate_client.rs:122`, `codex-rs/shell-escalation/src/unix/escalate_server.rs:156`
  - テスト: `codex-rs/shell-escalation/src/unix/escalate_server.rs:315`
  - 検証: `cd codex-rs && cargo test -p codex-exec-server --test all suite::accept_elicitation::accept_elicitation_for_prompt_rule`

- shell snapshot の `exports` マスキング
  - 実装: `codex-rs/core/src/shell_snapshot.rs:244`
  - テスト: `codex-rs/core/src/shell_snapshot.rs:740`
  - 検証: `cd codex-rs && cargo test -p codex-core shell_snapshot`

- テスト出力でのホスト環境変数漏えい抑止
  - 実装: `codex-rs/core/src/shell_snapshot.rs:207`
  - テスト: `codex-rs/core/tests/suite/user_shell_cmd.rs:42`
  - 検証: `cd codex-rs && cargo test -p codex-core user_shell_cmd`

- tool parallelism 判定の安定化
  - 実装: `codex-rs/core/tests/suite/tool_parallelism.rs:80`
  - テスト: `codex-rs/core/tests/suite/tool_parallelism.rs:154`
  - 検証: `cd codex-rs && cargo test -p codex-core tool_parallelism`

- exec-server テスト向け dotslash 同梱
  - 実装: `docker/Dockerfile:28`
  - テスト: `なし`
  - 検証: `make verify-command-exec-worker-user`

- `make verify-*` の既定 `CODEX_HOME` 固定
  - 実装: `Makefile:28`
  - テスト: `なし`
  - 検証: `make verify-all-custom`

- `make test-*` / `make verify-*` ログ保存
  - 実装: `Makefile:187`, `Makefile:191`
  - テスト: `なし`
  - 検証: `make almost`, `make verify-all-custom`

- rustfmt 実行方針（`make fmt`）
  - 実装: `Makefile:131`
  - テスト: `なし`
  - 検証: `make fmt`

- `NOTICE` 著作権追記
  - 実装: `NOTICE:3`
  - テスト: `なし`
  - 検証: `NOTICE` 目視

- 既知不安定回避ターゲット `make almost`
  - 実装: `Makefile:187`
  - テスト: `なし`
  - 検証: `make almost`
