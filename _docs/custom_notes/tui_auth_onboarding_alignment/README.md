# tui_auth_onboarding_alignment

## 目的

- 標準 TUI の認証導線を upstream の実装方針に揃え、未認証時に onboarding の login step が正しく表示されるようにする。

## 変更内容（何がどう変わるか）

- `codex-rs/tui/src/lib.rs` の login 判定を `auth.json` 直読みではなく、embedded app-server の `bootstrap()` ベースに戻した。
- `show_login_screen=true` のときは onboarding 用に embedded app-server を起動し、`app_server_request_handle` を `run_onboarding_app(...)` へ渡すようにした。
- onboarding 利用後は一時的に起動した app-server を shutdown する。

## 対象範囲（非対象も）

- 対象: 標準 TUI (`codex-rs/tui`) の起動時 onboarding / login 判定。
- 非対象: `login` crate 自体の token 永続化仕様。

## 注意点（環境差・既知の制約）

- upstream の `OnboardingScreen` は `app_server_request_handle` が `None` だと login step を追加しない。
- そのため、`show_login_screen=true` でも request handle を渡さないと「認証画面が一瞬出て消える」症状になる。
- upstream の通常 Responses 経路は `OPENAI_API_KEY` を自動 fallback しない実装箇所があるため、未認証のまま chat 画面へ進むと `401 Unauthorized` になりうる。

## 動作確認手順（手動・テスト・スナップショット）

- MCP:
  - `run_cargo_check`
- 手動:
  1. `make run-tui` で起動する。
  2. 未認証状態では login onboarding が消えずに表示されることを確認する。
  3. login 完了後に chat 画面へ進むことを確認する。
  4. 通常送信で `401 Unauthorized: Missing bearer or basic authentication in header` が消えることを確認する。

## つまずきと対処（警告や失敗の修正）

- `would_show_login_screen=true` なのに chat 画面へ進む場合:
  - `tui/src/lib.rs` から `run_onboarding_app(...)` に `app_server_request_handle` が渡っているか確認する。
- `OnboardingScreen` に `skipping onboarding login step without app-server request handle` が出る場合:
  - onboarding 用 embedded app-server の起動や `request_handle()` の受け渡しが欠けている。

## 関連ファイル一覧

- `codex-rs/tui/src/lib.rs`
- `codex-rs/tui/src/onboarding/onboarding_screen.rs`
- `codex-rs/core/src/api_bridge.rs`
