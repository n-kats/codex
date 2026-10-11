# 20260926 rebase custom

## 対象

- upstream base: `fork-origin/main` / `e72da2b53805894878023d01949a25a082e0a5cb`
- custom side: `tmp-rebase` / `eb4b9745b8f713e275df8642b4824b6f1d7d72cf`
- rebase state: `custom changes` を upstream に適用中

## 競合解消

- `codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst`
  - upstream の圧縮生成物を基礎にし、custom の `ThreadSettingsUpdateParams.projectDocPaths` を TypeScript 1箇所、JSON 2箇所へ再適用した。
  - custom 側の古い生成物に含まれない upstream の executor bearer token、thread item anchor/cursor、realtime reasoning status などは保持した。
- `codex-rs/app-server/src/config_manager.rs`
  - upstream の cloud bundle endpoint network policy を保持し、custom の cloud 無効ビルド用 default loader を接合した。
- `codex-rs/cli/src/exec_server_command.rs`
  - custom 側に存在しなかった upstream の exec-server 新規実装を採用した。
- `codex-rs/cli/src/snapshots/codex__exec_server_args_tests__exec_server_help_documents_remote_options.snap`
  - upstream の PID namespace／WebSocket auth／現行 option 表示を保持し、custom の config/home/agents-md option 表示を維持した。重複した enable/disable 表示は1組に整理した。
- `codex-rs/cli/tests/doctor_path_safety.rs`
  - custom の ETXTBSY 回避（実行ファイルを常時コピー）を維持し、upstream の hard-link／Rosetta 分岐は戻さなかった。
- `codex-rs/code-mode-runtime/src/service.rs`
  - upstream の caller preemption signal を保持し、custom の `u64::MAX` による MCP 完了待機（`UntilCompletion`）を組み合わせた。
- `codex-rs/code-mode/src/remote_session/connection.rs`
  - upstream の Windows `CREATE_NO_WINDOW` を保持し、custom の `ExecutableFileBusy` retry を接合した。
- `codex-rs/config/src/loader/mod.rs`
  - upstream の application loader と custom test module を両方保持した。
- `codex-rs/core/src/shell_snapshot.rs`
  - upstream の `shell_environment_policy` 経路を保持し、custom の exports redaction を snapshot 保存前に適用した。
- `codex-rs/core/src/tools/code_mode/execute_handler.rs`
  - upstream の indirect namespace 処理と custom の MCP 待機用 yield time 上書きを両方保持した。
- `codex-rs/core/tests/suite/mcp_optional_startup_grace.rs`
  - upstream の現行 `TURN_TIMEOUT` を採用し、古い2秒待機変更は戻さなかった。
- `codex-rs/exec-server/src/shell_snapshot.rs`
  - upstream の unnamed file replay／snapshot capture を保持し、custom の Bash profile／rc 読み込み抑止を新しい restore 経路へ接合した。
- `codex-rs/exec/src/lib.rs`
  - upstream の embedded network policy／worktree 経路を保持し、custom の config file/profile override と cloud 無効 loader を維持した。custom の PSP／project doc path も維持した。
- `codex-rs/thread-manager-sample/src/main.rs`
  - upstream の auth-derived URL／proxy 設定を採用し、custom の PSP 初期値を保持した。
- `codex-rs/tui/src/chatwidget/tests/helpers.rs`
  - upstream の一時 Codex home を保持する test helper を採用し、custom の deterministic cwd helper を追加で保持した。
- `codex-rs/tui/src/lib.rs`、`codex-rs/tui/src/onboarding/auth.rs`、`codex-rs/tui/src/session_archive_commands.rs`
  - upstream の embedded network policy と cloud loader 引数を保持し、custom の cloud 無効 build 用 default loader を cfg 分離した。

## 確認状態

- Rust／snapshot の競合マーカーは作業ツリーから除去した。
- `projectDocPaths` と upstream-only schema export の両方を圧縮 fixture 内で確認した。
- `.git` が read-only のため unmerged index の解消（`git add`／`git rebase --continue`）は未実行。
- `support/executable.rs` は `exec_server.rs` と `app_server_daemon.rs` から参照されているため削除せず、競合解消で発生した未使用 import だけを除去した。
- 対象ファイルの conflict marker scan と `git diff --check` は成功した（workspace 内の意図的な denied fixture は Git の権限エラー対象外として扱う）。

## 20260927 追加確認

