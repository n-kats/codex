# Rebase conflict resolution — 2026-08-31

- File: `codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst`
  - Line: generated binary fixture
  - Resolution: 手動マージ
  - Note: upstream の `runtimeStatus` と `GetAccountRateLimitsResponse` の更新を保持し、custom の `ThreadSettingsUpdateParams.projectDocPaths` だけを再適用して圧縮生成物を更新した。

- File: `codex-rs/core/tests/suite/compact.rs`
  - Line: 590
  - Resolution: 手動マージ
  - Note: upstream の `Result` とエラー伝播を維持し、custom の `wait_for_compact_warning` による不要な警告の除外を組み合わせた。

- コンフリクトマーカーを除去した。`git add` と `git rebase --continue` は実行していない。

## Validation

- MCP の compact 対象テスト 4 件がパスした。
- MCP の app-server protocol experimental schema fixture テストがパスした。
- MCP の `make almost` 相当がパスした。
