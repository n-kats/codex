# 20260926 rebase custom report

## 対象

- upstream base: `fork-origin/main` / `e72da2b53805894878023d01949a25a082e0a5cb`
- rebase 前の custom: `tmp-rebase` / `eb4b9745b8f713e275df8642b4824b6f1d7d72cf`
- 現在: `custom changes` を適用中

## 本家追従と custom 維持

- `projectDocPaths`、PSP、config/home/memory/agents-md の custom 経路は維持した。
- cloud feature 無効ビルドでは default loader を使い、cloud feature 有効時は upstream の network policy と cloud loader を使うよう cfg を分離した。
- MCP 完了待機の `UntilCompletion`、shell snapshot の export redaction、shell snapshot の startup file 抑止、ExecutableFileBusy retry は維持した。
- upstream の unnamed snapshot replay、current exec-server command、network policy、Windows の `CREATE_NO_WINDOW`、現行 schema export は採用した。
- upstream で更新された MCP startup grace の期限、doctor の現行 snapshot 名、schema の新しい anchor/cursor export は custom 側の古い内容で上書きしていない。
- 既存の `skip_test_list.txt`／`flaky_test_list.txt` は変更していない。今回の競合解消を理由に skip を追加していない。

## custom 要素の担保箇所

- MCP/code-mode 完了待機: `codex-rs/core/src/tools/code_mode/execute_handler.rs`, `codex-rs/code-mode-runtime/src/service.rs`; `*_custom_tests.rs`。
- shell snapshot export redaction: `codex-rs/core/src/shell_snapshot.rs`; `shell_snapshot_custom_tests.rs`。
- config/home/memory/agents-md/project document paths/PSP: `codex-rs/cli`, `codex-rs/core`, `codex-rs/config`, `codex-rs/tui`, `codex-rs/exec` の既存 custom tests。
- TUI custom diff theme: `codex-rs/tui/src/diff_render.rs` と既存 custom tests。
- custom test helper: `codex-rs/tui/src/chatwidget/tests/helpers.rs` とその利用テスト。

## 確認状態

- 競合対象の Rust／snapshot に conflict marker は残っていない。
- schema 圧縮 fixture は upstream の key 集合を維持し、`projectDocPaths` だけを再適用した。
- `support/executable.rs` は別の upstream test から参照されているため保持した。
- `.git` が read-only のため、実 index への `git add` と `git rebase --continue` は未実行。

## 追加の失敗切り分けと最終検証

- `exec-server` の blocked descriptor 失敗は実行環境の user namespace 不可による skip 対象ではなかった。bwrap が `/dev/fd`（`/proc/self/fd` への絶対 symlink）へ setup 中に `tmpfs` を張ろうとして失敗していたため、sandbox 引数生成側で実体 path を mask するよう修正した。新しい skip は追加していない。
- `codex-otel` の2件は単体では通る一方、並列実行時に process-global OTEL state を競合させていた。関連する既存 stateful tests を共有 mutex で直列化し、`codex-otel` suite は `41 passed` になった。
- 一時診断用の bwrap argv、stderr、`RUST_LOG` は除去済み。
- `codex-rs/cli/tests/mcp_add_remove.rs` は、ホストの `RUST_BACKTRACE` 継承で inline snapshot に backtrace が混入する問題だけをテスト環境側で固定した。`mcp` の実装・エラーメッセージは変更していない。対象テストは `1 passed`。
- MCP `run_make_almost_equivalent` の最終結果は fmt、linux-sandbox build、`test-almost` の全て `ok: true`／exit code 0。core は `5526 passed; 0 failed; 4 ignored`。
- snapshot の legacy-format warning は残るがテストは成功しており、意図的な UI 変更に伴う snapshot 更新ではないため受理していない。

## 20260928 追加検証

- `suite::managed_proxy::proc_mount_denial_preserves_legacy_fallback_and_explicit_pid_inheritance` の一度の失敗は、bwrap の proc fallback 検証失敗ではなく、sandbox helper 起動時の `Command::output()` が `ENOENT` を返したものだった。
- 個別3回、linux-sandbox suite 全体（60 passed / 3 ignored）、`make almost` 相当を再実行し、全て成功した。`test-almost` では対象テストも skip されず成功している。
- 再現性のない実行環境側の一時的な起動失敗と判断し、skip追加・テスト改変・product code変更は行っていない。
