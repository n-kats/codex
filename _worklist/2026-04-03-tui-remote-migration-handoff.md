# TUI Remote Migration Handoff

## 目的

標準 `tui` を upstream の app-server ベース構成へ寄せる。custom 機能は維持しつつ、旧実装依存を減らす。最終的に `--remote` を本家方式で有効化したい。

## 作業方針

- 日本語で報告
- 変更は小さく、局所的に
- upstream に寄せる。独自実装を増やさない
- `tui_app_server` は削除済み。復活させない
- `ThreadManager` と shared `AppServerSession` はまだ別 runtime なので、両者を混ぜて同一 thread 集合として扱わない
- Rust の検証は MCP の `cargo check` / 必要最小限の selected test を使う
- `make` は使わない
- 変更したら `_docs/custom_notes/tui_remote_alignment/README.md` を更新する

## 現状

- `cli/src/main.rs`
  - `--remote` / `--remote-auth-token-env` の surface は upstream 寄りに復元済み
- `tui/src/lib.rs`
  - `normalize_remote_addr(...)` あり
  - onboarding 後に runtime 用 embedded `AppServerSession` を起動し、`run_ratatui_app(...)` に渡している
  - `--remote` の surface は upstream 形に戻し、`AppServerTarget` で embedded / remote を切り替える入口を戻した
  - onboarding 用に作った app-server は embedded target のときだけ shutdown し、remote target では drop だけにして外部 server を落とさない
  - remote 起動時は trust screen を抑制し、trust-only onboarding でも app-server を立てる upstream 形に寄せた
  - `remote auth token` の transport validation と `load_config_or_exit(...)` の `loader_overrides` 非依存形も upstream に戻した
  - `read_session_cwd(...)` / `resolve_cwd_for_resume_or_fork(...)` を upstream の `Option<&Path>` 形へ戻し、`read_session_model(...)` も upstream 互換で復活させた
  - `LoginStatus::AuthMode` は app-server protocol の `AuthMode` を使う形へ戻した
  - `run_ratatui_app(...)` の終端処理を upstream どおり `TerminalRestoreGuard` ベースへ戻した
  - onboarding の `auth_manager` 引き回しを削って、login step は `app_server_request_handle` だけで構成する upstream 形に戻した
- `tui/src/chatwidget.rs`
  - 旧 `ChatWidget::new(...)` は削除済み
  - live 経路は `CommandTransport::AppEvent`
  - `new_from_existing(...)` は live `CodexThread` を要求しない
- `tui/src/app.rs`
  - shared `AppServerSession` を保持
  - plugin 系 RPC は shared app-server の request handle 経由
  - `pending_app_server_requests` を保持
  - `submit_thread_op()` で app-server request resolution を先に試し、該当時は `resolve_server_request(...)` へ返す
  - それ以外は従来どおり `ThreadManager` thread へ submit
  - `/new` の初期 thread 生成は `AppServerSession::start_thread(...)` を優先し、session 再構成も app-server 起点に寄せた
  - `/new` / `/resume` / `/fork` は app-server で live thread lifecycle を作り直す本線に戻し、`ThreadManager` 直結の旧経路は runtime から外した
  - `backfill_loaded_subagent_threads(...)` を戻し、app-server 側に既に載っている subagent を `/agent` picker に再登録できるようにした
  - `/new` の thread 切り替えは upstream 形に寄せ、旧 thread へ `Op::Shutdown` を送らず unsubscribe + listener abort で処理する
  - remote URL / auth token の no-op state は削除した
  - `/exit` も upstream 形に寄せ、`Op::Shutdown` ではなく thread unsubscribe で閉じる
  - `StartFresh` の thread 生成は app-server がある場合に `AppServerSession::start_thread(...)` を優先するようにした
  - `resume` / `fork` の app-server 経路でも `reset_thread_event_state()` の後に `enqueue_primary_event(SessionConfigured(...))` を通して `active_thread_id` を復元するように寄せた
  - UI の `OpenResumePicker` / `ForkCurrentSession` も同じ再接続パターンに寄せた
  - `ThreadStarted` 通知は `primary_session_configured` から session を推定し、`ThreadEventStore` の初期 session として載せる
  - `ThreadStarted` の session 再構成では、`SessionSource::SubAgent(ThreadSpawn { parent_thread_id, ... })` から `forked_from_id` を復元する
  - `/agent` の live attach は app-server が使える場合に `thread/read` を優先して liveness を判定する
  - `ThreadStatusChanged` 通知は `NotLoaded` を closed、それ以外を open として `/agent` picker に反映する
  - `SkillsChanged` 通知は `ListSkills` の再取得を促すようにした
  - `reasoning_effort` は `App` 側の専用 state を canonical にして、config は rebuild 用の mirror として扱うようにした
  - `fresh_session_config()` も canonical な reasoning effort state を使うようにした