- `codex-rs/linux-sandbox/src/bwrap.rs`
  - upstream の proc マウントを維持しつつ、custom の `mount_proc = false` fallback と共存するよう、procfs が必要な path mask の直前に `--proc /proc` を置く処理へ整理した。
  - `/dev/fd` は `--dev /dev` が作る絶対 symlink のため、bwrap の `/newroot` 構築中にその path 自体へ `tmpfs` を張れない。拒否対象だけ `/proc/self/fd` に写像して同じ隔離結果を得るようにした。skip は追加していない。
  - `shell_snapshot::shell_snapshot_concurrent_replays_keep_independent_readers::blocked_descriptor_path` は `1 passed`、linux-sandbox の bwrap 関連 filter は `44 passed`。
- `codex-rs/otel/tests/suite/{mod.rs,buffered_operations_tests.rs,otlp_http_loopback.rs}`
  - `OtelProvider` 等のプロセスグローバル状態を操作する既存 integration test 群だけを mutex で直列化した。製品コードの動作変更・skip はない。
  - `codex-otel` suite は `41 passed; 0 failed`。
- bwrap の argv／stderr と exec-server の一時 `RUST_LOG` 出力は診断後に除去した。
- `codex-rs/cli/tests/mcp_add_remove.rs` は、子プロセスがホストの `RUST_BACKTRACE` を継承して inline snapshot に backtrace を混入させないよう、テスト用 Codex command に `RUST_BACKTRACE=0` を明示した。実装のエラー内容は変更していない。対象テストは `1 passed`。
- MCP の `run_make_almost_equivalent` は再起動後に完走し、fmt、linux-sandbox build、`test-almost` の全ステップが `ok: true`／exit code 0 だった。core の最終集計は `5526 passed; 0 failed; 4 ignored`。
- snapshot の legacy-format warning は出たが、失敗や `.snap.new` の生成はなかった。

## 20260928 追加確認

- `suite::managed_proxy::proc_mount_denial_preserves_legacy_fallback_and_explicit_pid_inheritance` の失敗ログは、bwrap の proc 拒否結果ではなく、テストが sandbox helper を起動する `Command::output()` で `ENOENT` を受けたものだった。
- 対象テストの単独実行を3回、linux-sandbox の suite 63件（60 passed / 3 ignored）、および `run_make_almost_equivalent` を再実行した。全て成功し、最終 `test-almost` でも対象テストは実行され `... ... ok` になった。
- したがって、今回の1件を flaky/skip リストへ追加せず、bwrap 実装や custom 機能には追加変更を行わなかった。ログに出る snapshot の legacy-format warning は今回の失敗原因ではない。

## 20260930 追加確認

- upstream `ab84d71f5767e4a565ce81c2c426287cb48c7918` への再リベースで、次の6競合を解消した。
  - app-server schema fixture: upstream の生成内容を基礎にし、custom の `projectDocPaths` だけを再適用した。upstream-only の bearer token、anchor/cursor、realtime 状態は保持した。
  - selected capability stack: upstream が削除した古い同一ターン capability-root テストと helper は削除し、custom の動的 exec-server URL と selected capability 検証は保持した。
  - exec-server command: upstream の `ExecServerRuntimeOptions` と private-IP proxy option を採用し、custom の cloud-free loader、loader/CLI override、config/home 経路は保持した。cloud loader を再導入していない。
  - doctor path safety: upstream の hard-link/Rosetta 分岐は戻さず、custom の常時 byte-copy による ETXTBSY 回避を保持した。
  - exec-server help snapshot: upstream の現行 option 順と PID/WebSocket auth 表示を採用し、custom の config/home/memory/agents-md/no-config-file 表示を保持した。
  - realtime conversation: upstream の現行接続処理を取り込み、custom の V1 設定、startup/realtime fixture 分離、header/prewarm fixture と明示的 shutdown を保持した。
