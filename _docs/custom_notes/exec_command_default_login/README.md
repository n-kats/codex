# `exec_command` の login と初期化ファイル読み込みの制御

## 目的

- `exec_command` を login shell（`-lc`）で動かすか（または `-c`）は、用途により使い分けたい。
- さらに、login shell にした場合に「ユーザーの dotfiles（`~/.zprofile` など）」を読むかどうかも、テスト/CI では固定したい。

## 変更内容

- `exec_command` の `login` は従来どおり（未指定なら login）を維持する。
- dotfiles の読み込みは `CODEX_SHELL_STARTUP_FILES` で制御できるようにした。
  - `CODEX_SHELL_STARTUP_FILES=default`: 通常の挙動（ユーザー dotfiles を読む可能性がある）
  - `CODEX_SHELL_STARTUP_FILES=clean`: 可能な範囲でユーザー dotfiles を読まない（現状は zsh を `ZDOTDIR` で隔離）

## 影響範囲

- `exec_command`（および共有されるシェル実行経路）。

## 動作確認（手元環境で実行）

- `make verify-exec-command-default-login`
  - `CODEX_SHELL_STARTUP_FILES=clean` で `exec_command` 系の動作確認を行う（テストは Makefile 側で `clean` を付与して実行する）。

## 関連ファイル

- `codex-rs/core/src/tools/handlers/unified_exec.rs`
- `codex-rs/core/src/tools/spec.rs`
- `codex-rs/core/src/shell_startup_files.rs`
- `Makefile`
