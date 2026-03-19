# シェル初期化ファイルの制御（zsh dotfiles を隔離する）

## 背景（問題）

- Linux でユーザーのログインシェルが zsh の場合、`zsh -lc ...` の起動が以下の理由で遅くなったり停止したりすることがある。
  - `.zprofile` / `.zshenv` などの初期化で重い処理をしている
  - 対話入力待ち（例: キーチェーン/ssh-agent/補完初期化など）に入る
  - 初期化ファイルが `exit 1` などで失敗し、コマンド実行まで到達しない
- Codex の `shell_command` / `exec_command` / `!` のユーザーコマンドは、短い `timeout_ms` での完了や、テストでの安定実行が期待されるため、利用者の dotfiles に依存すると不安定になりやすい。

## 目標

- **デフォルトは「ユーザーのシェル」を使い続ける**（例: `/usr/bin/zsh`）。
- ただし、dotfiles の読み込みは **設定で制御できる** ようにする（テストは `clean`、実運用は `default` など）。
- 「非 login をデフォルトにする」や「bash を強制する」ではなく、必要なときに切り替えられるようにする。

## 変更内容

### `CODEX_SHELL_STARTUP_FILES=clean`

- `CODEX_SHELL_STARTUP_FILES=clean` を指定すると、起動時のユーザー dotfiles を**可能な範囲で**読み込まないようにする。
- 現状は zsh に対して実装しており、子プロセスの環境変数 `ZDOTDIR` を空ディレクトリに向けることで、`zsh -l`（login）自体は維持しつつ、ユーザーの `~/.z*` を隔離する。

### 対象

- `shell_command`（モデルが呼ぶツール）
- `exec_command`（PTY 実行）
- `!` のユーザーコマンド

## 動作確認（手元環境で実行）

- `make test-all`
  - `Makefile` の test/verify ターゲットは `CODEX_SHELL_STARTUP_FILES=clean` を付与して実行するため、dotfiles に依存したタイムアウト/失敗が減る。
- ログは `_tmp/test_all_test_result.txt` を確認する（`tee` で保存される）。

## 関連ファイル

- `codex-rs/core/src/shell_startup_files.rs`
- `codex-rs/core/src/tools/handlers/shell.rs`
- `codex-rs/core/src/unified_exec/session_manager.rs`
- `codex-rs/core/src/tasks/user_shell.rs`
- `Makefile`
