# 20260325 rebase custom report

## Summary

- Base rebase: `fork-origin/main`
- Comparison branch: `tmp-rebase`
- Follow-up focus: `SandboxType` の型不整合を、`be19390d2f` への丸ごと復元ではなく通常編集で最小修正する

## Itemized changes

- `codex-rs/core/src/exec.rs`
  - 実装: `codex_sandboxing::SandboxType` と `crate::exec::SandboxType` の相互変換 `From` を追加
  - 目的: rebase 後に sandbox 型をまたぐ変換を明示できるようにする

- `codex-rs/core/src/sandboxing/mod.rs`
  - 実装: platform sandbox 選択で `codex_sandboxing::get_platform_sandbox(..).map(Into::into)` を使うように修正
  - 目的: `exec::SandboxType` 側の値に統一する

- `codex-rs/core/src/sandboxing/mod_tests.rs`
  - 実装: 参照元を公開関数へ変更し、`get_platform_sandbox(..)` の戻り値を `Into::into` で `exec::SandboxType` に寄せる
  - テスト: `transform_additional_permissions_enable_network_for_external_sandbox`
  - テスト: `transform_additional_permissions_preserves_denied_entries`

- `codex-rs/core/src/exec_tests.rs`
  - 実装: test 側の `SandboxType` import を `crate::exec::SandboxType` に変更し、platform sandbox の比較値を `Into::into` で合わせる
  - テスト: `windows_restricted_token_skips_external_sandbox_policies`

- `codex-rs/core/src/tools/runtimes/shell/unix_escalation_tests.rs`
  - 実装: test 側の `SandboxType` import を `crate::exec::SandboxType` に変更
  - テスト: `map_exec_result_preserves_stdout_and_stderr`

- `codex-rs/app-server/src/command_exec.rs`
  - 実装: `SandboxType` import を `codex_core::exec::SandboxType` に統一

- `codex-rs/core/src/tasks/user_shell.rs`
  - 実装: `SandboxType` import を `crate::exec::SandboxType` に統一

- `codex-rs/core/src/tools/orchestrator.rs`
  - 実装: `SandboxType` / `SandboxManager` を core 側に統一

- `codex-rs/core/src/unified_exec/process.rs`
  - 実装: `SandboxType` import を `crate::exec::SandboxType` に統一

- `codex-rs/sandboxing/src/manager_tests.rs`
  - 実装: sandboxing crate 側のテストを upstream API に合わせて維持

## Verification

- MCP: `make_almost_equivalent`
  - `fmt`: passed
  - `build-linux-sandbox`: passed
  - `test-almost`: failed because the broader suite still contains unrelated `SandboxType` mismatch errors in other `codex-core` tests

- MCP selected tests:
  - `codex-core` / `transform_additional_permissions`:
    - `sandboxing::tests::transform_additional_permissions_enable_network_for_external_sandbox` passed
    - `sandboxing::tests::transform_additional_permissions_preserves_denied_entries` passed
  - `codex-core` / `windows_restricted_token_skips_external_sandbox_policies` passed
  - `codex-core` / `map_exec_result_preserves_stdout_and_stderr` passed

## Notes

- `be19390d2f` は参照元として使ったが、以後の修正は `apply_patch` による通常編集へ切り替えた
- `test-almost` の残り失敗は、今回の follow-up とは別に `codex-core` の広範な `SandboxType` 追従が必要な箇所に起因する
