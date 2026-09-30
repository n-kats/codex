# 20260921 rebase custom report

## range-diff

`fork-origin/main` を共通祖先とした比較は次の1件だけで、rebase 前の `tmp-rebase` と custom commit の内容は一致している。

```text
1: 083bdc72c1 = 1: 083bdc72c1 custom changes
```

## custom 仕様の保持確認

- `--agents-md`／`/custom-agents`
  - 実装: `codex-rs/core/src/config/mod.rs`, `codex-rs/core/src/agents_md_manager.rs`, `codex-rs/core/src/session/`, `codex-rs/app-server-protocol/src/protocol/v2/thread.rs`, `codex-rs/tui/src/chatwidget/slash_dispatch.rs`
  - テスト: `codex-rs/tui/src/chatwidget/tests/slash_commands_custom_tests.rs`, config／core custom tests
  - 検証: experimental schema fixture test は `1 passed`; compile は成功
- config／home／memory／shell startup の custom flags
  - 実装: `codex-rs/cli/src/main.rs`, `codex-rs/config/`, `codex-rs/core/src/config/`, `codex-rs/utils/cli/`
  - テスト: `custom_tests.rs` と各専用 custom test ファイル
  - 検証: `cargo check` 成功
- `custom.user_shell.no_inject` と shell snapshot exports redaction
  - 実装: `codex-rs/core/src/config/custom/`, `codex-rs/core/src/tasks/user_shell.rs`, `codex-rs/core/src/shell_snapshot.rs`
  - テスト: `*_custom_tests.rs`、既存 shell snapshot tests
  - 検証: 今回の conflict resolution 後の `cargo check` 成功
- MCP 完了待機／code-mode 待機
  - 実装: `codex-rs/core/src/tools/handlers/mcp.rs`, `codex-rs/core/src/tools/registry.rs`, `codex-rs/core/src/tools/code_mode/`, `codex-rs/code-mode-runtime/`
  - テスト: core／code-mode custom tests
  - 検証: `cargo check` 成功
- custom theme／version suffix／PSP routing
  - 実装: `codex-rs/config/src/custom/`, `codex-rs/tui/src/diff_render.rs`, `codex-rs/tui/src/update_versions.rs`, `codex-rs/core/src/config/mod.rs`, CLI／exec／TUI の config path
  - テスト: corresponding custom tests
  - 検証: `cargo check` と fmt check 成功。PSP cookie 条件は `Config` に保持
- Cloud Tasks 除去
  - 実装: CLI surface と workspace active members／通常依存から除去。`codex-rs/cloud-tasks*/` の source、Cargo、BUILD は upstream 追従用に保持。`cli/src/cloud_config.rs` の参照用コードも保持
  - テスト: `codex-rs/cli/src/custom_tests.rs` の `custom__cloud_tasks_command__is_removed_from_cli`
  - 検証: workspace manifest の Cloud Tasks members は無効化され、今回 cloud CLI 専用 skip 6件は維持
- Docker／テスト運用
  - 実装: `docker/Dockerfile`, `_mcp/exec_mcp/docker/Dockerfile`, `Makefile`, test lists
  - 検証: 2つの Dockerfile は Ubuntu 26.04、GStreamer development packages、`plugins-base`／`plugins-good` runtime packages を同じ構成で持つ。fmt は成功

## 本家追従で戻したもの／戻さなかったもの

- 戻した: upstream の current config layer API、tool-result metadata、daemon／permission profile routing、filesystem probe、current TUI snapshots、app-server cloud tests。
- 戻さなかった: upstream で廃止された `windows_sandbox_level` 引数、旧 custom Windows propagation helper、`codex cloud` command／Cloud Tasks 通常依存、現行 UI と異なる旧 snapshots。
- `project_doc_paths`、PSP、custom agent-center／worktree assertions は維持した。

## 検証状態

- 成功: cargo check、cargo fmt check、experimental schema selected test、app-server all-features の cloud-config/auth selected tests。
- 未完了: MCP `make almost`。2026-09-21 実行は fmt のみ成功し、Linux sandbox build と test-almost は target mount の `No space left on device` で停止した。よってこのレポートは green／rebase 完了を意味しない。
- Git metadata: `.git` が read-only のため `git add`／`git rebase --continue` は未実行。作業ツリーの競合マーカーは除去済みだが、Git の unmerged stage は残っている。

