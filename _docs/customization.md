# Codex カスタム手順（運用）

このドキュメントは、リポジトリ内で Codex をカスタムしていくための運用メモです。方針の一次情報源は `CUSTOM.md` です。

## 記録ルール

- 方針: `CUSTOM.md`
- 作業メモ/参考資料: `_docs/`（このディレクトリ）
- カスタムの知見: `_docs/custom_notes/{custom-name}/`
- `AGENTS.md`: `CUSTOM.md` を参照する旨のみを追記する（追加ルールは書かない）
- 一時的なタスクリスト: `_worklist/`
- よく使うコマンド: `Makefile`

## セキュリティ運用（コマンド実行）

この fork では、AI（モデル）が起動するコマンド実行（`shell` / `shell_command` / `exec_command`）を OS の worker ユーザー（例: `assistant`）に固定できる（`custom.exec.*`）。

- 目的: invoker の `HOME` / `CODEX_HOME`（ログイン状態・キャッシュ等）に AI の実行プロセスが触れないようにする。
- 注意: worker 化しても、プロセスに渡した環境変数は `env` / `printenv` で出せるため、環境変数の設計も重要。
  - そのため本 fork では、`custom.exec.*` が有効な場合に `shell_environment_policy.inherit = "all"` をエラーにする（invoker 環境の全量継承を避ける）。

推奨設定例（`config.toml`）:

```toml
[custom.exec]
worker_user = "assistant"

[shell_environment_policy]
inherit = "core"
ignore_default_excludes = false
experimental_use_profile = false
```

詳細は `_docs/custom_notes/command_exec_worker_user/README.md` を参照。

## rebase/merge を楽にするコツ

- 既存ファイルの大規模な整形（reformat）や並べ替えは避ける。
- 可能な限り「追記」で運用する（既存文の改変を最小化）。
- 変更が大きくなりそうなら、まず `_docs/` に背景と意図を記録してから実装する。
  - カスタム固有の知見は `_docs/custom_notes/{custom-name}/` を優先する。

## 追加するときのテンプレ

### 1) 方針の更新が必要な場合

- `CUSTOM.md` に追記する（原則、既存行の編集は避ける）。

### 2) 参考資料を残したい場合

- `_docs/` に Markdown を追加する。
  - 例: `_docs/yyyymmdd-<topic>.md`
  - 例: `_docs/decisions/<topic>.md`（ディレクトリを増やす場合も「追加」で）
  - 一時的なタスクリストは `_worklist/` に置く。