- `codex-otel` の process-global state 用 mutex は custom 要素として保持した。再リベース後の `almost` で `rejected_opt_out_preserves_recording_and_accepted_opt_out_survives_shutdown` だけが全体実行時に一度余分な buffered operation を観測したが、fully-qualified 実行、buffered 2件、`codex-otel` suite 全42件を通過した。本体変更ではなく、rebase 手順の基準に従って当該テストだけを `flaky_test_list.txt` に追加した。
- Enterprise policy reload テストは削除せず、app-server cloud feature を無効にする通常 `almost` だけ `skip_test_list.txt` で除外した。同じ2件は `--all-features` で `2 passed` だった。cloud 機能とテスト自体は保持している。
- 最終の MCP `run_make_almost_equivalent` は `ok: true`／exit code 0。`codex-core` の集計は `5592 passed; 0 failed; 4 ignored`。fmt、Linux sandbox build、test-almost の全段階も成功した。
- 競合対象の conflict marker と `git diff --check` は問題なし。`.git` が read-only のため、index の `git add`、`git rebase --continue`、および確定済み ref を使う形式の range-diff 保存は実行できない。代わりに upstream HEAD と作業ツリー、旧 custom commit の対象差分を目視確認した。

## 20260930 追加確認（managed-proxy / OTEL）

- `suite::managed_proxy::proc_mount_denial_preserves_legacy_fallback_and_explicit_pid_inheritance` の再失敗は、proc fallback の判定失敗ではなく、テスト helper の `Command::output()` が `ENOENT` になったものだった。Linux sandbox を明示ビルドして単独実行すると `1 passed`、managed-proxy 群は `15 passed; 0 failed` だった。
- crate 全体の再実行で別の `suite::managed_proxy::unsupported_system_bwrap_falls_back_to_bundled_bwrap` が、Cargo の実行ファイルを hard-link する fixture による `ETXTBSY` で失敗した。custom の doctor fixture と同じ byte-copy + permissions 方式へ変更し、linux-sandbox suite は `60 passed; 0 failed; 3 ignored` になった。
- `codex-otel` の buffered 2件は単独・suite 全体（`42 passed; 0 failed`）では通る一方、workspace 全体の test binary 並列実行時だけ global metrics が混入した。`flaky_test_list.txt` に2件を完全修飾名で追加した。skip ではなく、対象テスト自体は通常どおり直接実行可能な扱いとした。
- fixture 修正と flaky 判定後の MCP `run_make_almost_equivalent` は `ok: true`、exit code 0。最終ログに `test result: FAILED` と `TEST SUMMARY: FAILED` はなく、codex-core 集計は `5592 passed; 0 failed; 4 ignored` だった。

## 20260930 追加確認（realtime prompt override）

- `suite::realtime_conversation::conversation_uses_experimental_realtime_ws_backend_prompt_override` の再失敗は、realtime 本体ではなく、rebase 後に Responses の prewarm 用接続と realtime 用接続を同じ WebSocket fixture に戻していたことが原因だった。prewarm 側が realtime 用の `session.updated` を消費し、Responses 層で未処理イベントになっていた。
- upstream の現行 fixture と同じく startup 用 `startup_server` と realtime 用 `server` を分離し、custom の `experimental_realtime_ws_backend_prompt` 検証、prompt override の assertion、明示的 shutdown は維持した。実装側の realtime 処理は変更していない。
- 整形チェック、対象テスト（`1 passed`）、realtime suite（`64 passed; 0 failed`）、および修正後の MCP `run_make_almost_equivalent`（`ok: true`／exit code 0）が成功した。このテストを skip/flaky リストへ追加していない。

## 20261001 追加確認（managed-proxy 実行ファイル解決）

- `suite::managed_proxy::proc_mount_denial_preserves_legacy_fallback_and_explicit_pid_inheritance` の全体実行時 `ENOENT` は、proc fallback の実装失敗ではなく、同テストだけが `env!("CARGO_BIN_EXE_codex-linux-sandbox")` を使い、custom 側で既に導入済みの実行ファイル探索 helper を使っていなかったことが原因だった。
- 既存の `codex_linux_sandbox_exe()` に揃えた。skip、flaky 登録、製品コードの変更は行っていない。
- fmt、対象テスト（`1 passed`）、managed-proxy suite（`15 passed; 0 failed; 1 ignored`）、修正後の `run_make_almost_equivalent`（`ok: true`／exit code 0）を確認した。全体ログでも対象テストは `... ok` になっている。
- 追加で linux-sandbox 周辺の `CARGO_BIN_EXE_*` 利用を監査した。`bundled_bwrap.rs` に残っていた Bazel 専用の compile-time `CARGO_BIN_EXE_codex-linux-sandbox` も `codex_utils_cargo_bin::cargo_bin()` へ統一し、同じ固定パス問題を残していない。検索範囲に該当する固定利用はない。
- 変更後の bundled-bwrap fixture test は `1 passed`、最終 `almost` も `ok: true`／exit code 0 だった。
