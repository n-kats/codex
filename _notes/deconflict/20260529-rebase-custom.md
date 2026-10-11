# 20260529-rebase-custom

- File: `README.md`
  - Line: 1
  - Resolution: custom 維持
  - Note: rebase で発生した README の衝突は、再生された custom 側の内容をそのまま採用した。

- File: `codex-rs/app-server/README.md`
  - Line: 1
  - Resolution: custom 維持
  - Note: app-server の README は custom 側の説明を優先し、上流差分は別途反映する前提で解消した。

- File: `codex-rs/app-server/src/message_processor_tracing_tests.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: テスト本体は custom 側の実装を採用して衝突を解消した。

- File: `codex-rs/app-server/src/request_processors/thread_processor.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: thread processor の custom 差分を優先して解消した。

- File: `codex-rs/app-server/src/request_processors/thread_summary.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: thread summary の custom 側を採用して衝突を解消した。

- File: `codex-rs/app-server/src/request_processors/turn_processor.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: turn processor の custom 側を採用して衝突を解消した。

- File: `codex-rs/app-server/tests/suite/v2/thread_start.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: thread_start の custom テスト差分を優先した。

- File: `codex-rs/cli/src/debug_sandbox.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: debug_sandbox の custom 差分を採用した。

- File: `codex-rs/cli/src/main.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: CLI の custom 差分を採用して rebase 衝突を解消した。

- File: `codex-rs/cli/src/marketplace_cmd.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: marketplace コマンドの custom 差分を採用した。

- File: `codex-rs/cli/src/mcp_cmd.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: mcp コマンドの custom 差分を採用した。

- File: `codex-rs/config/src/loader/mod.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: config loader の custom 差分を採用した。

- File: `codex-rs/core/config.schema.json`
  - Line: 1
  - Resolution: custom 維持
  - Note: schema は custom 側の生成結果を採用した。

- File: `codex-rs/core/src/agents_md.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: AGENTS.md 読み込みまわりの custom 差分を採用した。

- File: `codex-rs/core/src/compact_remote_v2.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: compact remote v2 の custom 差分を採用した。

- File: `codex-rs/core/src/config/config_tests.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: config テストは custom 側を採用した。

- File: `codex-rs/core/src/config/mod.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: config モジュールの custom 差分を採用した。

- File: `codex-rs/core/src/session/handlers.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: session handlers の custom 差分を採用した。

- File: `codex-rs/core/src/session/mod.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: session モジュールの custom 差分を採用した。

- File: `codex-rs/core/src/session/review.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: review/session の custom 差分を採用した。

- File: `codex-rs/core/src/session/session.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: session 本体の custom 差分を採用した。

- File: `codex-rs/core/src/session/tests.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: session テストの custom 差分を採用した。

- File: `codex-rs/core/src/session/turn_context.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: turn context の custom 差分を採用した。

- File: `codex-rs/core/src/tools/handlers/multi_agents_common.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: multi-agent 共通処理の custom 差分を採用した。

- File: `codex-rs/core/src/tools/handlers/shell.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: shell handler の custom 差分を採用した。

- File: `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: unix escalation の custom 差分を採用した。

- File: `codex-rs/core/tests/common/responses.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: responses test helper の custom 差分を採用した。

- File: `codex-rs/core/tests/common/test_codex.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: test_codex helper の custom 差分を採用した。

- File: `codex-rs/core/tests/suite/collaboration_instructions.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: collaboration instructions テストの custom 差分を採用した。

- File: `codex-rs/core/tests/suite/compact.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: compact テストの custom 差分を採用した。

- File: `codex-rs/core/tests/suite/compact_remote.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: compact remote テストの custom 差分を採用した。

- File: `codex-rs/core/tests/suite/compact_resume_fork.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: compact resume/fork テストの custom 差分を採用した。

- File: `codex-rs/core/tests/suite/model_overrides.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: model override テストの custom 差分を採用した。

- File: `codex-rs/core/tests/suite/model_switching.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: model switching テストの custom 差分を採用した。

- File: `codex-rs/core/tests/suite/model_visible_layout.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: visible layout テストの custom 差分を採用した。

- File: `codex-rs/core/tests/suite/override_updates.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: override updates テストの custom 差分を採用した。

- File: `codex-rs/core/tests/suite/permissions_messages.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: permissions messages テストの custom 差分を採用した。

- File: `codex-rs/core/tests/suite/personality.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: personality テストの custom 差分を採用した。

- File: `codex-rs/core/tests/suite/prompt_caching.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: prompt caching テストの custom 差分を採用した。

- File: `codex-rs/core/tests/suite/remote_models.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: remote models テストの custom 差分を採用した。

- File: `codex-rs/core/tests/suite/request_permissions.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: request permissions テストの custom 差分を採用した。

- File: `codex-rs/core/tests/suite/resume.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: resume テストの custom 差分を採用した。

- File: `codex-rs/core/tests/suite/review.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: review テストの custom 差分を採用した。

- File: `codex-rs/exec/src/lib.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: exec crate の custom 差分を採用した。

- File: `codex-rs/mcp-server/src/lib.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: mcp-server の custom 差分を採用した。

- File: `codex-rs/memories/write/src/startup_tests.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: memories write の startup テスト差分を採用した。

- File: `codex-rs/protocol/src/protocol.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: protocol の custom 差分を採用した。

- File: `codex-rs/thread-manager-sample/src/main.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: thread-manager sample の custom 差分を採用した。

- File: `codex-rs/tui/src/app/config_persistence.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: config persistence の custom 差分を採用した。

- File: `codex-rs/tui/src/app/event_dispatch.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: event dispatch の custom 差分を採用した。

- File: `codex-rs/tui/src/app/tests.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: app tests の custom 差分を採用した。

- File: `codex-rs/tui/src/app/thread_routing.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: thread routing の custom 差分を採用した。

- File: `codex-rs/tui/src/app_server_session.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: app-server session の custom 差分を採用した。

- File: `codex-rs/tui/src/bottom_pane/chat_composer.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: chat composer の custom 差分を採用した。

- File: `codex-rs/tui/src/bottom_pane/command_popup.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: command popup の custom 差分を採用した。

- File: `codex-rs/tui/src/bottom_pane/mod.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: bottom pane の custom 差分を採用した。

- File: `codex-rs/tui/src/chatwidget.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: chatwidget の custom 差分を採用した。

- File: `codex-rs/tui/src/chatwidget/constructor.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: chatwidget constructor の custom 差分を採用した。

- File: `codex-rs/tui/src/chatwidget/slash_dispatch.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: slash dispatch の custom 差分を採用した。

- File: `codex-rs/tui/src/chatwidget/tests/composer_submission.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: composer submission テストの custom 差分を採用した。

- File: `codex-rs/tui/src/chatwidget/tests/helpers.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: chatwidget test helpers の custom 差分を採用した。

- File: `codex-rs/tui/src/chatwidget/tests/slash_commands.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: slash commands テストの custom 差分を採用した。

- File: `codex-rs/tui/src/lib.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: tui lib の custom 差分を採用した。
