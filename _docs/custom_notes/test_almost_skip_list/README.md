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

## 関連ファイル一覧

- `Makefile`
- `codex-rs/mcp-server/tests/suite/codex_tool.rs`
