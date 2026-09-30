# Codex カスタム手順（運用）

このドキュメントは、リポジトリ内で Codex をカスタムしていくための運用メモです。方針の一次情報源は `CUSTOM.md` です。

## 記録ルール

- 方針: `CUSTOM.md`
- 作業メモ/参考資料: `_docs/`（このディレクトリ）
- カスタムの知見: `_docs/custom_notes/{custom-name}/`
- `AGENTS.md`: `CUSTOM.md` を参照する旨のみを追記する（追加ルールは書かない）
- 一時的なタスクリスト: `_worklist/`
- よく使うコマンド: `Makefile`

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
