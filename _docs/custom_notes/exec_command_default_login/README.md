# `exec_command` の login と初期化ファイル読み込みの制御

## 目的

- `exec_command` を login shell（`-lc`）で動かすか（または `-c`）は、用途により使い分けたい。
- さらに、login shell にした場合に「ユーザーの dotfiles（`~/.zprofile` など）」を読むかどうかも、テスト/CI では固定したい。

## 変更内容

- `exec_command` の `login` は従来どおり（未指定なら login）を維持する。
- dotfiles の読み込みは `CODEX_SHELL_STARTUP_FILES` で制御できるようにした。
  - `CODEX_SHELL_STARTUP_FILES=default`: 通常の挙動（ユーザー dotfiles を読む可能性がある）
  - `CODEX_SHELL_STARTUP_FILES=clean`: 可能な範囲でユーザー dotfiles を読まない（現状は zsh を `ZDOTDIR` で隔離）
- `--shell-startup-files <MODE>` も同じ設定を起動前に反映する CLI フラグとして追加した。

## 影響範囲

- `exec_command`（および共有されるシェル実行経路）。
- `shell_command`
- `shell_snapshot`
- `!` のユーザーコマンド

## 動作確認（手元環境で実行）

- `make verify-exec-command-default-login`
  - `command_execution_notifications_include_process_id` で exec の統合経路を確認する。
  - `shell_tests::derive_exec_args` で login shell の引数生成を確認する。
  - `custom__シェル起動ファイル__` で `CODEX_SHELL_STARTUP_FILES=clean` の挙動を確認する。
  - `custom__shell_startup_files_cli_flag__` で CLI フラグの受け口を確認する。

## 関連ファイル

- `codex-rs/core/src/tools/handlers/unified_exec.rs`
- `codex-rs/core/src/tools/spec.rs`
- `codex-rs/core/src/unified_exec/process_manager.rs`
- `codex-rs/core/src/shell_startup_files.rs`
- `codex-rs/core/src/shell_snapshot.rs`
- `codex-rs/core/src/tasks/user_shell.rs`
- `codex-rs/cli/src/main.rs`
- `Makefile`
