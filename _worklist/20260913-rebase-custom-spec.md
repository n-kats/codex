# 20260913 rebase custom spec report

本家追従を優先して 2026-09-13 の rebase 競合を解消した結果を、現行 custom 要件ごとに確認する。

| custom 要件 | 現行実装 | テスト／確認 |
| --- | --- | --- |
| `--agents-md`／`/custom-agents` による project docs 指定 | `core/src/config/mod.rs`, `core/src/agents_md_manager.rs`, `core/src/session/*`, `app-server-protocol/src/protocol/v2/thread.rs`, `tui/src/chatwidget/slash_dispatch.rs` | `core` custom tests、TUI custom slash tests、実験 schema fixture |
| `custom.user_shell.no_inject` | `core/src/config/custom/user_shell.rs`, `core/src/tasks/user_shell.rs` の注入・履歴境界 | core custom tests 3件、warning integration test |
| Shell snapshot の exports 秘匿 | `core/src/shell_snapshot.rs` | `shell_snapshot_custom_tests.rs` 2件、exec-server `local_pipe` |
| MCP ツール完了待機 | `core/src/tools/handlers/mcp.rs`, `core/src/tools/registry.rs`, `core/src/tools/code_mode/execute_handler.rs`, `code-mode-runtime/src/service.rs` | code-mode-runtime／core custom tests、既存 MCP wait tests |
| 差分 theme (`custom.theme.diff`) | `config/src/custom/theme.rs`, `core/src/config/mod.rs`, `tui/src/diff_render.rs`, `tui/src/lib.rs` | config／TUI の custom tests、TUI startup／snapshot selected tests |
| custom version suffix | `tui/src/update_versions.rs` | `update_versions_custom_tests.rs` |
| PSP routing | `core/src/config/mod.rs`, CLI／exec／TUI config path | CLI／exec selected tests、workspace compile 接合部 |
| test／Docker 実行環境の再現性 | `docker/Dockerfile`, `_mcp/exec_mcp/docker/Dockerfile`, custom test files | fmt、selected tests。glib は Docker 側開発パッケージ追加で補完 |

## 20260914 skip 外の失敗の修正

- `codex-rs/tui/src/app_server_session/rollout_history_tests.rs` の既存テストで、起動時 background
  migration と resume の maintenance lock 取得順を固定値 `+2` としていたため、並列実行時に正当な
  `Paginated/+3` 結果を失敗扱いしていた。lock によって直列化された `Legacy/+2`／`Paginated/+3` の
  二つの結果だけを検証するよう最小限修正した。
- skip／flaky list には追加していない。codex-tui 全テストと `almost` 相当を再実行し、いずれも失敗
  なしを確認した。

## 本家側を維持した項目

- 現行 app-server の plugin disabled IDs、provider-based AGENTS manager、delegate／step-context
  API、rich shell profile test、Windows startup 警告、schema の user-verification 更新。
- upstream が削除した `codex mcp-server` command／crate と、現行 custom 方針にない Friendly／
  Pragmatic personality popup、`SandboxReadRoot` 分岐は復元していない。

## 不要な過去維持を削除した項目

- caller のない runtime-cancellation hook と `dispatch_any` wrapper。
- 現行 upstream API と合わない AgentsMdManager test の旧呼び出し。
- loader の upstream fixture に重複していた custom-only 設定差分。
- 未設定時は静かに扱う現行 `custom.user_shell.no_inject` と矛盾する、upstream 通常 test に残った
  旧 warning filter／定数。
- upstream の通常 test ファイルに混在していた custom-only test。現行の
  `*_custom_tests.rs`（および `custom_tests.rs`）へ移し、`custom__...` 命名に統一した。
- `thread_start.rs` のコメントアウト済み import と無条件 skip は整理し、Cloud 設定エラー test
  自体は本家追従のため `feature = "cloud"` 条件付きで保持。

## 残作業

- Git metadata が書き込み可能な環境で、解消済み 19 パスと追加した `*_custom_tests.rs` を
  `git add` して `git rebase --continue`。
- range-diff は `_tmp/range-diff/20260913-rebase-custom.txt` に保存済み。ただし現環境では
  upstream HEAD と元 custom commit の比較に留まるため、最終 rebase 後に同じコマンドを再実行する。
- 必要なら完全 workspace test を実行する。

## 20260915 追加検証

- PTY startup、descriptor reuse、provider model refresh の全体並列 failure を、focused／crate-wide 実行と
  照合して分類した。機能を除外する `skip_test_list.txt` ではなく、根拠のある exact flaky entries だけを
  `flaky_test_list.txt` に追加した。
- daybreak test は skip にせず、production の3秒 timeout を維持したまま test wait の deadline を10秒へ
  広げた。`codex-tui --lib` 全4,649件が成功した。
- MCP `run_make_almost_equivalent` は fmt／Linux sandbox build／test-almost の全工程で `exit_code: 0`。
