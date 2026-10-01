# repository_guidance_upstream_alignment

## 目的

- 本家追従時のリポジトリ内 Codex 指示ファイル削除と、それに伴う旧運用ルールの削除を記録する。

## 変更内容（何がどう変わるか）

- ルート `AGENTS.md` と `CUSTOM_AGENTS.md` を削除した。
- `CUSTOM.md` と `_docs/customization.md` から、`AGENTS.md` に `CUSTOM.md` の参照を追記する旧手順を除いた。
- 削除の経緯は本ノートに記録した。本家の削除コミットは `18131270fe`。

## 対象範囲（非対象も）

- 対象: リポジトリルートの指示ファイルと、旧参照追記手順。
- 非対象: Codex が利用者のプロジェクト文書として扱う `AGENTS.md` の自動探索、`--agents-md`、`/custom-agents` の実装や使用方法。

## 注意点（環境差・既知の制約）

- 本家追従で削除したリポジトリ内ファイルと、利用者が指定するプロジェクト文書機能は別の扱い。

## 動作確認手順（手動・テスト・スナップショット）

- 文書差分であるため、ビルド・テスト・フォーマットは実行しない（`CUSTOM.md` の環境制約に従う）。
- `rg -n 'AGENTS\.md.*CUSTOM\.md|CUSTOM\.md.*AGENTS\.md|AGENTS\.md.*追記' CUSTOM.md _docs/customization.md` で古い参照追記方針が残っていないことを確認する。
- `test ! -e AGENTS.md && test ! -e CUSTOM_AGENTS.md` でルートの2ファイルが存在しないことを確認する。

## つまずきと対処（警告や失敗の修正）

- 削除前の `CUSTOM.md` とカスタマイズ手順書に残っていた旧参照追記手順を除いた。

## 関連ファイル一覧

- `README.md`
- `CUSTOM.md`
- `_docs/customization.md`
- `_docs/custom_notes/README.md`
- `AGENTS.md`（削除）
- `CUSTOM_AGENTS.md`（削除）
