# TUI Remote Alignment

## 目的

- 標準 `tui` の CLI/TUI 入口を upstream の `--remote` / `--remote-auth-token-env` 形に寄せる。
- `tui_app_server` 削除後も、remote 移行の差分を小さく保つ。

## 変更内容

- `cli/src/main.rs`
  - `InteractiveRemoteOptions` に `--remote-auth-token-env` を戻した。
  - remote auth token の環境変数読み出し helper を upstream 相当で戻した。
  - interactive 入口で remote URL 正規化を行うようにした。
- `tui/src/lib.rs`
  - `normalize_remote_addr` を upstream 相当で戻した。
  - `remote_auth_token` があるときは upstream と同じ transport チェックを行うようにした。
  - `run_main(...)` の引数形を upstream 寄りに合わせ、`remote` / `remote_auth_token` を受けるようにした。
  - `--remote` 入力は `AppServerTarget` に正規化され、embedded / remote の app-server 起動を切り替える入口を戻した。
  - `load_config_or_exit(...)` / `load_config_or_exit_with_fallback_cwd(...)` は upstream の `loader_overrides` 非依存形に戻した。
  - `read_session_cwd(...)` / `resolve_cwd_for_resume_or_fork(...)` を upstream の `Option<&Path>` 形へ戻し、`read_session_model(...)` も upstream 互換で復活させた。
  - `LoginStatus::AuthMode` は app-server protocol の `AuthMode` を使う形へ戻した。
  - `run_ratatui_app(...)` の終端処理を upstream どおり `TerminalRestoreGuard` ベースに戻した。
  - onboarding の `auth_manager` 引き回しを削って、upstream 同様 `app_server_request_handle` だけで login step を構成する形に戻した。
- `tui/src/chatwidget.rs`
  - `Op` 送信を `UnboundedSender<Op>` 直結だけに依存しないようにし、`AppEvent::CodexOp` 経由へ流せる `CommandTransport` を導入した。
  - fresh session は `new_with_app_event(...)` を使うように寄せた。
- `tui/src/app.rs`
  - `AppEvent::CodexOp` を active thread に流す経路を追加した。
  - `StartFresh` と `/new` の初期化を `ChatWidget::new_with_app_event(...)` ベースへ寄せた。
  - `StartFresh` の thread 生成は app-server が使える場合に `AppServerSession::start_thread(...)` を優先するようにした。
  - plugin 系 RPC は毎回別の embedded app-server を起こす旧経路をやめ、`App` が持つ shared `AppServerSession` の request handle を使うようにした。
  - `pending_app_server_requests` を `App` に戻し、approval / request_user_input / MCP elicitation 応答は `resolve_server_request(...)` へ返せる staged 経路を復元した。
  - app-server の notification stream を受ける入口を追加し、account / rate-limit / thread notification の一部を current TUI に流せるようにした。
  - `ServerRequest` の受信を live に戻し、command/file-change/permissions approval、`request_user_input`、MCP elicitation の一部を bottom pane / chatwidget に流せるようにした。
  - `ThreadStarted` 通知は `primary_session_configured` から session を推定し、`ThreadEventStore` の初期 session として載せるようにした。
- `tui/src/lib.rs`
  - onboarding 後に runtime 用の embedded `AppServerSession` を起動し、`App::run(...)` へ渡すようにした。

## 対象範囲

- 対象:
  - CLI/TUI の remote オプション surface
  - remote URL バリデーション
- 非対象:
  - `tui/src/app.rs` 全体の `AppServerSession` 移行
  - 実際の remote 実行

## 注意点