- `tui/src/app/app_server_requests.rs`
  - staged で存在
  - request resolution は live
  - `ServerRequest` の一部を UI に載せる request bridge を進めている
  - `SkillsChanged` の再取得テスト `app_server_skills_changed_triggers_skill_reload` を追加した
  - `update_reasoning_effort_updates_collaboration_mode` は reasoning effort state 分離後も通過した
- `tui/src/app/app_server_notifications.rs`
  - app-server の notification を TUI の core `Event` に寄せる bridge を持つ
  - thread lifecycle notification は `/agent` picker の表示更新にも反映する
- `tui_app_server` は削除済み
- `features.tui_app_server` と schema key も削除済み

## 重要な制約

- `ThreadRolledBack` の live response は app-side で完結させ、`ChatWidget` 側の replay 用 `ThreadRolledBack` 処理とは分離する。
- `App` は runtime で app-server を field に二重所有せず、local `AppServerSession` 参照で動かす。
- `loaded_threads` を shared app-server から読んで current `ThreadManager` の subagent と混ぜない
- `ThreadManager` 直結の runtime はまだ一部残っており、remote 本体はまだ hybrid
- `StartFresh` は app-server 優先に寄せたが、resume / fork / agent attach などの旧経路はまだ一部残る
- `/new`・`/resume`・`/fork` の live thread lifecycle は app-server 側に戻し、runtime の `thread_created_rx` / `handle_thread_created` は削除した
- approval response と history が必要なテストだけ local `ThreadManager` の test helper を使う
- `/agent` の live attach は app-server 形へ少しずつ寄せているが、完全な `AppServerSession` ベース移行はまだ途中
- `resume` / `fork` は app-server 経路で live thread の再接続を進めている
- `AppEvent::TranscriptionComplete` / `AppEvent::TranscriptionFailed` は戻し済み
- `run_ratatui_app(...)` で `AuthManager` を再構成し、`App::run(...)` 呼び出しの欠落を解消済み

## 次にやること

1. shared `AppServerSession` の event stream を `App::run()` の main loop に接続する
2. `pending_app_server_requests.note_server_request()` / `resolve_notification()` が実際に使われる状態にする
3. `ThreadStarted` 系の session 再構成と request buffering を upstream に寄せる
4. remote 起動経路の実機確認をする
5. `cargo check` と短い selected test を回す
6. note 更新

## まず読むべきファイル

- `/workspace/codex-rs/tui/src/app.rs`
- `/workspace/codex-rs/tui/src/chatwidget.rs`
- `/workspace/codex-rs/tui/src/app/app_server_notifications.rs`
- `/workspace/codex-rs/tui/src/app/app_server_requests.rs`
- `/workspace/codex-rs/tui/src/lib.rs`
- `/workspace/_docs/custom_notes/tui_remote_alignment/README.md`


## 直近で通っている確認

- MCP `cargo check`
- `cargo test -p codex-tui --lib custom__slash_new__NewSessionをemitしconfigをmutateしない`
- `cargo test -p codex-tui --lib shutdown_current_thread_removes_server_thread`

## 報告のしかた

- 何を upstream に寄せたか
- 何がまだ hybrid で残っているか
- `cargo check` と selected test の結果
- `_docs/custom_notes/tui_remote_alignment/README.md` の更新有無

## rebase 前の未収束問題

- `custom.user_shell.no_inject` が `!` の履歴/注入に end-to-end で反映されているか不明。
- `custom.exec.worker_user` が `id` / shell 実行に反映されているか不明。
- `--config` / loader overrides は修正したが、本家の config フローへ十分に乗っているか再確認が必要。
- `AppServerSession` 化の hybrid が残っており、approval/history と live thread の整合性が再発しうる。
