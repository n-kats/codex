# 20260601-rebase-custom.md

- File: `codex-rs/app-server/src/request_processors/windows_sandbox_processor.rs`
  - Line: 78
  - Resolution: custom 維持
  - Note: Windows sandbox setup の開始/完了フローは custom 側の実装を採用し、再取得ロジックや旧 helper は採用しなかった。

- File: `codex-rs/cli/src/main.rs`
  - Line: 42
  - Resolution: custom 維持
  - Note: CLI の古い archive/unarchive ルートや sandbox 分岐は custom 側へ寄せ、上流の新しい起動経路に合わせて残りを統合した。

- File: `codex-rs/core/config.schema.json`
  - Line: 2440
  - Resolution: custom 維持
  - Note: config schema の custom 拡張項目を維持し、上流の更新を取り込んだうえで schema を再整形した。

- File: `codex-rs/core/src/config/config_tests.rs`
  - Line: 14
  - Resolution: custom 維持
  - Note: config テストの custom 用初期化と新 API 追従を残し、上流側の古い前提は採用しなかった。

- File: `codex-rs/core/src/config/mod.rs`
  - Line: 2205
  - Resolution: custom 維持
  - Note: multi-agent / tool / config 合成ロジックは custom 側の解決順序を維持し、上流のシグネチャ差分だけ反映した。

- File: `codex-rs/core/src/session/review.rs`
  - Line: 114
  - Resolution: custom 維持
  - Note: review thread の TurnContext 初期化は custom 側の goal/tool 連携を維持した。

- File: `codex-rs/core/src/session/tests.rs`
  - Line: 7362
  - Resolution: custom 維持
  - Note: session テストの goal / pending input 互換は custom 側に合わせ、上流の旧型参照を採用しなかった。

- File: `codex-rs/core/src/session/turn_context.rs`
  - Line: 587
  - Resolution: custom 維持
  - Note: TurnContext の追加フィールド/補助メソッドは custom 側の API 形状を維持した。

- File: `codex-rs/core/src/tools/handlers/multi_agents_tests.rs`
  - Line: 11
  - Resolution: custom 維持
  - Note: multi-agent v2 の service_tier / spawn 互換テストは custom 側を採用した。

- File: `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs`
  - Line: 121
  - Resolution: custom 維持
  - Note: sandbox / escalation の `ExecRequest` 形状は custom 側の追加フィールドを維持した。

- File: `codex-rs/core/tests/suite/plugins.rs`
  - Line: 14
  - Resolution: custom 維持
  - Note: plugin 周りのテストは custom の新フローに合わせて保持した。

- File: `codex-rs/tui/src/app/config_persistence.rs`
  - Line: 620
  - Resolution: custom 維持
  - Note: TUI の config 永続化は active profile / permission profile の custom 追従を残した。

- File: `codex-rs/tui/src/app/event_dispatch.rs`
  - Line: 875
  - Resolution: custom 維持
  - Note: app event のディスパッチは custom 側の profile / remote / session 更新経路を維持した。

- File: `codex-rs/tui/src/app_server_session.rs`
  - Line: 540
  - Resolution: custom 維持
  - Note: app-server session の thread 操作 API は custom 側の新しいメソッド群を採用した。

- File: `codex-rs/tui/src/chatwidget.rs`
  - Line: 67
  - Resolution: custom 維持
  - Note: ChatWidget の構造は custom 側の追加状態・表示ロジックを維持した。

- File: `codex-rs/tui/src/lib.rs`
  - Line: 171
  - Resolution: custom 維持
  - Note: TUI 起動と remote / sandbox / archive 系の custom ルートを維持した。
