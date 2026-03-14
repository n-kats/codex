# TUI: Enter で改行 / Ctrl+Enter で送信

## 注記（tui2 について）

このリポジトリには過去に `codex-rs/tui2` が存在していましたが、現在は削除されています。
そのため、入力キー仕様の実装・テスト・スナップショットの更新対象は **`codex-rs/tui` のみ**です。

## 目的

- 長文入力のたびに誤送信しやすい問題を避けるため、`Enter` は改行、`Ctrl+Enter` を送信にする。

## 変更内容（概要）

- `Enter` を改行、送信キーを `Ctrl+Enter` にした。
- `Ctrl+Enter` が端末で区別できない/潰れる環境向けに、送信のフォールバックとして `Ctrl+J` も扱うようにした。
- フッターのショートカット表示を更新した（`Enter: newline` / `Ctrl+Enter: send (or Ctrl+J)`）。
- 送信に関わるテスト入力を `Ctrl+Enter` / `Ctrl+J` に更新した（UI スナップショット更新が必要になる場合あり）。
- `/` コマンドの実行も「送信」扱い（`Ctrl+Enter` / `Ctrl+J`）に統一する。

## 追記（2026-02-27 回帰修正）

- upstream 取り込み後の回帰で `Enter` が再び送信になっていたため、`chat_composer` のキー分岐を再修正した。
- あわせてフッター表示を `Ctrl+Enter`（端末非対応時は `Ctrl+J`）送信 / `Enter` 改行に戻した。
- `docs/tui-chat-composer.md` の説明文も現行仕様に合わせて更新した。
- 回帰防止として、`chat_composer` に次のキー仕様テストを追加した。
  - `enter_inserts_newline_without_submitting`
  - `ctrl_enter_submits_message`
  - `ctrl_j_submits_message`
- 既存テストの追従として、`request_user_input` の回答確定テスト入力を `Enter` から `Ctrl+Enter` に更新した。
- フッターの collaboration mode ありスナップショット（`footer_shortcuts_collaboration_modes_enabled`）を現行表示に更新した。
- `/custom-agents <path...>` を含む `/` コマンド実行も「送信」扱いとして `Ctrl+Enter` / `Ctrl+J` に統一する（無修飾 `Enter` では実行しない）。

## 追記（2026-03-05 rebase 後のテスト追従）

- `make almost` の `codex-tui` で、送信意図のテストが `Enter`（無修飾）のまま残っていたため、以下のテスト入力を `Ctrl+Enter` に修正した。
- 対象:
  - `codex-rs/tui/src/app.rs`
    - `replayed_turn_complete_submits_restored_queued_follow_up`
    - `replay_only_thread_keeps_restored_queue_visible`
    - `replay_thread_snapshot_keeps_queue_when_running_state_only_comes_from_snapshot`
    - `replay_thread_snapshot_does_not_submit_queue_before_replay_catches_up`
    - `replay_thread_snapshot_restores_pending_pastes_for_submit`
    - `replay_thread_snapshot_restores_collaboration_mode_for_draft_submit`
    - `replayed_interrupted_turn_restores_queued_input_to_composer`
  - `codex-rs/tui/src/chatwidget/tests.rs`
    - `enter_queues_user_messages_while_review_is_running`
    - `manual_interrupt_restores_pending_steers_to_composer`
    - `manual_interrupt_restores_pending_steer_mention_bindings_to_composer`
    - `manual_interrupt_restores_pending_steers_before_queued_messages`
    - `replaced_turn_clears_pending_steers_but_keeps_queued_drafts`
    - `plan_implementation_popup_shows_after_new_plan_follows_steer`
    - `plan_implementation_popup_skips_when_steer_follows_proposed_plan`
    - `steer_enter_uses_pending_steers_while_turn_is_running_without_streaming`
- 方針:
  - 仕様は維持（`Enter`=改行、`Ctrl+Enter`/`Ctrl+J`=送信）。
  - 実装は変えず、rebase で upstream 側に混在した送信キー期待のみテストで揃える。

### 検証ログ（2026-03-04 実行分）

- 実行ログ: `_tmp/almost_test_result.txt`
- ログ更新時刻（UTC）: `2026-03-04 19:15:12`
- 確認結果: `FAILED` / `error: test failed` / `make[*]: ***` の出現なし（テスト通過）

## 対象範囲

- 対象: 入力欄の「送信」操作（通常のメッセージ送信、`/` コマンドの dispatch、`/prompts:*` の実行）。
- 対象: 選択系ポップアップ（ListSelection / file / mention）と review custom prompt view でも `Ctrl+Enter` を受理する。
- 非対象: `Enter` の従来動作（候補選択/確定など）は維持する。`Ctrl+Enter` は追加で受理する互換キー扱い。

## 注意点

- 端末によっては `Ctrl+Enter` を区別できない場合がある（環境依存）。その場合は送信できない可能性があるため、実端末で動作確認する。
- `Ctrl+Enter` が効かない端末では `Ctrl+J` で送信できる（`Enter` は改行のまま）。
- ポップアップ（候補選択など）は `Enter` でも `Ctrl+Enter` でも選択/確定できる。
- 実装後に `dead_code` 警告が出た場合は、使われなくなったフラグ/分岐（例: 以前の `Shift+Enter` ヒント）を削除し、テスト・スナップショットも合わせて更新する。

## 実装メモ

- `Ctrl+Enter` を `KeyCode::Enter` + `KeyModifiers::CONTROL` として扱う前提で実装している（端末/入力バックエンドによっては届かない可能性がある）。
- `Ctrl+J` は（端末が `Ctrl+Enter` を区別できない場合でも）届きやすいフォールバックとして扱う。
- `Ctrl+Enter` がどう届いているか確認するため、別ツール `_tmp/key-probe`（`crossterm` ベース）を用意している。
- フッターの「改行」表示は `Enter` に統一し、「送信」を `Ctrl+Enter` として表示する。

## 動作確認（別環境で実行）

- `make verify-tui-enter-newline-ctrl-enter-send`（`CODEX_HOME=<リポジトリ配下>/_cache/codex_home_debug` で実行される）
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
- フッター表示: `codex-rs/tui/src/bottom_pane/footer.rs`
- 公開ウィジェットの説明: `codex-rs/tui/src/public_widgets/composer_input.rs`
- テスト（必要に応じて）: `codex-rs/tui/src/chatwidget/tests.rs` / `codex-rs/tui/src/bottom_pane/chat_composer.rs`
- スナップショット: `codex-rs/tui/src/bottom_pane/snapshots/`
