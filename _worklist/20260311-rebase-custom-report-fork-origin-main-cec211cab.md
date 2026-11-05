# 20260311 rebase custom report (fork-origin/main cec211cab)

## Summary

- Base: `fork-origin/main` @ `cec211cabc158532459b0c522a0cf855a891bd40`
- Branch: `custom` @ `7820b8470adb6f0fc4d82e5d6917f41b0fd9f136`
- Comparison branch: `tmp-rebase` @ `0821c449d8053701d0425c976c31c35bcbca7c38`
- Range-diff log: `_tmp/range-diff/20260311-rebase-range-diff.fork-origin-main-cec211cab.custom-7820b8470.txt`
- Deconflict log: `_notes/deconflict/20260311-rebase-custom.md`

この rebase は `fork-origin/main` の更新（`56420da8...` → `cec211ca...`）を取り込むために実施した。`tmp-rebase`（rebase 前に squash した比較用）と、rebase 後の `custom`（`custom changes`）の間にはコンフリクト解消に伴う差分がある（`range-diff` で `!`）。

## Range-diff review

- Reviewed:
  - `_tmp/range-diff/20260311-rebase-range-diff.fork-origin-main-cec211cab.custom-7820b8470.txt`
- Summary:
  - `tmp-rebase` の `0821c449d custom changes` は、rebase 後 `custom` の `7820b8470 custom changes` と比較して `!`（内容変更あり）
  - 変更の主因はコンフリクト解消で、詳細は `_notes/deconflict/20260311-rebase-custom.md` に記録した
  - 主な差分（抜粋）:
    - `.devcontainer/Dockerfile` の build deps 統一（`cmake` / `libseccomp-dev` 追加など）
    - `codex login` の file-backed tracing と `--config` / `--no-config` 系の統合
    - `core/config.schema.json` の `default_permissions` と `custom` の両立
    - `spawn_child_async` / `SpawnChildRequest` の `run_as` / `sandbox_policy` 取り回し差分
    - `codex-rs/otel` の `traces` 廃止に追従（`events` 側へ）
- Conclusion:
  - 目視レビュー対象は `!` 箇所（このファイルの Summary で列挙）で、意図した解消であることを確認する

## Custom report

カスタム仕様の項目別レポートは、前回作成分を参照（必要に応じて更新する）:

- `_worklist/2026-03-06-rebase-custom-report-fork-origin-main-56420da8.md`

## Verification

この環境ではテストは実行しない（`CUSTOM.md` の方針）。手元環境で次を実行して確認する:

- `make all`（フォーマット + 全テスト）
- `cd codex-rs && cargo test -p codex-tui`
  - `cargo insta pending-snapshots -p codex-tui` で差分確認（UI 変更が入っているため）

## Scope note

- `range-diff` は `tmp-rebase` と `custom` の比較であり、未コミット変更は含まれない

