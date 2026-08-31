# 2026-02-03 Rebase Custom Report

このファイルは `custom` ブランチの `fork-origin/main` への rebase 後に、フォークで維持しているカスタム仕様の実装箇所・テスト/検証手順をざっと確認するためのレポートです。

- `--codex-home` / `CODEX_HOME` 上書き
  - 実装: `codex-rs/arg0/src/lib.rs:114`
  - テスト: `codex-rs/arg0/src/lib.rs:335`
  - 検証: `Makefile:191`（`make verify-codex-home-cli-flag`）

- `--config` / `--no-config`（config.toml 読み込み制御）
  - 実装: `codex-rs/cli/src/main.rs:72` / `codex-rs/core/src/config/mod.rs:440`
  - テスト: なし
  - 検証: `Makefile:188`（`make verify-all-custom`）

- `CODEX_ADDITIONAL_PROMPT_DIRS`（追加プロンプト探索パス）
  - 実装: `codex-rs/core/src/custom_prompts.rs:8`
  - テスト: `codex-rs/core/src/custom_prompts.rs:281`
  - 検証: `Makefile:202`（`make verify-additional-prompt-dirs-env`）

- TUI 入力キー: Enter=改行 / Ctrl+Enter or Ctrl+J=送信
  - 実装: `codex-rs/tui/src/bottom_pane/chat_composer.rs:2297`
  - テスト: `Makefile:195`（`cargo test -p codex-tui --lib enter_inserts_newline_instead_of_submitting` など）
  - 検証: `Makefile:194`（`make verify-tui-enter-newline-ctrl-enter-send`）

- `custom.exec.worker_user`（コマンド実行の Run-As）
  - 実装: `codex-rs/core/src/config/mod.rs:1072`
  - テスト: `codex-rs/core/src/config/mod.rs:1247`
  - 検証: `Makefile:215`（`make verify-command-exec-worker-user`）

- `CODEX_SHELL_STARTUP_FILES`（シェル起動ファイル適用の制御）
  - 実装: `codex-rs/core/src/shell_startup_files.rs:6`
  - テスト: `codex-rs/core/src/shell_startup_files.rs:72`
  - 検証: `cargo test -p codex-core`（ローカル）

- Shell snapshot の `exports` の許可リスト化（秘匿情報混入の抑制）
  - 実装: `codex-rs/core/src/shell_snapshot.rs:175`
  - テスト: `codex-rs/core/src/shell_snapshot.rs:669`
  - 検証: `Makefile:209`（`make verify-linux-default-shell`）
