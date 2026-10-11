# 20260905 rebase follow-up

## 対象

- upstream base: `refs/remotes/fork-origin/main` (`3921a30d6b11`)
- custom side: `custom` / `tmp-rebase` (`e2c2c0f10f85`)
- 制約: `git add` と rebase の continue は実行しない。テスト・ビルド確認は MCP 経由で行う。

## コンフリクト解消

- app-server protocol の生成済み experimental schema は upstream の runtime/API と `LegacyAppPathString` を採用し、custom の `projectDocPaths` だけを再適用した。
- CLI の cloud config、doctor、main は upstream の config builder、診断 timing、remote exec validation を残し、custom の loader override と cloud 無効化を維持した。
- core config tests は upstream の session-picker fixture と custom の theme tests を両立した。
- tool registry は upstream の cached `SandboxTags` に追従し、custom の async telemetry と runtime/MCP completion wait hooks を接続した。
- exec は upstream の worktree validation と custom の bootstrap environment を両立し、cloud feature の有無を分岐した。
- TUI の status surface／terminal title は upstream の local settings 構造へ追従し、custom の project-root／決定的 cwd fixture を残した。

## 追加修正

- `user_shell.no_inject` は既定値 `false` では warning を出さず、明示的な `false` だけ warning とした。
- custom で cloud config bundle を除外しているため、cloud-managed filesystem を要求する doctor テストを `skip_test_list.txt` に追加した。
- full workspace の並列実行時だけ失敗し、個別 MCP 実行で成功した fd mount／OpenAI file upload の2テストを `flaky_test_list.txt` に追加した。
- 追加の全体実行で個別成功を確認した reconnect／exec-server network policy／network approval の3テストも、全体並列時の失敗として `flaky_test_list.txt` に追加した。
- `mcp_optional_startup_grace` の runtime configuration refresh も全体並列時だけ初期 startup grace の通知期限を取り逃がした。個別 MCP 実行は成功したため `flaky_test_list.txt` に追加した。
- 2つの safety buffering テストが同一 helper の inline snapshot を共有するため、`insta::allow_duplicates!` を追加した。
- テスト実行で生成された doctor の pending snapshot は、仕様変更を受け入れるスナップショットではないため削除した。

## 検証結果

- MCP cargo fmt check: 成功
- MCP selected tests: app-server protocol、CLI、core、exec、TUI、guardian、safety buffering: 成功
- MCP `run_make_almost_equivalent`: flaky 追加後の最終実行で fmt、Linux sandbox build、test-almost の全 step が成功（終了コード 0）。

作業ツリーの競合マーカーは除去済み。ただし `git add` を行っていないため、`git ls-files -u` では9パスが未ステージ競合として表示される。これはユーザー指定の操作制約によるもので、rebase continue は未実行。
