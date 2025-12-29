# TUI: Enter で改行 / Ctrl+Enter で送信

## 目的

- 長文入力のたびに誤送信しやすい問題を避けるため、`Enter` は改行、`Ctrl+Enter` を送信にする。

## 変更内容（概要）

- 送信キーを `Enter` → `Ctrl+Enter` に変更した。
- フッターのショートカット表示を更新した（`Enter: newline` / `Ctrl+Enter: send`）。
- 送信に関わるテスト入力を `Ctrl+Enter` に更新した（UI スナップショット更新が必要になる場合あり）。

## 対象範囲

- 対象: 入力欄の「送信」操作（通常のメッセージ送信、`/` コマンドの dispatch、`/prompts:*` の実行）。
- 非対象: ポップアップ表示中の `Enter`（候補の選択/確定などの UI 操作）は従来どおり。

## 注意点

- 端末によっては `Ctrl+Enter` を区別できない場合がある（環境依存）。その場合は送信できない可能性があるため、実端末で動作確認する。
- ポップアップ（候補選択など）の `Enter` は従来どおり選択に使う（改行にはならないことがある）。
- 実装後に `dead_code` 警告が出た場合は、使われなくなったフラグ/分岐（例: 以前の `Shift+Enter` ヒント）を削除し、テスト・スナップショットも合わせて更新する。

## 実装メモ

- `Ctrl+Enter` を `KeyCode::Enter` + `KeyModifiers::CONTROL` として扱う前提で実装している（端末/入力バックエンドによっては届かない可能性がある）。
- フッターの「改行」表示は `Enter` に統一し、「送信」を `Ctrl+Enter` として表示する。

## 動作確認（別環境で実行）

- フォーマット:
  - `make fmt`（nightly rustfmt 推奨。stable だと一部設定が無視される場合あり）
- テスト:
  - `make test-tui`
- スナップショット差分が出た場合:
  - `make insta-pending-tui`
  - `make insta-accept-tui`（意図した差分のみ受理する）

## 実際に試すこと

- `Enter` で改行され、送信されないこと。
- `Ctrl+Enter` で送信されること。
- `/di` → `Tab` で `/diff ` 補完後に `Ctrl+Enter` でコマンドが実行されること。
- `/` でコマンド候補が出ている状態で `Enter` が候補選択/確定として動くこと。

## つまずきと対処（テスト/スナップショット/警告）

- 送信キー変更後、`Enter` を使っていたテストが `None` になって落ちる場合がある → 送信を意図するテスト入力は `Ctrl+Enter` に更新する。
- フッター表示が変わるため、`insta` のスナップショットが差分になる → 差分を確認して意図どおりなら受理する。
- `dead_code` 警告（旧 `Shift+Enter` ヒント等の残骸）が出た → 未使用の enum variant / フィールド / 分岐を削除して解消する。

## 関連ファイル（今回の変更箇所）

- 入力処理: `codex-rs/tui/src/bottom_pane/chat_composer.rs`
- フッター表示: `codex-rs/tui/src/bottom_pane/footer.rs`
- 公開ウィジェットの説明: `codex-rs/tui/src/public_widgets/composer_input.rs`
- テスト（必要に応じて）: `codex-rs/tui/src/chatwidget/tests.rs`
- スナップショット: `codex-rs/tui/src/bottom_pane/snapshots/`
