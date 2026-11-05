# 20260509-rebase-custom

- File: `codex-rs/app-server-protocol/src/protocol/v2.rs`
  - Line: 1
  - Resolution: upstream 優先
  - Note: `v2/mod.rs` と重複していた monolithic `v2.rs` を削除し、モジュール分割された upstream の構成に寄せた。

- File: `codex-rs/app-server/README.md`
  - Line: 90
  - Resolution: custom 維持
  - Note: `thread/turns/list` と `thread/metadata/update` の custom 記述を残した。

- File: `codex-rs/app-server/src/codex_message_processor.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: upstream 削除に対して custom 実装を保持した。

- File: `codex-rs/app-server/src/in_process.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: 内容衝突は custom 側を採用した。

- File: `codex-rs/app-server/src/message_processor_tracing_tests.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: 内容衝突は custom 側を採用した。

- File: `codex-rs/app-server/tests/suite/v2/thread_read.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: 内容衝突は custom 側を採用した。

- File: `codex-rs/cli/src/main.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: `Responses` サブコマンドを含む custom CLI 結線を維持した。

- File: `codex-rs/core/config.schema.json`
  - Line: 1
  - Resolution: custom 維持
  - Note: config schema の custom 追加項目を維持した。

- File: `codex-rs/core/src/config/config_tests.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の config テスト群を維持した。

- File: `codex-rs/core/src/config/mod.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の config 読み込み/拡張を維持した。

- File: `codex-rs/core/src/session/tests.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の session テストを維持した。

- File: `codex-rs/core/src/session/turn_context.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の turn context 変更を維持した。

- File: `codex-rs/core/src/tools/handlers/shell.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の shell 実行経路を維持した。

- File: `codex-rs/core/src/tools/runtimes/apply_patch.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の apply_patch ランタイム変更を維持した。

- File: `codex-rs/core/tests/suite/plugins.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の plugin テスト変更を維持した。

- File: `codex-rs/exec/src/lib.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の exec 実装変更を維持した。

- File: `codex-rs/linux-sandbox/tests/suite/landlock.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の landlock テスト変更を維持した。

- File: `codex-rs/linux-sandbox/tests/suite/managed_proxy.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の managed proxy テスト変更を維持した。

- File: `codex-rs/mcp-server/src/lib.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の mcp-server 実装変更を維持した。

- File: `codex-rs/thread-store/Cargo.toml`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の thread-store 依存/設定を維持した。

- File: `codex-rs/thread-store/examples/generate-proto.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: upstream 削除に対して custom の例を残した。

- File: `codex-rs/thread-store/scripts/generate-proto.sh`
  - Line: 1
  - Resolution: custom 維持
  - Note: upstream 削除に対して custom の補助スクリプトを残した。

- File: `codex-rs/tui/src/bottom_pane/chat_composer.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の chat composer 実装を維持した。

- File: `codex-rs/tui/src/bottom_pane/command_popup.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の command popup 実装を維持した。

- File: `codex-rs/tui/src/chatwidget.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の chatwidget 変更を維持した。

- File: `codex-rs/tui/src/chatwidget/slash_dispatch.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の slash dispatch 変更を維持した。

- File: `codex-rs/tui/src/session_resume.rs`
  - Line: 1
  - Resolution: custom 維持
  - Note: custom の session resume 変更を維持した。
