# make test-almost: known failure skip list

## 目的

- `make test-almost`（=`cargo test` から既知の不安定/前提不足テストを除外）を継続的に回せる状態に保つ。

## 変更内容

- `Makefile` の `SKIP_ALMOST_TESTS` に `cargo test -- --skip ...` 用のテスト名（部分一致）を列挙する。

## 対象範囲

- `Makefile` の `SKIP_ALMOST_TESTS` と `test-almost` ターゲット。

## 注意点（環境差・既知の制約）

- `--skip` は「部分一致」なので、意図せず別テストにマッチしないよう名前はできるだけ fully-qualified に寄せる。
- ここに追加するのは「常に失敗する」か「環境前提が揃わないと必ず落ちる」類に限定し、単なる flaky は原因修正を優先する。

## 動作確認手順

- `make test-almost` を実行し、対象テストが `--skip` されていることを確認する。

## つまずきと対処

- 追加したのにスキップされない場合:
  - `cargo test -q -- --list` で実際のテスト名を確認し、`SKIP_ALMOST_TESTS` の文字列を合わせる。

## 最近の追加（例）

- `suite::codex_tool::test_shell_command_approval_triggers_elicitation`
  - bwrap（bubblewrap）環境での検査/観測の限界により、approval→elicitation のトリガが安定して再現できないため `test-almost` から除外。
- `system_bwrap_warning_skips_supported_system_bwrap`
  - fake bwrap の実行可否が実行環境の noexec / 実行権限の影響を受けやすく、supported 判定の安定性が環境依存になるため `test-almost` から除外。
- `exec-server/tests/file_system.rs` の bwrap 依存ケース群
  - `sandboxed_file_system_helper_finds_bwrap_on_preserved_path`
  - `file_system_sandboxed_read_allows_readable_root`
  - `file_system_sandboxed_write_rejects_unwritable_path`
  - `file_system_sandboxed_write_allows_explicit_alias_roots`
  - `file_system_sandboxed_write_allows_additional_write_root`
  - `file_system_sandboxed_read_rejects_symlink_escape`
  - `file_system_sandboxed_read_rejects_symlink_parent_dotdot_escape`
  - `file_system_sandboxed_write_rejects_symlink_escape`
  - `file_system_create_directory_rejects_symlink_escape`
  - `file_system_read_directory_rejects_symlink_escape`
  - `file_system_copy_rejects_symlink_escape_destination`
  - `file_system_remove_removes_symlink_not_target`
  - `file_system_copy_preserves_symlink_source`
  - `file_system_remove_rejects_symlink_escape`
  - `file_system_copy_rejects_symlink_escape_source`
  - `make almost` では環境依存の sandbox 前提を避けるため除外する。
- `chatwidget::tests::approval_modal_exec_no_reason`
- `chatwidget::tests::approval_modal_exec`
- `chatwidget::tests::chatwidget_markdown_code_blocks_vt100_snapshot`
- `chatwidget::tests::compact_queues_user_messages_snapshot`
- `chatwidget::tests::review_queues_user_messages_snapshot`
- `history_cell::tests::user_history_cell_wraps_and_prefixes_each_line_snapshot`
  - `insta` の snapshot legacy format 更新が必要なため、`test-almost` では別途更新手順に切り出す。

## 関連ファイル一覧

- `Makefile`
- `codex-rs/mcp-server/tests/suite/codex_tool.rs`
