# _docs/custom_notes

カスタマイズごとの知見（背景・設計・注意点・検証手順など）を置く場所です。

## 置き方

- カスタム名ごとにディレクトリを作る: `_docs/custom_notes/{custom-name}/`
- その中に `README.md` を作り、知見を十分詳しく記録する。

## 最低限含める項目（テンプレ）

- 目的
- 変更内容（何がどう変わるか）
- 対象範囲（非対象も）
- 注意点（環境差・既知の制約）
- 動作確認手順（手動・テスト・スナップショット）
- つまずきと対処（警告や失敗の修正）
- 関連ファイル一覧

## 動作確認の共通ルール

- `make` 経由の動作確認は、デフォルトで `CODEX_HOME=<リポジトリ配下>/_cache/codex_home_debug` を使う（`Makefile` で指定）。

## まず辿る導線

- custom 全体方針: `CUSTOM.md`
- rebase 運用: `_docs/custom_notes/rebase_rules/README.md`
- custom 専用テスト運用: `_docs/custom_notes/custom_tests/README.md`
