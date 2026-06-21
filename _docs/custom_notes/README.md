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
- 上流追従しやすい custom 実装方針: `_docs/custom_notes/implementation_policy.md`
- rebase 運用: `_docs/custom_notes/rebase_rules/README.md`
- custom 専用テスト運用: `_docs/custom_notes/custom_tests/README.md`
- 許可された sudo fallback: `_docs/custom_notes/allowed_sudo_fallback.md`
- 未接続 `tui_app_server` の削除: `_docs/custom_notes/tui_app_server_removal/README.md`
- 標準 TUI の remote 入口整備: `_docs/custom_notes/tui_remote_alignment/README.md`
- exec_mcp の almost 相当: `_docs/custom_notes/exec_mcp_almost_equivalent/README.md`
- exec_mcp の長時間 job 停止と応答性改善: `_docs/custom_notes/exec_mcp_background_stop/README.md`
