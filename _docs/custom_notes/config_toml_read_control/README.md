# config.toml 読み込み制御（`--config` / `--no-config`）

## 目的

- デフォルトのユーザー設定（通常 `$CODEX_HOME/config.toml`）を、任意のファイルへ切り替えられるようにする。
- ユーザー設定＋プロジェクト設定（`.codex/` ツリー）を完全に無視して起動できるようにする（システム設定と `-c key=value` は有効）。

## 変更内容（何がどう変わるか）

- `codex --config <FILE>`:
  - ユーザー設定レイヤーの読み込み元を `<FILE>` に差し替える（相対パスは実行時のカレント基準）。
- `codex --no-config`:
  - ユーザー設定レイヤーとプロジェクト設定レイヤーを読み込まない。

## 対象範囲

- 対象:
  - `codex`（TUI / `exec` / `login` / `logout` / `mcp-server` / `mcp` などのサブコマンドを含む）から `LoaderOverrides` へ反映される経路。
- 非対象:
  - `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` / `CODEX_SANDBOX_ENV_VAR` への変更（本リポジトリ方針により不可）。

## 注意点（環境差・既知の制約）

- `--no-config` は「ユーザー＋プロジェクト」を無視するだけで、システム設定（例: MDM/管理設定）と `-c key=value` は引き続き適用される想定。
- `--config` と `--no-config` は同時指定不可（clap の `conflicts_with` で弾く）。
- `--config` は互換のため `--config-toml-file` も受理する（alias）。

## 動作確認手順（手動・テスト）

この環境（エージェント側）では `cargo` 等が無い前提のため、検証は手元環境で行う。

- 手動（推奨）
  - `codex --help` に `--config <FILE>` / `--no-config` が出ることを確認
  - `codex --config sample_config.toml "hi"` が起動することを確認
  - `codex --no-config "hi"` が起動することを確認
- Rust テスト（任意）
  - `codex-rs` で `cargo test -p codex-cli`（clap parse の単体テストを含む）

## つまずきと対処（警告や失敗の修正）

- `error: unexpected argument '--config' found` が出る
  - `codex-rs/cli` の clap 定義からフラグが消えている／`LoaderOverrides` が TUI 起動に伝播していない可能性が高い。

## 関連ファイル一覧

- `codex-rs/cli/src/main.rs`
- `codex-rs/exec/src/lib.rs`
- `codex-rs/cli/src/mcp_cmd.rs`
- `codex-rs/mcp-server/src/lib.rs`
