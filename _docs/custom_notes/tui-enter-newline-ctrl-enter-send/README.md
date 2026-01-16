# TUI: Enter で改行 / Ctrl+Enter で送信

## 目的

- 長文入力のたびに誤送信しやすい問題を避けるため、`Enter` は改行、`Ctrl+Enter` を送信にする。

## 変更内容（概要）

- `Enter` を改行、送信キーを `Ctrl+Enter` にした。
- `Ctrl+Enter` が端末で区別できない/潰れる環境向けに、送信のフォールバックとして `Ctrl+J` も扱うようにした。
- フッターのショートカット表示を更新した（`Enter: newline` / `Ctrl+Enter: send (or Ctrl+J)`）。
- 送信に関わるテスト入力を `Ctrl+Enter` / `Ctrl+J` に更新した（UI スナップショット更新が必要になる場合あり）。
- `/` コマンドの実行も「送信」扱い（`Ctrl+Enter` / `Ctrl+J`）に統一する。

## 対象範囲

- 対象: 入力欄の「送信」操作（通常のメッセージ送信、`/` コマンドの dispatch、`/prompts:*` の実行）。
- 非対象: ポップアップ表示中の `Enter`（候補の選択/確定などの UI 操作）は従来どおり。

## 注意点

- 端末によっては `Ctrl+Enter` を区別できない場合がある（環境依存）。その場合は送信できない可能性があるため、実端末で動作確認する。
- `Ctrl+Enter` が効かない端末では `Ctrl+J` で送信できる（`Enter` は改行のまま）。
- ポップアップ（候補選択など）の `Enter` は従来どおり選択に使う（改行にはならないことがある）。
- 実装後に `dead_code` 警告が出た場合は、使われなくなったフラグ/分岐（例: 以前の `Shift+Enter` ヒント）を削除し、テスト・スナップショットも合わせて更新する。

## 実装メモ

- `Ctrl+Enter` を `KeyCode::Enter` + `KeyModifiers::CONTROL` として扱う前提で実装している（端末/入力バックエンドによっては届かない可能性がある）。
- `Ctrl+J` は（端末が `Ctrl+Enter` を区別できない場合でも）届きやすいフォールバックとして扱う。
- `Ctrl+Enter` がどう届いているか確認するため、別ツール `_tmp/key-probe`（`crossterm` ベース）を用意している。
- フッターの「改行」表示は `Enter` に統一し、「送信」を `Ctrl+Enter` として表示する。

## 動作確認（別環境で実行）

- `make verify-tui-enter-newline-ctrl-enter-send`（`CODEX_HOME=<リポジトリ配下>/_cache/codex_home` で実行される）
- フォーマット:
  - `make fmt`（nightly rustfmt 推奨。stable だと一部設定が無視される場合あり）
- テスト:
  - `make test-tui`
  - `make test-tui2`
- スナップショット差分が出た場合:
  - `make insta-pending-tui`
  - `make insta-accept-tui`（意図した差分のみ受理する）
  - `make insta-pending-tui2`
  - `make insta-accept-tui2`（意図した差分のみ受理する）

## 実際に試すこと

- `Enter` で改行され、送信されないこと。
- `Ctrl+Enter` で送信されること（効かない端末では `Ctrl+J` で送信できること）。
- （key-probe）`_tmp/key-probe` で `Ctrl+Enter` / `Ctrl+J` を押し、どのキーイベントとして観測されるか確認できること。
- `/di` → `Tab` で `/diff ` 補完後に `Ctrl+Enter` でコマンドが実行されること。
- `/` でコマンド候補が出ている状態で `Enter` が候補選択/確定として動くこと。
- `/_unknown_` のような存在しないコマンドは、`Ctrl+Enter` / `Ctrl+J` で送信するとエラー表示になること。

## つまずきと対処（テスト/スナップショット/警告）

- 送信キー変更後、`Enter` を使っていたテストが `None` になって落ちる場合がある → 送信を意図するテスト入力は `Ctrl+Enter` に更新する。
- フッター表示が変わるため、`insta` のスナップショットが差分になる → 差分を確認して意図どおりなら受理する。
- `dead_code` 警告（旧 `Shift+Enter` ヒント等の残骸）が出た → 未使用の enum variant / フィールド / 分岐を削除して解消する。

## 関連ファイル（今回の変更箇所）

- 入力処理: `codex-rs/tui/src/bottom_pane/chat_composer.rs`
- 入力処理: `codex-rs/tui2/src/bottom_pane/chat_composer.rs`
- フッター表示: `codex-rs/tui/src/bottom_pane/footer.rs`
- フッター表示: `codex-rs/tui2/src/bottom_pane/footer.rs`
- 公開ウィジェットの説明: `codex-rs/tui/src/public_widgets/composer_input.rs`
- 公開ウィジェットの説明: `codex-rs/tui2/src/public_widgets/composer_input.rs`
- キー表示（TUI2）: `codex-rs/tui2/src/pager_overlay.rs` / `codex-rs/tui2/src/slash_command.rs`
- テスト（必要に応じて）: `codex-rs/tui/src/chatwidget/tests.rs` / `codex-rs/tui2/src/bottom_pane/chat_composer.rs`
- スナップショット: `codex-rs/tui/src/bottom_pane/snapshots/` / `codex-rs/tui2/src/bottom_pane/snapshots/`
