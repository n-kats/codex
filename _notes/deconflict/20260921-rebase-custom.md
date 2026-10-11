# 20260921-rebase-custom

## 対象

- upstream base: `fork-origin/main` / `7d99ee82d74325cabf485ea1e2adbd0c2625ab19`
- custom side: `custom` / `083bdc72c137321d4ceacfea0e26a5d2670228aa`
- 比較用: `tmp-rebase` / `083bdc72c137321d4ceacfea0e26a5d2670228aa`
- `custom changes` 1コミットを upstream に適用中。作業ツリー上の競合マーカーは除去済みだが、`.git` が read-only のため `git add` と `git rebase --continue` は未実行。

## 競合解消

- `codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst`
  - upstream の実験 API schema を基準に再生成相当の fixture を作り、custom の `ThreadSettingsUpdateParams.projectDocPaths` だけを再適用した。
  - `schema_fixtures_tests::experimental_precomputed_exports_match_generated` は all-features なしで `1 passed`。
- `codex-rs/app-server/tests/suite/v2/thread_start.rs`
  - upstream の `thread/start`／`model/list` の parameterized cloud-config test を採用し、`feature = "cloud"` 条件だけを付けて保持した。
  - custom の通常ビルドでは cloud feature を無効にし、all-features では本家 test を実行できる構成にした。
- `codex-rs/cli/src/doctor.rs`
  - upstream の filesystem probe、typed config error、filesystem check を採用し、custom の `LoaderOverrides` と既存の config override 経路を接合した。
- `codex-rs/cli/tests/worktree.rs`
  - upstream の worktree／daemon 検証を保持し、custom CLI で cloud feature が無効なため cloud-managed source recheck だけは戻していない。
- `codex-rs/core/src/config/mod.rs`
  - upstream の現行 config layer rebuild、system proxy fallback、Windows sandbox config を採用し、custom の `psp`／`project_doc_paths` と PSP cookie 条件を保持した。
- `codex-rs/core/src/tools/parallel.rs`
  - upstream の tool-result metadata／executed-call recording を採用し、custom の tool timing と MCP 待機経路を保持した。
- `codex-rs/exec-server/tests/file_system/shared.rs`
  - upstream の filesystem API／Arc 変更と custom の Linux bwrap 用 `Command` を接合した。
- `codex-rs/tui/src/app/agents_overview_tests.rs`
  - upstream の現行 server-version notice assertion と snapshots を採用し、custom の agent-center test と path normalization を保持した。
- `codex-rs/tui/src/app/config_persistence.rs`
  - upstream の permission profile／Windows sandbox 更新経路と custom の agents overview profile 状態を保持した。
  - upstream API で廃止された `windows_sandbox_level` 引数を戻さず、同じ引数を渡していた未使用の custom helper は削除した。
- `codex-rs/tui/src/app/event_dispatch.rs`
  - upstream の現行 event routing と `select_permission_profile` を採用し、custom の daemon／agent overview／project-doc 経路を保持した。
- `codex-rs/tui/src/chatwidget/snapshots/*approval_modal_patch.snap`
  - upstream の snapshot path normalization と現行表示を採用した。
- `codex-rs/tui/src/chatwidget/snapshots/*terminal_title_setup_popup_live_only.snap`
  - upstream の current live preview UI を採用した。旧 custom snapshot の表示文言は現行 UI と一致しないため戻していない。
- `codex-rs/tui/src/chatwidget/tests/terminal_title.rs`
  - upstream の title-effects／spinner assertion を採用し、custom の deterministic cwd setup と title behavior assertion を保持した。
- `codex-rs/tui/src/cli.rs`
  - upstream の daemon executable／no-daemon option と custom の PSP routing field を両方保持した。

## 競合外の整理

- `codex-rs/app-server/tests/suite/auth.rs` の proactive refresh test は、`codex cloud` command ではない app-server auth test なので `cfg(any())` を解除した。
- `codex-rs/app-server/tests/suite/v2/thread_fork.rs` と `thread_resume.rs` の cloud-config bundle test は削除せず、imports と test 本体を `feature = "cloud"` 条件付きで復元した。
- 上記3テストに対応する不要な `skip_test_list.txt` の3行を削除した。Cloud Tasks CLI 専用6テストの skip は維持した。
- `flaky_test_list.txt` と `skip_test_list.txt` に重複していた `sandbox_fetches_and_enforces_cloud_managed_permission_profile` は、Cloud managed 機能を通常 CLI から除外する skip 側だけに残した。
- `/custom-agents` の `OverrideTurnContext` 呼び出しに残っていた upstream で廃止済みの `windows_sandbox_level` 引数を削除した。custom の `project_doc_paths` は保持した。

## 検証結果

- MCP cargo check: 成功。
- MCP cargo fmt check: 成功。stable rustfmt の `imports_granularity = Item` 警告のみ。
- app-server protocol experimental schema test: `1 passed`。
- app-server all-features selected tests: thread/fork、thread/start の2ケース、thread/resume、proactive auth refresh が成功。
- codex-tui selected tests: MCP target の容量不足（`No space left on device`）でコンパイル開始前に停止。テスト assertion の failure ではない。
- 全体 `almost` は、この作業ツリーについてまだ成功確認できていない。既存の `_tmp/almost_test_result.txt` には本家由来の network approval failure が残っているため、現時点で green と扱わない。
- 2026-09-21 の MCP `make almost` 相当を再実行した結果は `exit_code: 101`。fmt は成功したが、`build-linux-sandbox` と `test-almost` は MCP の target mount が `No space left on device`／`database or disk is full` で成果物を書けず、テスト assertion まで到達しなかった。
- range-diff:
  `git range-diff b04a2c264516ec2e6b3c91dd73ad18a21fd5a88f..tmp-rebase b04a2c264516ec2e6b3c91dd73ad18a21fd5a88f..custom`
  は `083bdc72c1 = 083bdc72c1 custom changes` で、rebase 前後の custom commit 内容に差はないことを確認した。
