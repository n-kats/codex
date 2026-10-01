# 2026-04-30 TUI Enter/Newline custom の削除

`Enter`=改行 / `Ctrl+Enter`・`Ctrl+J`=送信 のカスタム実装を外し、設定例だけ残す作業メモ。

## 残すもの

- [`/workspace/sample_config.toml`](/workspace/sample_config.toml)
  - `tui.keymap` の設定例として残す。
  - 複数キー割り当ての例も維持する。

## 外すもの

1. TUI の表示・入力ロジック
   - `codex-rs/tui/src/bottom_pane/chat_composer.rs`
   - `codex-rs/tui/src/bottom_pane/footer.rs`
   - `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
   - `codex-rs/tui/src/public_widgets/composer_input.rs`

2. このカスタムに紐づくスナップショット
   - `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__chat_composer__tests__footer_mode_shortcut_overlay.snap`
   - `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__footer__tests__footer_shortcuts_collaboration_modes_enabled.snap`
   - `codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__footer__tests__footer_shortcuts_shift_and_esc.snap`

3. カスタム説明ノート
   - `_docs/custom_notes/tui-enter-newline-ctrl-enter-send/README.md`

## 作業順

1. 上記ファイルを本家側の挙動に戻す。
2. 必要なら関連スナップショットを本家状態へ戻す。
3. `sample_config.toml` は残したまま、設定例として整合するか確認する。
4. `just fmt` を実行する。
5. `codex-tui` の影響範囲テストを回す。
6. 変更理由を `_docs/custom_notes/` に残す必要があるか最後に判断する。

## 注意

- keymap 基盤そのものは触らない。
- 複数キー割り当ての設定機能も維持する。
- 送信/改行の既定挙動だけを本家に戻す。

## 進行状況

- [x] ワークリスト作成
- [x] TUI の enter/newline カスタム実装を削除
- [x] 関連 snapshot を本家状態へ戻す
- [x] `sample_config.toml` を保持
- [x] カスタム説明ノートを削除
- [x] `codex-tui` の関連テストを確認
