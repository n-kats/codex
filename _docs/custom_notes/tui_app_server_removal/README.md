# 未接続 `tui_app_server` の削除

## 目的

- 上流に存在しない `codex-rs/tui_app_server` の未接続コードを削除し、実行経路を標準 `codex-rs/tui` に一本化する。
- rebase 時に旧 UI 実装へ引っ張られる差分を減らす。

## 変更内容（何がどう変わるか）

- `codex-rs/tui_app_server/` を削除した。
- `codex-rs/cli/src/main.rs` の `codex_tui_app_server` stub と分岐を削除した。
- interactive TUI は常に `codex_tui::run_main(...)` を使う。
- `--remote` はこの build では未サポートとして即時エラーにした。
- `tui_app_server` にだけあった custom テスト/参照は、標準 `tui` 側の既存テストへ寄せた。

## 対象範囲

- 対象:
  - `codex-rs/cli/src/main.rs`
  - `codex-rs/tui/src/lib.rs`
  - `codex-rs/tui_app_server/` 配下
  - `tui_app_server` を参照していた custom notes
- 非対象:
  - 履歴として残す `_worklist/` と `_notes/deconflict/`

## 注意点（環境差・既知の制約）

- `features.tui_app_server` と schema 上の `tui_app_server` キーは削除した。
- 現在の標準 `tui` は custom 側で remote app-server 接続を実装していないため、`--remote` は使えない。
- upstream は標準 `tui` に remote 対応を持つので、必要になったら upstream 方式で標準 `tui` に直接載せる。

## 動作確認手順（手動・テスト・スナップショット）

- MCP:
  - `cargo check`
  - `cargo test -p codex-tui --lib custom__slash_new__NewSessionをemitしconfigをmutateしない`
  - `cargo test -p codex-tui --lib bang_shell_command_submits_run_user_shell_command_in_standard_tui`
- 手動:
  - `codex` 起動で従来通り標準 TUI が開く
  - `/new` 後の送信と `/exit` が動く
  - `--remote` で明示エラーになる

## つまずきと対処（警告や失敗の修正）

- `tui_app_server` は crate ではなく、CLI からも実行されていなかった。
  - `cli/src/main.rs` の stub が `codex_tui::run_main(...)` を呼んでいただけだった。
- `tui_app_server` 側にだけ custom テストがあるように見えたが、主要な custom テストはすでに標準 `tui` 側へ移植済みだった。
- `_worklist/` と `_notes/deconflict/` には「rebase 通過を優先して旧 UI を残した」記録がある。これは履歴として残す。

## 関連ファイル一覧

- `codex-rs/cli/src/main.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/tui/src/chatwidget/custom_tests.rs`
- `codex-rs/tui/src/custom_config_loader_tests.rs`
- `codex-rs/tui/src/chatwidget/tests/exec_flow.rs`
