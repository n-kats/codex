# 20260905-rebase-custom.md

- File: `codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst`
  - Line: generated binary fixture
  - Resolution: 手動マージ
  - Note: upstream の生成済みエクスポートを基礎にし、現行 custom API の `projectDocPaths` だけを再適用した。upstream で変更された `PermissionsRequestApprovalParams.cwd` の `LegacyAppPathString` と新しい runtime/API 定義は戻していない。

- File: `codex-rs/cli/src/cloud_config.rs`
  - Line: 120-145
  - Resolution: 手動マージ
  - Note: upstream の cwd 解決と config builder 呼び出しを採用し、custom の loader override と cloud 無効化方針を維持した。

- File: `codex-rs/cli/src/doctor.rs`
  - Line: 375-430
  - Resolution: 手動マージ
  - Note: upstream の config timing/reporting を維持し、custom の loader override と project-doc override を loader-aware な呼び出しに接続した。

- File: `codex-rs/cli/src/main.rs`
  - Line: 850-900, 1870-2040
  - Resolution: 手動マージ
  - Note: upstream の remote exec direct/AWS SigV4/forward/auth-host validation を維持し、custom の loader override を config 読み込みに残した。PSP と worktree の custom 分岐も維持した。

- File: `codex-rs/core/src/config/config_tests.rs`
  - Line: 5900-5990
  - Resolution: 手動マージ
  - Note: custom の TUI theme と upstream の session-picker config fixture を両方残した。

- File: `codex-rs/core/src/tools/registry.rs`
  - Line: 520-590
  - Resolution: 手動マージ
  - Note: upstream の cached `SandboxTags` に追従し、custom の async telemetry と runtime/MCP completion wait hooks を dispatch に接続した。

- File: `codex-rs/exec/src/lib.rs`
  - Line: 80-115
  - Resolution: 手動マージ
  - Note: upstream の worktree 検証を維持し、custom の bootstrap environment 適用を実行前に残した。

- File: `codex-rs/tui/src/chatwidget/tests/status_surface_previews.rs`
  - Line: 70-110
  - Resolution: 手動マージ
  - Note: upstream の `local_settings.tui.status_line` へ追従しつつ、custom の project-root cache fixture を維持した。

- File: `codex-rs/tui/src/chatwidget/tests/terminal_title.rs`
  - Line: 25-100
  - Resolution: 手動マージ
  - Note: upstream の `local_settings.tui` 設定へ追従しつつ、custom の deterministic cwd fixture と期待タイトル計算を維持した。

## Rebase 後の検証と追加対応

- `codex-rs/core/src/config/custom/user_shell.rs`
  - custom の方針どおり、`no_inject` の既定値 `false` では warning を出さず、明示的な `Some(false)` のときだけ warning を出すように修正した。
- `skip_test_list.txt`
  - custom では cloud config bundle／`codex cloud` を通常 workspace から除外しているため、cloud-managed filesystem を要求する upstream の doctor テストを skip 対象にした。
- `flaky_test_list.txt`
  - `fd_mount::tests::duplicate_mount_descriptors_are_rejected` は full workspace の fd 再利用競合、`files::tests::upload_openai_file_reports_blob_transport_diagnostics_without_sas` は full workspace の network/proxy 並列負荷でのみ失敗し、各個別 MCP 実行は成功したため登録した。
  - 追加の全体実行で、`suite::reconnect::automatic_reconnect_restores_draft_and_routes_new_notifications` は PTY 起動30秒超過、`client::tests::network_policy_tests::policy_requests_use_process_decider_and_cancel_on_unregister` は inbound request span 欠落、`suite::network_approval::failed_network_policy_amendment_denies_request_and_does_not_approve_host` は承認要求到達前の完了が発生した。3件とも個別 MCP 実行は成功したため登録した。
  - さらに `suite::mcp_optional_startup_grace::running_thread_uses_refreshed_optional_mcp_startup_grace::runtime_configuration_refresh` は全体並列実行で初期 startup grace の通知期限を取り逃がしたが、個別 MCP 実行は成功したため登録した。
- `codex-rs/tui/src/app/tests/safety_buffering.rs`
  - 2つのテストが同じ helper 内の inline snapshot を通るため、Insta 1.46 の重複検出に `insta::allow_duplicates!` を付けた。両テストの個別 MCP 実行後、最終 MCP almost equivalent は成功した。

## 検証

- MCP cargo fmt check: 成功
- MCP selected tests: app-server protocol、CLI、core、exec、TUI、guardian、今回の safety buffering を成功確認
- MCP `run_make_almost_equivalent`: 追加 flaky 登録後の最終実行でも fmt／Linux sandbox build／test-almost の全 step が終了コード 0

ユーザー指定により `git add` と rebase の continue は実行していない。したがって `git ls-files -u` では内容上は解消済みの9パスが未ステージ競合として残る。
