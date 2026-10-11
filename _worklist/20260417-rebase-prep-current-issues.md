# Rebase Prep: Current Issues Before `fork-origin/main` Rebase

## 背景

`custom` ブランチは標準 TUI の `app-server` 化と remote surface の整備を進めているが、`fork-origin/main` との差分が大きくなっている。
rebase 前に、現状の不具合・未収束点を整理しておく。

## 主要な問題

### 1. config 読み込み/反映が不安定

- `--config` / `loader_overrides` の経路は修正しているが、
  本家の config フローへ差分が十分に乗っていない可能性がある。
- `custom.user_shell.no_inject` や `custom.exec.worker_user` が、
  実際の実行経路まで end-to-end で反映されていない疑いがある。
- 起動時/リロード時/onboarding 後の config 再構成で、本家の loader flow に合わせる必要がある。

### 2. user shell の `no_inject` が期待通りに効いていない

- `!pwd` を打っても後続 UI/履歴で `pwd` として扱われるように見える。
- `custom.user_shell.no_inject = true` の設定が、
  - 履歴保存
  - model コンテキスト注入
  - 表示上のコマンド再現
  に正しく伝播しているか要確認。

### 3. worker-user 実行が反映されていない疑い

- `id` の結果が `uid=1000(ubuntu)` のままで、設定した worker-user が使われていないように見える。
- `custom.exec.worker_user` が shell/exec の実行本体へ届いているか、
  本家の Run-As 実装に寄せて再確認が必要。

### 4. `app-server` 化の hybrid がまだ残っている

- `tui/src/app.rs` は `AppServerSession` 側の bridge を増やしているが、
  まだ local `ThreadManager` 前提の残りがある。
- `ThreadStarted` / `ThreadStatusChanged` / `SkillsChanged` / request bridge は進めているが、
  本家の event loop と完全一致ではない。
- `Not available in TUI yet for thread ...` のような fallback メッセージは、
  upstream 追従時に残すべきか整理が必要。

### 5. `/new` / `/resume` / `/fork` と approval/history の整合性

- 一部の live thread lifecycle を app-server に寄せ戻しているが、
  approval response の thread lookup や history replay の整合性がまだ崩れうる。
- `Failed to find thread ... for approval response` のような不整合は、
  rebase 後に再発しやすいので、衝突解消時に優先チェックする。

## 今後の方針

- `bwrap` と `sudo` は競合ではなく別軸として扱う。
- upstream にある `bwrap` の sandbox 流れは優先して取り込む。
- custom の `sudo` / `run_as` は、spawn / sandbox の seam にだけ差し込む。
- つまり「どちらかを消す」のではなく、「同じ抽象に乗せて backend を分ける」方針で解消する。
- UI や config の上位で `sudo` / `bwrap` の違いを広げず、`core/src/spawn.rs` と `core/src/tools/runtimes/shell/unix_escalation.rs` のような実行境界で合成する。
- rebase では upstream 優先で衝突を解き、custom 差分は実行境界へ薄く戻す。

## rebase 全体方針

- まず upstream の責務分離を採用し、custom は seam にだけ差し込む。
- config / login / UI は upstream の流れを壊さず、custom 設定は正規化層で吸収する。
- shell / exec / sandbox は、`run_as` と `bwrap` を別軸として backend 合成する。
- hybrid を温存しない。`false &&` のような一時しのぎではなく、upstream 形に戻すか、custom を seam に寄せて再実装する。
- rebase の判断に迷ったら、`fork-origin/main` 側の責務配置を優先し、custom は最小差分で追随する。

### 詳細な判断基準

- `bwrap` は sandbox の責務として扱う。
  - filesystem / network / namespace の制御は sandbox 層で完結させる。
  - UI / login / config の上位に sandbox backend の詳細を漏らさない。
- `sudo` / `run_as` は実行ユーザーの責務として扱う。
  - spawn / exec の境界で user switch を適用する。
  - sandbox と同一の概念として扱わず、あとから合成する。