- 現在の標準 TUI はまだ `ThreadManager` 直結経路を多く持っており、upstream の `AppServerSession` ベースへは未移行。
- ただし plugin 系の app-server RPC は shared session に寄せたため、旧来の「都度 embedded app-server 起動」より upstream に近い。
- `pending_app_server_requests` の response 解決は live で、notification/request bridge も `App` 側へ個別に戻している。
- `ThreadStarted` / `ThreadArchived` / `ThreadClosed` / `ThreadUnarchived` などの thread lifecycle は `/agent` picker の表示更新に反映するようにした。
  - `/new` / `/resume` / `/fork` は app-server で live thread lifecycle を作り直す本線に戻し、`ThreadManager` 直結の旧経路は runtime から外した。
  - `backfill_loaded_subagent_threads(...)` を戻し、app-server 側に既に載っている subagent を `/agent` picker に再登録できるようにした。
  - `ThreadStarted` 通知は `/agent` picker の表示更新に加えて、初期 session の再構成にも使うようにした。
  - `ThreadStarted` の `SessionSource::SubAgent(ThreadSpawn { parent_thread_id, ... })` から `forked_from_id` を復元するようにした。
  - `/agent` の live attach は app-server が使える場合に `thread/read` を優先して liveness を判定するようにした。
  - `ThreadStatusChanged` 通知は `NotLoaded` を closed、それ以外を open として `/agent` picker に反映するようにした。
  - `SkillsChanged` 通知は `ChatWidget` 側で `ListSkills` の再取得を要求するようにした。
  - `reasoning_effort` は `App` 側の専用 state を canonical にして、config は rebuild 用の mirror として扱うようにした。
  - `fresh_session_config()` は canonical な reasoning effort state を優先して session rebuild に渡すようにした。
- `resume` / `fork`
  - app-server を使える場合は、`reset_thread_event_state()` の後に `enqueue_primary_event(SessionConfigured(...))` を通して `active_thread_id` を復元し、live session を再接続するようにした。
  - UI からの `OpenResumePicker` / `ForkCurrentSession` も同じ再接続パターンに寄せた。
- `/new` の thread 切り替えでは、upstream に合わせて旧 thread へ `Op::Shutdown` を送らず、thread unsubscribe + listener abort で切り替える。
- `/exit` も upstream に合わせて `Op::Shutdown` ではなく thread unsubscribe で閉じる。
- app-server の request bridge は部分的に live で、`ThreadStarted` 系の session 再構成も入れた。
- `/new`・`/resume`・`/fork` の live thread lifecycle は app-server 側に戻した。
- runtime の `thread_created_rx` / `handle_thread_created` は削除し、thread lifecycle は app-server の `ThreadStarted` / `ThreadArchived` / `ThreadClosed` / `ThreadUnarchived` 通知に一本化した。
- approval response と history の整合性が必要なテストだけ local `ThreadManager` の test helper を使う。
- subagent thread の live event stream や turn 実行は、app-server 側の bridge で `/agent` picker に再登録できるようにした。
- `--remote` の surface は upstream 形に戻し、`AppServerTarget` を通して embedded / remote を切り替えられるようにした。
- onboarding 用に作った app-server は embedded target のときだけ shutdown し、remote target では drop だけにして外部 server を落とさない。
- remote 起動時は upstream に合わせて trust screen を抑制し、trust-only onboarding でも app-server を立てる。
- `App` に残していた remote URL / auth token の no-op state は削除した。
- `App::run(...)` 全体はまだ `ThreadManager` 直結経路を一部持っているため、remote 実行本体はまだ hybrid。
- この変更は「入口差分の縮小」と「`ChatWidget -> App` の送信責務を寄せる」前準備が目的。
- `backfill_loaded_subagent_threads(...)` を戻し、app-server 側に既に載っている subagent を `/agent` picker に再登録できるようにした。
- remote URL / token は `run_ratatui_app(...)` で受け、`AppServerTarget` の remote 起動に使う。
- `AppEvent::TranscriptionComplete` / `AppEvent::TranscriptionFailed` を戻し、linux の voice fallback と app 側の受け口を再接続した。
- `run_ratatui_app(...)` では `AuthManager` を再構成し、`App::run(...)` の呼び出しが欠けないように戻した。
- `/new` の初期 thread 生成は `ThreadManager` 直結ではなく、`AppServerSession::start_thread(...)` を優先するように寄せた。
- `/new` の session 再構成は app-server が返す thread session を基準にするように寄せた。
- `StartFresh` は app-server がある時に thread start を優先し、local `ThreadManager` 直結を減らした。
- `/agent` の live attach は app-server が使える場合に `thread/read` を優先するようにした。

### rebase 方針

