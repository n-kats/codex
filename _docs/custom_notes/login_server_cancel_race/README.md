# login server cancel race

## 目的

`make almost` の `codex-app-server` 統合テスト `suite::v2::account::login_account_chatgpt_start_can_be_cancelled` が、`account/login/cancel` 後に `account/login/completed` を待ってタイムアウトすることがあった。

原因は `codex-rs/login/src/server.rs` のキャンセル通知が `tokio::sync::Notify` の「待機者がいないと通知が失われる」性質により、キャンセルが早すぎると `shutdown_notify.notified()` を待っている前に通知が飛び、ログインサーバー側タスクが終了せず `block_until_done()` が返らない点。

## 変更内容

- `ShutdownHandle` に `shutdown_requested: Arc<AtomicBool>` を追加し、`shutdown()` でフラグを立てる
- サーバータスクはループ先頭でフラグを確認し、通知を取り逃がしても確実に終了する

## 対象範囲

- 対象: `ChatGPT` ログインのローカル OAuth コールバックサーバーのキャンセル
- 非対象: 実際の OAuth トークン交換処理（ネットワーク）

## 動作確認

- 既存テスト: `codex-rs/app-server/tests/suite/v2/account.rs` の `login_account_chatgpt_start_can_be_cancelled`
- 追加テスト: `codex-rs/login/tests/custom__login_server_cancel_race.rs` の `custom__ログインサーバ__起動直後にcancelしてもハングしない`
