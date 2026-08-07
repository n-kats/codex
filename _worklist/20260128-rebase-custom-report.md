# 2026-01-28 Rebase Custom Report

このファイルは `custom` ブランチの `fork-origin/main` への rebase 後に、フォークで維持しているカスタム仕様の実装箇所・テスト/検証手順をざっと確認するためのレポートです。

- `--codex-home` / `CODEX_HOME` 上書き
  - 実装: `codex-rs/arg0/src/lib.rs:114`
  - テスト: `codex-rs/arg0/src/lib.rs:335`
  - 検証: `Makefile:189`（`make verify-codex-home-cli-flag`）

- `CODEX_ADDITIONAL_PROMPT_DIRS`（追加プロンプト探索パス）
  - 実装: `codex-rs/core/src/custom_prompts.rs:8`
  - テスト: `codex-rs/core/src/custom_prompts.rs:281`
  - 検証: `Makefile:200`（`make verify-additional-prompt-dirs-env`）

- TUI 入力キー: Enter=改行 / Ctrl+Enter or Ctrl+J=送信
  - 実装: `codex-rs/tui/src/public_widgets/composer_input.rs:5`
  - テスト: `codex-rs/tui/src/chatwidget/tests.rs:288`
  - 検証: `Makefile:192`（`make verify-tui-enter-newline-ctrl-enter-send`）

- `custom.exec.worker_user`（コマンド実行の Run-As）
  - 実装: `codex-rs/core/src/config/mod.rs:1039`
  - テスト: `codex-rs/core/src/config/mod.rs:1221`
  - 検証: `Makefile:213`（`make verify-command-exec-worker-user`）

- `CODEX_SHELL_STARTUP_FILES`（シェル起動ファイル適用の制御）
  - 実装: `codex-rs/core/src/shell_startup_files.rs:6`
  - テスト: `codex-rs/core/src/shell_startup_files.rs:73`
  - 検証: `cargo test -p codex-core`（ローカル）

- Shell snapshot の `exports` の許可リスト化（秘匿情報混入の抑制）
  - 実装: `codex-rs/core/src/shell_snapshot.rs:168`
  - テスト: `codex-rs/core/src/shell_snapshot.rs:729`
  - 検証: `cargo test -p codex-core`（ローカル）

