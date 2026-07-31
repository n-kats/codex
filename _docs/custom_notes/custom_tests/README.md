# custom_tests

## 目的

- フォーク由来の `custom` 変更を、上流（`fork-origin/main`）の更新に強い形でテストできるようにする。
- rebase 時の衝突を増やさず、意図した `custom` 仕様が欠損していないことを素早く検出できるようにする。

## 変更内容（何がどう変わるか）

- 上流に存在しないテストは「custom 専用テスト」として分離し、`custom` を含むファイル名にのみ追加する。
- テスト名は `custom__...` を接頭辞にし、絞り込み実行（例: `cargo test custom__`）を前提にする。
- テスト名の形式は `custom__機能名__テスト内容` を基本とし、機能名は日本語/英語どちらでも「何のカスタムか」を優先して命名する。

## 対象範囲（非対象も）

- 対象
  - `fork-origin/main` に無い（上流に存在しない）テスト全般
  - 上流追従の rebase で壊れやすいカスタム仕様の回帰テスト
- 非対象
  - 上流由来の既存テストそのものの大規模な改変（必要最小限に留める）

## 注意点（環境差・既知の制約）

- 上流の既存テストファイルに新規の `#[test]` / `#[tokio::test]` を追加しない。
  - 理由: rebase の衝突増加、上流との差分レビュー難化を招くため。
- 「custom 専用テスト」はファイル名に `custom` を含める。
  - 例: `custom_tests.rs` / `custom_*.rs` / `*_custom_*.rs`
- テスト名は `custom__...` に統一する。
  - 例: `custom__custom_agents__slash_custom_agentsでsessionのproject_doc_pathsへ反映する`
  - 例: `custom__差分テーマ色__色指定を全体無効化できる`
- 日本語名を使う場合は `non_snake_case` 警告が出るため、custom テストファイル先頭に `#![allow(non_snake_case)]` を付けて運用する。

## 置き場（パターン）

- `src/...` の近くに `custom_tests.rs` を作って `#[cfg(test)] mod custom_tests;` でぶら下げる。
  - 例: `codex-rs/tui/src/diff_render/custom_tests.rs`
- 統合テスト（`core/tests/suite/...`）の場合は、`custom_*.rs` として別ファイルに切り出す。
  - 例: `codex-rs/core/tests/suite/custom_user_shell_cmd.rs`

## 実行方法（手元環境）

- custom テストだけ流したい場合（例）:
  - `cd codex-rs && cargo test custom__`
- crate を絞りたい場合（例）:
  - `cd codex-rs && cargo test -p codex-core custom__`
  - `cd codex-rs && cargo test -p codex-tui custom__`
- 一覧だけ確認したい場合（例）:
  - `make list-custom-tests`

## 既存 custom テスト（例）

- `codex-rs/core/src/codex/custom_tests.rs`
  - `custom__custom_agents__override_turn_context_project_doc_paths変更時にuser_instructionsを再生成する`
- `codex-rs/tui/src/chatwidget/custom_tests.rs`
  - `custom__custom_agents__slash_custom_agents_*`
- `codex-rs/tui/src/diff_render/custom_tests.rs`
  - `custom__差分テーマ色__*`

## custom 追加テスト棚卸し

- `custom` ブランチで追加されたテストと custom 機能の対応関係は `custom_added_tests_inventory.md` に記録する。
- 再実装時は、機能ごとの実装対象/再実装対象外候補をこの棚卸しから確認する。

## つまずきと対処（2026-02-27）

- 症状
  - `make almost` の `core` 統合テストで `suite::compact_resume_fork::compact_resume_and_fork_preserve_model_history_view` が `timeout waiting for event` で失敗。
  - 同時にログに `Shell snapshot validation failed: Failed to execute bash (os error 2)` が出る。
- 原因
  - `codex-rs/core/tests/suite/compact_resume_fork.rs` の `start_test_conversation()` が `TestCodex.cwd`（`Arc<TempDir>`）を返しておらず、関数終了時に `cwd` が削除される。
  - その後の shell snapshot 検証で `Command::current_dir(cwd)` が存在しないディレクトリを参照し、`os error 2` になる。
- 対処
  - `start_test_conversation()` の戻り値に `Arc<TempDir>`（`cwd` ガード）を追加し、呼び出し側で `_cwd` として保持する。
  - 対象: `compact_resume_and_fork_preserve_model_history_view` / `compact_resume_after_second_compaction_preserves_history`。
- 確認観点（手元環境で実行）
  - `cd codex-rs && cargo test -p codex-core --test all suite::compact_resume_fork::compact_resume_and_fork_preserve_model_history_view`
  - `cd codex-rs && cargo test -p codex-core --test all suite::compact_resume_fork::compact_resume_after_second_compaction_preserves_history`
  - shell snapshot の `Failed to execute bash (os error 2)` が再発しないことを確認する。

## 関連ファイル一覧

- `_docs/custom_notes/rebase_rules/README.md`
- `_docs/custom_notes/custom_tests/custom_added_tests_inventory.md`
- `CUSTOM.md`
- `codex-rs/core/tests/suite/compact_resume_fork.rs`
