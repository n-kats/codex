# 20260930 rebase custom report

## 対象

- upstream: `fork-origin/main` / `ab84d71f5767e4a565ce81c2c426287cb48c7918`
- rebase 前 custom: `custom` / `699dbcb8470c114d1871e68de9922524ff55c083`
- 現在: upstream HEAD 上で custom changes を適用中

## 本家追従と custom 維持

- upstream の exec-server runtime API、proxy-private option、selected-capability の現行テスト構造、realtime 接続処理、schema export を採用した。
- `projectDocPaths`、cloud-free 通常ビルド、config/home/memory/agents-md、動的 exec-server URL、doctor の ETXTBSY 回避、realtime V1/prewarm fixture は維持した。
- upstream が削除した古い selected-capability 同一ターンテストと helper は残していない。
- app-server の cloud 機能と Enterprise テストは削除していない。通常 feature 構成でだけ必要な1件を `skip_test_list.txt` に置き、`--all-features` で実行できる状態を維持した。

## custom 要素の担保箇所

- `projectDocPaths`: `codex-rs/app-server-protocol/src/protocol/v2/thread.rs:294-297`、schema fixture、app-server protocol schema test。
- cloud-free loader と config override: `codex-rs/cli/src/exec_server_command.rs`、CLI exec-server tests。
- doctor ETXTBSY 回避: `codex-rs/cli/tests/doctor_path_safety.rs:46-52`、doctor tests 6件。
- selected capability の動的 executor: `codex-rs/app-server/tests/suite/v2/selected_capability_stack.rs:567` 付近、selected-capability suite 9件。
- realtime V1/prewarm fixture: `codex-rs/core/tests/suite/realtime_conversation.rs`、realtime suite 64件。
- process-global OTEL test isolation: `codex-rs/otel/tests/suite/mod.rs` と関連既存テストの mutex。

## 検証

- `codex-otel` buffered 2件: `2 passed`。
- `codex-otel` suite: `42 passed; 0 failed`。
- app-server selected capability: `9 passed; 0 failed`。
- core realtime: `64 passed; 0 failed`。
- CLI exec-server option tests: `7 passed; 0 failed`。
- schema fixture test: `1 passed; 0 failed`。
- MCP `run_make_almost_equivalent`: `ok: true`、exit code 0、`codex-core 5592 passed; 0 failed; 4 ignored`。

全体 `almost` でのみ再現した OTEL buffered tests は、focused 実行と crate 全体が通ることを確認したうえで、`flaky_test_list.txt` に当該2件の fully-qualified 名だけを追加した。製品コードの skip や環境変数判定は追加していない。

## 追加確認（managed-proxy / OTEL）

- proc fallback の単独テストは `1 passed`、managed-proxy 群は `15 passed; 0 failed`。失敗時の `ENOENT` は helper 起動時の環境差であり、proc fallback 実装の失敗ではなかった。
- linux-sandbox suite 全体で発見した `ETXTBSY` は、既存 custom の doctor fixture と同じく hard-link を standalone byte-copy に変更して解消した。全体は `60 passed; 0 failed; 3 ignored`。
- `codex-otel` suite は `42 passed; 0 failed`。workspace 並列時だけ global state が混入する buffered 2件は、CUSTOM の基準に従い `flaky_test_list.txt` に完全修飾名で整理した。
- その後の `run_make_almost_equivalent` は `ok: true`／exit code 0。最終ログは `codex-core 5592 passed; 0 failed; 4 ignored`。

`.git` が read-only のため、競合ファイルの index 登録と `git rebase --continue` は未実行。conflict marker scan と対象差分の目視確認は完了している。

## 追加確認（realtime prompt override）

- rebase 後の realtime prompt override テストは、Responses prewarm と realtime の WebSocket fixture が同一 server の接続列になっていたため、prewarm が realtime 用 `session.updated` を消費して timeout していた。
- startup 用／realtime 用の server を upstream 現行構成へ分離した。custom の backend prompt override、V1/prewarm 関連設定、検証は残し、製品実装は変更していない。
- 対象テストは `1 passed`、realtime セクションは `64 passed; 0 failed`。その後の `run_make_almost_equivalent` も `ok: true`／exit code 0 で完了した。今回のテストを skip/flaky リストへ追加していない。

## 20261001 追加確認（managed-proxy 実行ファイル解決）

- `proc_mount_denial_preserves_legacy_fallback_and_explicit_pid_inheritance` の全体実行時 `ENOENT` をログで確認した。原因は proc mount 処理ではなく、テストだけが `CARGO_BIN_EXE` の埋め込みパスを使っていたことだった。
- custom 側の既存 `codex_linux_sandbox_exe()` helper をこのテストでも使うよう統一した。skip/flaky への追加はしていない。
- 対象テストは `1 passed`、managed-proxy は `15 passed; 0 failed; 1 ignored`、修正後の `almost` は `ok: true`／exit code 0。最終ログ上も対象テストは成功している。
- linux-sandbox 周辺に残っていた Bazel 専用 `bundled_bwrap.rs` の compile-time `CARGO_BIN_EXE_codex-linux-sandbox` も `codex_utils_cargo_bin::cargo_bin()` へ変更した。検索範囲の同パターンはゼロである。
- bundled-bwrap fixture test は `1 passed`、追加変更後の `almost` も `ok: true`／exit code 0 だった。
