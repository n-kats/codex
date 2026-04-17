# TUI2: 送信キー変更に伴うテスト修正メモ

## 注記（このノートは履歴です）

このリポジトリには過去に `codex-rs/tui2` が存在していましたが、現在は削除されています。
そのため、このノートは **当時の背景メモとして残しているだけ**で、現行の実装・テストの更新対象ではありません。

現行の入力キー仕様（`Enter`=改行、`Ctrl+Enter`/`Ctrl+J`=送信）は `codex-rs/tui` 側の実装を参照してください:

- `_docs/custom_notes/tui-enter-newline-ctrl-enter-send/README.md`
- `codex-rs/tui/src/bottom_pane/chat_composer.rs`

## 目的

- TUI2 の入力仕様（Enter=改行、Ctrl+Enter/Ctrl+J=送信）に合わせて、テストが誤ったキー操作前提にならないようにする。
- `/prompts:<name>`（カスタムプロンプト）において、引数が必要なテンプレートと不要なテンプレートで挙動が分かれることを明文化する。

## 背景（今回落ちたテスト）

- 送信キーを Ctrl+Enter（または Ctrl+J）に寄せた結果、`Enter` を送信として扱っていたテストが失敗した。
- `/prompts:<name>` を引数なしで submit した時に「テンプレを挿入すべきケース」と「そのまま submit すべきケース」が混在しており、仕様が曖昧になっていた。

## 仕様（TUI2）

- `Enter`:
  - 通常は改行を挿入する（送信はしない）。
  - Slash コマンド候補ポップアップ表示中は、選択中の候補を確定する用途に使われる。
- `Ctrl+Enter` / `Ctrl+J`:
  - 送信（submit）扱い。
  - タスク実行中は「キューに積む」挙動になる（ChatWidget 側）。

## カスタムプロンプト（`/prompts:<name>`）の引数なし submit

引数なしで submit されたときの扱いは、プロンプト本文が「引数を期待するか」で分岐する。

- 引数を期待するテンプレート（テンプレ挿入する / submit しない）
  - named placeholders: `$USER` のような `$[A-Z][A-Z0-9_]*` を含む
  - numeric placeholders: `$1..$9` または `$ARGUMENTS` を含む
  - → `"/prompts:<name> "` を入力欄に挿入して、ユーザーが引数を続けて入力できるようにする
- 引数を期待しないテンプレート（そのまま submit する）
  - 上記の placeholders を含まない
  - → テンプレ本文を即時 submit する

## 今回の修正内容（要点）

- `ChatComposer::set_text_content()` がカーソルを先頭に戻してしまい、`Enter` で先頭に改行が入る状態になっていたため、カーソルを末尾に置くように修正。
- `Ctrl+Enter` submit 時に、`/prompts:<name>`（引数なし）が常にテンプレ挿入にならないように条件を厳密化（上記「引数なし submit」の仕様に合わせる）。
- TUI2 の `ChatWidget` 系テストで、送信操作を `Enter` ではなく `Ctrl+Enter` に寄せた。

## 動作確認（手元）

- `cargo test -p codex-tui2 --lib`
- もしくは repo の手順に合わせて `make almost`（フォーマット + 主要テスト）。

## 関連ファイル

- `codex-rs/tui2/src/bottom_pane/chat_composer.rs`
- `codex-rs/tui2/src/chatwidget/tests.rs`
- `README.md` / `CUSTOM.md`（入力仕様の記述）
