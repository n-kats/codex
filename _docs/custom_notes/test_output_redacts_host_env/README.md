# テスト/ログ出力でホスト環境変数を全量表示しない

## 目的

テスト失敗時の `assert_eq!` などで、親プロセス（ホスト）の環境変数が大量に差分表示されると、秘匿情報やホスト固有情報がログに混入するリスクがある。  
このリポジトリでは、ログをそのまま共有できる状態（安全に貼り付けられる状態）を維持する。

## 背景（何が起きたか）

- `shell` / `unified_exec` などの実行時には `ShellEnvironmentPolicy` に従って環境変数を組み立てる。
- 既定では `inherit = All` のため、テストで `env` を丸ごと `assert_eq!` すると、失敗時に **環境変数の値まで** 差分として表示され得る。

## 方針

- テストでは `env` の **値を丸ごと `assert_eq!` しない**。
  - 代わりに、主に次を検証する:
    - キー集合（`BTreeSet<String>` 等）で期待どおりのキーが含まれること
    - 変更で追加したキーなど、必要最小限のキーのみ値を比較する（安全な値のみ）
- どうしても `env` 全体の一致を検証したい場合は、入力側（テスト用の `vars`）を固定してホスト依存を排除する。

## 対象例

- `codex-rs/core/src/tools/handlers/shell.rs` の `ShellCommandHandler::to_exec_params` テスト:
  - `exec_params.env` の比較を「キー集合＋安全なキーの値比較」に変更した。

## 関連ファイル

- `codex-rs/core/src/tools/handlers/shell.rs`

