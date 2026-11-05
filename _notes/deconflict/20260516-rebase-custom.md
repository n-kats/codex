# 20260516-rebase-custom

- File: `codex-rs/app-server/README.md`
  - Line: 205
  - Resolution: 手動マージ
  - Note: `remoteControl/enable` / `disable` / `status/read` を戻しつつ、実装と protocol に存在しない `device/key/*` 記述は入れず、`remoteControl/status/changed` は custom 側の説明を維持した。

- File: `codex-rs/app-server/src/message_processor_tracing_tests.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の tracing テスト実装を維持した。

- File: `codex-rs/app-server/tests/suite/v2/thread_read.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の thread read テスト変更を維持した。

- File: `codex-rs/cli/src/debug_sandbox.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の debug sandbox 実装を維持した。

- File: `codex-rs/cli/src/main.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の CLI 結線を維持した。

- File: `codex-rs/cli/src/marketplace_cmd.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の marketplace コマンド実装を維持した。

- File: `codex-rs/config/src/loader/layer_io.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の config layer 入出力変更を維持した。

- File: `codex-rs/config/src/loader/mod.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の loader 変更を維持した。

- File: `codex-rs/core/config.schema.json`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の config schema 追加項目を維持した。

- File: `codex-rs/core/src/config/config_tests.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の config テスト変更を維持した。

- File: `codex-rs/core/src/config/mod.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の config 読み込み拡張を維持した。

- File: `codex-rs/core/src/session/mod.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の session 管理変更を維持した。

- File: `codex-rs/core/src/session/tests.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の session テスト変更を維持した。

- File: `codex-rs/core/src/session/tests/guardian_tests.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の guardian テスト変更を維持した。

- File: `codex-rs/core/src/tools/handlers/mod.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の tool handler 切り替えを維持した。

- File: `codex-rs/core/src/tools/handlers/shell.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の shell handler 変更を維持した。

- File: `codex-rs/core/src/tools/runtimes/apply_patch.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の apply_patch runtime 変更を維持した。

- File: `codex-rs/core/tests/common/test_codex.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の test helper 変更を維持した。

- File: `codex-rs/exec-server/tests/file_system.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の exec-server file system テスト変更を維持した。

- File: `codex-rs/exec/src/lib.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の exec 実装を維持した。

- File: `codex-rs/mcp-server/src/lib.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の mcp-server 実装を維持した。

- File: `codex-rs/mcp-server/src/main.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の mcp-server entrypoint 変更を維持した。

- File: `codex-rs/rmcp-client/src/program_resolver.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の program resolver 変更を維持した。

- File: `codex-rs/thread-store/Cargo.toml`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の thread-store 依存設定を維持した。

- File: `codex-rs/tui/src/app/config_persistence.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の TUI 設定永続化変更を維持した。

- File: `codex-rs/tui/src/app/tests.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の app テスト変更を維持した。

- File: `codex-rs/tui/src/app_server_session.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の app-server session 変更を維持した。

- File: `codex-rs/tui/src/bottom_pane/chat_composer.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の chat composer 変更を維持した。

- File: `codex-rs/tui/src/bottom_pane/command_popup.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の command popup 変更を維持した。

- File: `codex-rs/tui/src/bottom_pane/mod.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の bottom pane 集約変更を維持した。

- File: `codex-rs/tui/src/chatwidget.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の chatwidget 変更を維持した。

- File: `codex-rs/tui/src/chatwidget/slash_dispatch.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の slash dispatch 変更を維持した。

- File: `codex-rs/tui/src/chatwidget/tests/composer_submission.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の composer submission テストを維持した。

- File: `codex-rs/tui/src/chatwidget/tests/helpers.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の test helpers を維持した。

- File: `codex-rs/tui/src/chatwidget/tests/plan_mode.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の plan mode テストを維持した。

- File: `codex-rs/tui/src/chatwidget/tests/review_mode.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の review mode テストを維持した。

- File: `codex-rs/tui/src/chatwidget/tests/slash_commands.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の slash command テストを維持した。

- File: `codex-rs/tui/src/lib.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の TUI module wiring を維持した。