- `core/src/codex.rs` は turn_context の組み立てと配布が主で、`worker_user` / `no_inject` の適用場所ではない。
- `custom.exec.worker_user` は config の正規化層から `spawn` / `unix_escalation` の seam へ流し、`custom.user_shell.no_inject` は `user_shell` の注入/履歴保存境界に閉じる。
- `tui/src/app.rs` の `ThreadManager` 参照は test-only cleanup に閉じ、runtime では app-server bootstrap と local `ModelsManager` を使う。
- upstream にある流れを壊すような custom 分岐は避ける。
  - まず upstream の責務配置を受け入れる。
  - custom はその seam にだけ追加する。
- 「どちらかを消す」ではなく、backend を分けて合成する。
  - 例: `sudo` で実行ユーザーを切り替えた上で `bwrap` を適用する。
  - 例外がある場合も、config で明示し、暗黙の hybrid にしない。
- rebase 中に一旦 `false &&` や no-op で逃がした箇所は、最終的に消す。
  - その場しのぎの条件分岐は rebase 後の保守性を悪化させる。
  - 迷ったら upstream の実装を採用し、custom は test で補う。

## rebase 時に優先して見るファイル

- `codex-rs/tui/src/lib.rs`
- `codex-rs/tui/src/app.rs`
- `codex-rs/tui/src/chatwidget.rs`
- `codex-rs/tui/src/app/app_server_notifications.rs`
- `codex-rs/tui/src/app/app_server_requests.rs`
- `codex-rs/tui/src/custom_config_loader_tests.rs`
- `codex-rs/cli/src/main.rs`
- `codex-rs/cli/src/login.rs`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs`
- `codex-rs/core/src/tools/runtimes/shell/unix_escalation_tests.rs`

## 次のアクション候補

1. `fork-origin/main` を取り込み、config / shell / worker-user の差分を upstream 基準で見直す。
2. `custom.user_shell.no_inject` と `custom.exec.worker_user` の end-to-end 経路を、rebase 後に再検証する。
3. hybrid の残りを減らしてから、`cargo check` と selected test を回す。
4. 変更点を `_docs/custom_notes/tui_remote_alignment/README.md` とこの worklist に追記する。

## 2026-04-21 追加整理: app-server に載せる前提での移行方針

前提:

- 今回は「いったん upstream 旧形に戻す」のではなく、`app-server` / `app-server-protocol` / app-server client ルートへ寄せながら収束させる。
- `tui` / `exec` / remote surface は app-server の consumer に寄せる。
- `core` / `protocol` 側は、app-server が必要とする contract を upstream 相当まで引き上げるが、local fallback を増やして app-server 側に責務を戻さない。

### 戻さないもの

- `ThreadManager` 直結の旧 runtime を app-server の代替経路として復活させない。
- `codex_message_processor.rs` に「current tree だけ通すための一時 wrapper」を増やさない。
- realtime 開始時の `output_modality` / `transport` / `voice` を app-server で握りつぶして `prompt + session_id` だけに落とす方向へは戻さない。
- `instruction_sources` を app-server 側の adhoc 実装で再構成しない。AGENTS.md / global instructions の責務は core 側に寄せる。

### 先に合わせるべき contract

- `protocol::ConversationStartParams`
  - upstream 相当の realtime 開始 payload を先に揃える。
  - `output_modality`
  - `prompt: Option<Option<String>>`
  - `session_id`
  - `transport`
  - `voice`
  - app-server-protocol v2 が既にこの shape を前提にしているので、app-server 側で潰すのではなく core/protocol を合わせる。
  - v2 から core への変換は app-server の thin adapter ではなく protocol 側の `From<ThreadRealtimeStartParams>` に寄せる。
- `core::realtime_conversation`
  - すでに upstream 相当の field を使う方向に寄っているため、`protocol` と整合させて app-server realtime tests を通す。
- `core::AgentsMdManager` / instruction source path
  - app-server が thread start/read/resume/fork で使う `instruction_sources` は core の責務として復元する。
  - app-server 側で project doc 探索を再実装しない。
- `config_loader::load_config_layers_state`
  - app-server の `skills_list` / config layering は upstream 相当の `fs` 受け取り形に寄せる。
  - `LOCAL_FS` 固定で app-server 側から埋めるのは最後の呼び出し箇所だけに留め、signature 自体は upstream に合わせる。
- `config_loader::project_trust_key`
  - app-server から trust override を作るために必要なので、private helper の複製ではなく core 側から使える surface に揃える。

### app-server に寄せるときの実装順

1. `protocol` と `core` の public contract を upstream 相当に揃える
2. `app-server` をその contract の単純な consumer に戻す
3. `tui` / `exec` は app-server client ルートに寄せ、local fallback を増やさない

この順番にしないと、`codex_message_processor.rs` に一時吸収ロジックが増えて、rebase 後に再び hybrid になる。

### `codex_message_processor.rs` で避けるべき修正

- `load_config_layers_state` の current signature に合わせるために、app-server 側で env/filesystem 解決を抱え込む
- realtime transport を app-server だけで吸収して core へ落とす
- `AgentsMdManager` の代わりに app-server 内 helper で global/project instruction source を合成する
- plugin cache refresh や subtree enumeration を app-server 独自の degraded path で置き換える

### 直近の失敗テストを app-server 前提でどう捌くか

- `client_metadata`
  - まず `Op::UserInput.responsesapi_client_metadata` を core/app-server 間で完全に通す
  - app-server が request body を作る時点で落とさない
- `thread_start_response_includes_loaded_instruction_sources`
  - app-server 側の整形ではなく、core が返す instruction source contract を復元する
- `skills_list_skips_cwd_roots_when_environment_disabled`
  - `load_config_layers_state(fs, ...)` と skills loader の fs 受け渡しを upstream と同じ責務配置に戻す
- `plugin_install_makes_bundled_mcp_servers_available_to_followup_requests`
  - plugin install 後の app-server cache invalidation / refresh を app-server 管理下で完結させる
- `realtime_*`
  - `ConversationStartParams` と `realtime_conversation` を app-server v2 の shape に合わせる
  - app-server で field を消して通そうとしない

### 次モデル向けの実行方針

- 差分確認は `git diff --stat` を見ながら進める
- まず `core/src/lib.rs` / `core/src/config_loader/mod.rs` / `protocol/src/protocol.rs` の public contract を app-server 前提で揃える
- 次に `app-server/src/codex_message_processor.rs` の adhoc 差分を減らす
- その後に MCP で selected test を再実行する
- `almost` は、上の contract が揃ってから再実行する

## 2026-04-21 追加整理2: 難所ごとの具体的な設計方針

### A. realtime を app-server 主導へ移す設計

問題:

- `app-server-protocol` v2 はすでに `output_modality` / `prompt: Option<Option<String>>` / `transport` / `voice` を持っている
- 一方で current tree の `protocol::ConversationStartParams` は古い `prompt + session_id` shape に留まっている
- そのため `codex_message_processor.rs` が app-server v2 request をそのまま core へ流せず、そこで握りつぶす誘惑が出る

設計方針:

- realtime の canonical contract は `app-server-protocol v2` と `core::realtime_conversation` の間で一致させる
- `codex_message_processor.rs` は「transport を解釈して変換するだけ」の薄い adapter にする
- realtime の仕様判断は app-server ではなく core に集約する

具体:

- `protocol::ConversationStartParams` を upstream 相当に上げる
  - `output_modality`
  - `prompt: Option<Option<String>>`
  - `session_id`
  - `transport`
  - `voice`
- `core::realtime_conversation::prepare_realtime_start(...)` は `ConversationStartTransport` を受け、そのまま provider/session config に落とす
- `app-server` は `ThreadRealtimeStartTransport` -> `ConversationStartTransport` の enum 変換だけ持つ
- validation は core に寄せる
  - `RealtimeOutputModality::Text` が v1 では不可
  - voice/version の整合
  - `transport=webrtc` のときの SDP 必須

移行手順:

1. `protocol` を上げる
2. `core::realtime_conversation` の current 実装をその contract に合わせる
3. `bespoke_event_handling.rs` の realtime notification 変換を見直す
4. `app-server` realtime tests を selected で回す

### B. instruction_sources を app-server response に載せる設計

問題:

- `thread_start_response_includes_loaded_instruction_sources` が落ちている本質は、app-server が project docs しか返せず、global codex home の AGENTS.md を落としていること
- ここを app-server 側 helper で合成すると、AGENTS.md 探索と model-visible instructions の責務が二重化する

設計方針:

- instruction source discovery の canonical owner は core
- app-server は「thread に紐づく config から source paths を取得して response に載せる」だけにする

具体:

- `core::agents_md` を upstream と同じ public surface に戻す
  - `pub(crate) mod agents_md`
  - `pub use agents_md::AgentsMdManager`
  - 必要なら filename const も re-export
- `CodexMessageProcessor::instruction_sources_from_config(...)` は `AgentsMdManager::instruction_sources(...)` の呼び出しだけにする
- thread start / read / resume / fork の全レスポンスで、同じ helper を通して source path を取得する
- `ThreadState` に持つ `instruction_sources` は app-server の cache ではなく「response ordering のための配送用 snapshot」とみなす
  - source discovery 自体は state に閉じ込めない

難しい点への対処:

- `config_snapshot.cwd` が `AbsolutePathBuf` で、`Config.cwd` は `PathBuf` の場面がある
  - ここは app-server 側で path をいじらず、「config を組み直す helper」を用意するか、`instruction_sources_from_config_snapshot(...)` の薄い専用関数に分ける
  - ただし AGENTS.md 探索ロジックの本体は core 側から呼ぶ

### C. skills_list / plugin install を app-server 管理下で収束させる設計

問題:

- `skills_list_skips_cwd_roots_when_environment_disabled`
  - config layer 解決と fs 注入が中途半端で、app-server が current environment と `LOCAL_FS` のどちらを使うべきか曖昧
- `plugin_install_makes_bundled_mcp_servers_available_to_followup_requests`
  - plugin install 後に plugin cache, skills cache, MCP refresh, connector visibility が別々に更新され、follow-up request から見た整合が崩れやすい

設計方針:

- app-server は「plugin/skills の orchestration owner」になる
- ただし config layer 読み込み、skills discovery、plugin metadata 解釈の実体は core 側の API に寄せる

具体:

- `load_config_layers_state(fs, ...)` を upstream shape に揃える
  - fs を明示で受ける
  - `skills_list` は current environment があればその fs、無ければ `LOCAL_FS` を渡す
  - どの fs を使うかは app-server が決めるが、layer 解決本体は config_loader に置く
- `skills_for_cwd_with_extra_user_roots(...)`
  - app-server は `SkillsLoadInput` を組むだけ
  - cwd roots をスキップする判定は skills manager 側の contract に寄せる
- plugin install 後は app-server で次を必ず直列実行する
  - plugin install
  - config reload
  - plugin/skills cache clear
  - bundled MCP server refresh queue
  - 必要なら OAuth login start
  - connector/app visibility recompute
- follow-up request が参照するのは「refresh 済み state」
  - install response は早く返してもよいが、`plugin_install_makes_bundled_mcp_servers_available_to_followup_requests` を守るなら少なくとも app-server 内の cache invalidation は response 前に終わらせる

難しい点への対処:

- upstream にある `maybe_start_non_curated_plugin_cache_refresh(...)` が current tree に無い場合
  - app-server 内で別 helper を足して代用するのではなく、plugins manager に戻す
  - 理由: cache refresh policy は app-server ではなく plugin subsystem の責務だから

### D. thread lifecycle を app-server 一本化する設計

問題:

- `/new` / `/resume` / `/fork` / `turn/start` / `turn/interrupt` / approval/history replay が、local `ThreadManager` 直結経路と app-server 経路の混在で壊れやすい
- token usage replay や interrupted turn の扱いも thread listener ordering に依存している

設計方針:

- live thread lifecycle の single source of truth は app-server
- `ThreadManager` は app-server の内部依存であり、TUI/exec が直接 thread lifecycle を組み立てない
- event ordering と request resolution は `ThreadState` / thread listener の文脈で直列化する

具体:

- `PendingThreadResumeRequest` のような app-server 側の配送用 state は維持する
  - ただし「history をどう読むか」「instruction_sources をどう計算するか」は core helper に寄せる
- `TurnStarted` / `TurnComplete` / `TurnAborted` / realtime notifications は `bespoke_event_handling.rs` に集約
  - TUI 向け shape への変換はここ
  - turn summary や active turn snapshot の更新は `ThreadState` が持つ
- `turn_interrupt_aborts_running_turn`、resume/fork の token usage replay 系は
  - rollout/history 解釈を core
  - response ordering を app-server listener
  に分担する

難しい点への対処:

- interrupted tail turn を resume/replay 時にどう扱うか
  - app-server で「途中だったので捨てる/採る」を決めない
  - core が返す `InitialHistory` と `TurnAbortedEvent` / token usage event を正として app-server は notification 順序だけ管理する
- approval response の thread lookup
  - TUI 側で thread を推測しない
  - app-server の pending request table と thread listener generation で解決する

### E. client_metadata を通す設計

問題:

- `turn_start_*client_metadata*` 系テストは、app-server が Responses API 向け metadata を turn 単位で core に渡し切れていないと落ちる
- ここを request body 直前で付け足すと、turn state と analytics がずれる

設計方針:

- canonical data model は `Op::UserInput.responsesapi_client_metadata`
- app-server は `TurnStartParams` / `TurnSteerParams` からこれを `Op` に載せるだけにする
- request body への merge は core/client 側に寄せる

具体:

- app-server-protocol v2 params -> `Op::UserInput.responsesapi_client_metadata`
- turn metadata の reserved field merge も core に任せる
- app-server は connection/request trace と turn metadata を別概念として持つ

### F. 実装順の固定

難所を同時に触ると差分が拡散するので、順番を固定する。

1. `protocol` realtime contract
2. `core::realtime_conversation`
3. `core::agents_md` / `config_loader` public surface
4. `app-server::codex_message_processor` の adhoc 吸収を削減
5. `plugin install` / `skills list` の ownership 整理
6. `bespoke_event_handling` / `ThreadState` の ordering 調整
7. selected test
8. `almost`

### G. 設計上の禁止事項

- app-server 内に temporary compatibility layer を積み増して current tree だけ通す
- realtime / instruction source / config layer / plugin cache policy を app-server 独自ロジックにする
- `ThreadManager` 直結経路を「テストが通るから」で runtime に戻す
- 失敗テストごとに別経路を追加して対応する

## 差分サイズの読み方

`git diff --numstat fork-origin/main -- codex-rs` の `(+add / -del)` は、単なる量ではなく
「旧実装をどれだけ消して、custom seam をどこに残したか」の目安として見る。

- [`tui/src/app.rs`](/workspace/codex-rs/tui/src/app.rs) `(+3068 / -5104)`
  - ここは old hybrid を消せているかが本題。
  - 削除が多いのは、単に upstream に寄ったからではなく、旧 thread lifecycle / event bridge を落としている可能性がある。
  - custom seam だけ残っているなら良いが、二重経路が残っていると危険。

- [`tui/src/chatwidget.rs`](/workspace/codex-rs/tui/src/chatwidget.rs) `(+1655 / -2981)`
  - direct sender / `ThreadManager` 直結の残骸がないかを見る。
  - custom の入力・履歴・送信 seam を残すのは OK。
  - 旧初期化経路が残っているなら要修正。

- [`tui/src/bottom_pane/chat_composer.rs`](/workspace/codex-rs/tui/src/bottom_pane/chat_composer.rs) `(+2971 / -1487)`
  - 入力と送信の seam が upstream 形に寄っているかを見る。
  - `!` / paste burst / submit 判定の旧経路が残っていないか確認対象。

- [`core/src/codex.rs`](/workspace/codex-rs/core/src/codex.rs) `(+5212 / -720)`
  - custom 実装の本丸なので差分は大きくなりやすい。
  - ただし、古い二重経路や hybrid を残すのは避ける。

- [`protocol/src/protocol.rs`](/workspace/codex-rs/protocol/src/protocol.rs) `(+71 / -535)`
  - upstream 追従で削除が多い。
  - 型・責務のズレが残っていないかを重点的に確認する。

- [`core/src/config/mod.rs`](/workspace/codex-rs/core/src/config/mod.rs) `(+1251 / -437)`
  - custom 設定を seam に残すため差分が大きくてもあり得る。
  - ただし config 正規化が upstream と噛み合っているかは要確認。

- [`cli/src/main.rs`](/workspace/codex-rs/cli/src/main.rs) `(+327 / -533)`
  - CLI 入口の custom 反映が upstream の流れに乗っているかを見る。
  - flags の seam が壊れていないかを確認する。

- [`exec/src/lib.rs`](/workspace/codex-rs/exec/src/lib.rs) `(+715 / -616)`
  - `sudo` / `run_as` と `bwrap` の別軸がここで壊れていないかを見る。
  - backend 合成の seam を確認する。

## 2026-04-22 app-server 移行メモ

- `skills/list` は `ThreadManager::current_environment_filesystem()` を使い、exec 環境が無い場合は cwd roots を載せない。
- これは `CODEX_EXEC_SERVER_URL_ENV_VAR=none` の環境で cwd discovery を抑え、`skills_for_cwd_with_extra_user_roots` の既存挙動に合わせるため。
- TUI 起動時の model / available_models の取得は app-server bootstrap を source of truth にして、`ThreadManager` のモデル一覧取得を startup から外す。
- `tui/src/app.rs` の runtime は old `ThreadManager` 直結を復活させない。必要な model catalog は `AppServerSession::bootstrap` と `ModelsManager` で賄う。
- `tui/src/app.rs` の test helper も `ThreadManager` 直結を持たず、`AppServerSession` を通す形へ寄せる。
- `tui/src/lib.rs` の起動フローも onboarding 用 / session lookup 用の app-server 二重起動を避け、upstream と同じく単一 `AppServerSession` を基本にして picker で消費した時だけ再生成する。
- `tui/src/chatwidget/tests.rs` も `ThreadManager` を介さず `ModelsManager::new(...)` を直接使う。
- `tui/src/chatwidget.rs` の MCP startup 追跡は simple map に潰さず、upstream の round 管理（expected servers / ignore stale updates / pending next round）へ寄せる。
- `thread_start` / `thread_read` / `thread_resume` / `thread_fork` の instruction sources も current environment filesystem を優先し、無い場合だけ local fs に落とす。
- `plugin/install` は response 前に `queue_mcp_server_refresh_for_config(...).await` を通すが、bundled MCP server の follow-up 可用性は引き続き確認対象。
- `plugin/install` は `plugin/list` と同じ `maybe_start_non_curated_plugin_cache_refresh` seam に寄せ、install 後の config reload を cache refresh 後に回す。
- `Config::to_mcp_config()` で `config.toml` 由来の `mcp_servers` に plugin 由来の `effective_mcp_servers()` を合成し、`mcpServer/oauth/login` や status refresh が follow-up で見えるようにする。
- install 後の bundled MCP server / app 探索は installed path の再スキャンではなく `plugins_for_config(&config)` の outcome を source of truth にする。
- installed plugin の lookup は `PluginLoadOutcome` の helper に寄せて、app-server 側の文字列走査を減らす。
- plugin outcome からは MCP server と app connector をまとめて取り、app-server の個別 lookup を減らす。
- 次は `plugin/install` と `client_metadata` の app-server v2 経路を、戻す方向ではなく contract を揃える方向で詰める。