- upstream の責務分離をまず採用し、custom は seam にだけ差し込む。
- config / login / UI は upstream の流れを壊さず、custom 設定は正規化層で吸収する。
- shell / exec / sandbox は `run_as` と `bwrap` を別軸として backend 合成する。
- hybrid を温存せず、upstream 形に戻すか custom を seam に寄せて再実装する。

### 判断基準の補足

- `bwrap` は sandbox の責務として扱う。
  - filesystem / network / namespace の制御は sandbox 層で完結させる。
  - UI / login / config の上位に sandbox backend の詳細を漏らさない。
- `sudo` / `run_as` は実行ユーザーの責務として扱う。
  - spawn / exec の境界で user switch を適用する。
  - sandbox と同一の概念として扱わず、あとから合成する。
- upstream にある流れを壊す custom 分岐は避ける。
  - まず upstream の責務配置を受け入れる。
  - custom はその seam にだけ追加する。
- `false &&` や no-op で逃がした箇所は rebase 後に削除する。
  - 一時しのぎの条件分岐は hybrid を固定化する。
  - 迷ったら upstream の実装を採用し、custom は test で補う。

## 動作確認手順

- MCP `cargo check`
- MCP `cargo test -p codex-cli --lib remote_flag_parses_for_interactive_root`
- MCP `cargo test -p codex-cli --lib remote_auth_token_env_flag_parses_for_interactive_root`
- MCP `cargo test -p codex-tui shutdown_current_thread_removes_server_thread -- --exact`
- MCP `cargo test -p codex-tui custom__slash_new__NewSessionをemitしconfigをmutateしない -- --exact`

## つまずきと対処

- `--remote` を有効化するときは、`tui/src/app.rs` と `tui/src/chatwidget.rs` の残る hybrid を先に詰める必要がある。
- surface は戻したので、実機では local/remote の責務が混ざっていないかを確認しながら進める。
- `ChatWidget` の送信先を差し替えると、手組みの test helper が旧フィールド初期化で壊れやすい。`command_transport` ベースへ合わせて追従させる。
- `app_server_adapter.rs` の未接続レイヤは削除済み。
- notification / request bridge は `App` / `ChatWidget` 側へ薄く戻している。
- `ThreadStarted` の session 再構成は、sub-agent spawn 元の thread id を `forked_from_id` として残す。
- `ThreadStarted` の回帰テスト `app_server_thread_started_updates_picker_and_session_cache` を追加済み。
- `ThreadStatusChanged` の回帰テスト `app_server_thread_status_changed_updates_picker_closed_state` を追加済み。
- `SkillsChanged` の回帰テスト `app_server_skills_changed_triggers_skill_reload` を追加済み。
- `/agent` の live attach は app-server 形へ少しずつ寄せているが、完全な `AppServerSession` ベース移行はまだ途中。
- `update_reasoning_effort_updates_collaboration_mode` は `App` の reasoning effort state 分離後も通る。

- `ThreadRolledBack` の live response は `handle_backtrack_rollback_succeeded()` だけで処理し、`ChatWidget` への再送はしない。二重適用を避けるため。
- `App` は runtime で app-server を field に二重所有せず、local `AppServerSession` 参照で動かすように寄せている。

## 関連ファイル

- `/workspace/codex-rs/cli/src/main.rs`
- `/workspace/codex-rs/tui/src/app.rs`
- `/workspace/codex-rs/tui/src/app/app_server_notifications.rs`
- `/workspace/codex-rs/tui/src/chatwidget.rs`
- `/workspace/codex-rs/tui/src/chatwidget/tests.rs`
- `/workspace/codex-rs/tui/src/lib.rs`
- `/workspace/codex-rs/tui/src/main.rs`
- `/workspace/codex-rs/tui/src/app/app_server_requests.rs`

## rebase 前の注意

- config の loader overrides は修正済みだが、`custom.user_shell.no_inject` と `custom.exec.worker_user` は本家との差分がまだ残っている疑いがある。
- `!` の履歴保存/注入、worker-user の run-as は、rebase 時に優先して見直す。
- `AppServerSession` の bridge は hybrid のまま残っているため、approval/history の不整合が起きやすい。
