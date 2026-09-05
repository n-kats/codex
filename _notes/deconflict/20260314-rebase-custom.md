# 20260314-rebase-custom

- File: `codex-rs/core/src/config/mod.rs`
  - Line: 140 / 290 / 900 / 2230 / 2410 / 2825
  - Resolution: 手動マージ
  - Note: 上流の `approvals_reviewer` / `web_search_config` / `windows_sandbox_private_desktop` / guardian alias migration を取り込みつつ、custom の memories ルート（`CODEX_MEMORIES_HOME`）、`exec_run_as`、UserShell policy/no-inject、`load_config_as_toml_with_cli_overrides_and_loader_overrides` を維持した。

- File: `codex-rs/core/src/sandboxing/mod.rs`
  - Line: 60 / 700
  - Resolution: 手動マージ
  - Note: 上流の `windows_sandbox_private_desktop` を維持しつつ、custom の `ExecRequest.run_as` を追加して `SandboxManager::env_for(...)` で配線した（tests 構造は上流の `mod_tests.rs` を採用）。

- File: `codex-rs/core/src/exec.rs`
  - Line: 70 / 230 / 290 / 790
  - Resolution: 手動マージ
  - Note: `ExecRequest.run_as` を `ExecParams` に伝播し、`spawn_child_async_with_run_as` 経由で Unix の setuid/setgid 実行ができるようにした。合わせて `build_exec_request` で `CommandSpec.run_as` をセットするよう修正した。

- File: `codex-rs/utils/pty/src/pipe.rs`
  - Line: 95 / 285 / 330
  - Resolution: 手動マージ
  - Note: 上流の `inherited_fds` 維持（`spawn_process_no_stdin_with_inherited_fds`）と、custom の run-as 実行（`RunAsUser`）を両立するため、内部 spawn helper に `run_as` を追加し `spawn_process_with_run_as` / `spawn_process_no_stdin_with_run_as` を復元した。

- File: `codex-rs/shell-escalation/src/unix/escalate_client.rs`
  - Line: 40 / 120
  - Resolution: custom 維持
  - Note: execve-wrapper から渡される `file` が `git` 等の素のコマンド名でも `PATH` 解決し、`EscalateRequest.file` を絶対パスとして送るようにした。

- File: `codex-rs/tui/src/lib.rs`
  - Line: 300 / 450
  - Resolution: 手動マージ
  - Note: 上流の `LoaderOverrides` 経路を維持しつつ、custom の `agents_md`（`project_doc_paths`）配線を `run_main(..., agents_md)` として復元し、`user_config_toml_path` helper（onboarding 用）も追加した。

- File: `codex-rs/cli/src/main.rs`
  - Line: 60 / 580 / 980
  - Resolution: custom 維持
  - Note: top-level `--agents-md` を復元し、interactive は TUI へ、exec/review は `codex_exec::run_main_with_agents_md` へ伝播。回帰テスト用に `INTERACTIVE_TUI_AGENTS_MD_CAPTURE` を復元した。

- File: `codex-rs/app-server/src/codex_message_processor.rs`
  - Line: 5890
  - Resolution: 手動マージ
  - Note: 上流の `submit_core_op(...)`（tracing 伝播）を採用しつつ、`Op::OverrideTurnContext` に `project_doc_paths: None` を補って欠けないようにした。

- File: `codex-rs/core/src/tasks/user_shell.rs`
  - Line: 155
  - Resolution: 手動マージ
  - Note: UserShell の `ExecRequest` に `run_as: None` を明示し、上流の `windows_sandbox_private_desktop` も欠けないようにした。

- File: `codex-rs/tui/src/chatwidget/tests.rs`
  - Line: 8400
  - Resolution: 手動マージ
  - Note: permission popup の項目追加（Smart Approvals）に追従しつつ、custom の送信キー（Ctrl+Enter）に合わせて操作イベントを更新した。

- File: `codex-rs/core/tests/suite/snapshots/all__suite__compact_remote__remote_pre_turn_compaction_restates_realtime_end_shapes.snap`
  - Line: 3
  - Resolution: 手動マージ
  - Note: `assertion_line` のみ競合だったため、`core/tests/suite/compact_remote.rs` の `assert_snapshot!` 行番号に合わせて更新した。

- File: `codex-rs/core/tests/suite/snapshots/all__suite__compact_remote__remote_compact_resume_restates_realtime_end_shapes.snap`
  - Line: 3
  - Resolution: 手動マージ
  - Note: `assertion_line` のみ競合だったため、`core/tests/suite/compact_remote.rs` の `assert_snapshot!` 行番号に合わせて更新した。

- File: `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__approvals_selection_popup.snap`
  - Line: 3
  - Resolution: 手動マージ
  - Note: `assertion_line` のみ競合だったため、`tui/src/chatwidget/tests.rs` の `assert_snapshot!` 行番号に合わせて更新した。

## Post-merge build fixes

- File: `codex-rs/core/src/sandboxing/mod.rs`
  - Resolution: ビルドエラー修正
  - Note: `ExecRequest.run_as` に合わせて `CommandSpec.run_as` を追加し、`RunAsUser` を import して型解決できるようにした。

- File: `codex-rs/core/src/tools/**`
  - Resolution: ビルドエラー修正
  - Note: `TurnContext.exec_run_as` が存在しないため、参照を `turn.config.exec_run_as` へ修正した（shell / unified_exec / js_repl）。

- File: `codex-rs/core/src/config/mod.rs`
  - Resolution: ビルドエラー修正
  - Note: `resolve_web_search_config` が削除されていたため、未使用の呼び出しを削除した（実際の `Config.web_search_config` は `cfg.tools.web_search` から設定している）。

- File: `codex-rs/core/src/codex.rs`
  - Resolution: ビルドエラー修正
  - Note: `Op::OverrideTurnContext` に `project_doc_paths` が追加されたため、match パターンで `..` を使って将来フィールド追加にも追従できるようにした。

- File: `codex-rs/app-server/src/codex_message_processor.rs`
  - Resolution: ビルドエラー修正
  - Note: `ExecParams` に `run_as` が追加されたため、app-server の exec 経路でも `run_as: self.config.exec_run_as.clone()` をセットして custom の worker user 設定を反映できるようにした。

- File: `codex-rs/core/src/custom_prompts.rs`
  - Resolution: custom 機能復元 + テスト修正
  - Note: `CODEX_ADDITIONAL_PROMPT_DIRS` のパース（カンマ区切り・相対パスは cwd 基準）と、複数ディレクトリからの prompt 発見（同名は後勝ち）を復元した。`list_custom_prompts` からも複数ディレクトリを参照するよう更新した。

- File: `codex-rs/core/src/codex_tests.rs`
  - Resolution: テスト修正
  - Note: `Op::OverrideTurnContext` に `project_doc_paths` を追加。`SessionConfiguration` から `base_instructions_pinned` が削除されていたため初期化を追従した。

- File: `codex-rs/core/src/config/service_tests.rs`
  - Resolution: テスト修正
  - Note: `LoaderOverrides` に `user_config_path` / `disable_user_config` / `disable_project_config` が追加されたため、`..Default::default()` で補完して追従した。

- File: `codex-rs/core/src/project_doc/custom_tests.rs`
  - Resolution: テスト修正
  - Note: `get_user_instructions` のシグネチャ変更（`(&Config)` のみ）に追従した。

- File: `codex-rs/tui/src/app.rs`
  - Resolution: テスト修正
  - Note: `Op::OverrideTurnContext` に `project_doc_paths` が追加されたため、TUI 側の `assert_eq!(Ok(Op::OverrideTurnContext { ... }))` を追従した。

- File: `codex-rs/tui/src/chatwidget/tests.rs`
  - Resolution: テスト修正
  - Note: 同上。permissions popup のテストで `project_doc_paths: None` を追加した。

- File: `codex-rs/core/src/config_loader/tests.rs`
  - Resolution: テスト修正
  - Note: `LoaderOverrides` の追加フィールドに追従するため、macOS 向け requirements テストの `LoaderOverrides { ... }` に `..LoaderOverrides::default()` を追加した。

- File: `codex-rs/linux-sandbox/src/linux_run_main_tests.rs`
  - Resolution: テスト修正
  - Note: `build_preflight_bwrap_argv` / `ensure_inner_stage_mode_is_valid` の引数数変更に追従して、不要になった引数を削除した。

- File: `codex-rs/tui/src/app.rs`
  - Resolution: ビルドエラー修正
  - Note: `Op::OverrideTurnContext` に `project_doc_paths` が追加されたため、Smart Approvals の runtime patch 送信箇所でも `project_doc_paths: None` を補った。

- File: `codex-rs/tui/src/chatwidget.rs`
  - Resolution: ビルドエラー修正
  - Note: `Op::OverrideTurnContext` に `approvals_reviewer` が必須のため、`/custom-agents` の override op に `approvals_reviewer: None` を追加した。

- File: `codex-rs/exec/src/lib.rs`
  - Resolution: ビルドエラー修正
  - Note: `Cli` に `config_toml_file` / `no_config` が追加されたため destructuring を `..` で追従し、`ConfigOverrides` の必須フィールド `project_doc_paths`（空）を追加した。

- File: `codex-rs/core/tests/suite/personality.rs`
  - Resolution: テスト修正
  - Note: `ModelInfo` に `supports_search_tool` が追加されたため、テスト用の `ModelInfo { ... }` 初期化に `supports_search_tool: false` を追加した。

- File: `codex-rs/cli/src/main.rs`
  - Resolution: ビルドエラー修正
  - Note: `run_interactive_tui(..., agents_md)` の引数変更に追従して Resume/Fork 経路にも `agents_md` を渡すようにした。あわせて login/logout の helper 関数が `config_toml_file` / `no_config` を受け取るため、TUI CLI 側でパースされた値を渡すよう修正した（tests の destructuring も追従）。

- File: `codex-rs/exec/src/lib.rs`
  - Resolution: custom 機能復元 + ビルドエラー修正
  - Note: `codex-cli` から `--agents-md` を exec に伝播するため `run_main_with_agents_md` を復元し、`ConfigOverrides.project_doc_paths` に反映した。さらに `--config/--no-config` を exec 経由でも効かせるため `LoaderOverrides.user_config_path/disable_user_config` を反映した。

- File: `codex-rs/tui/src/diff_render.rs`
  - Resolution: 警告抑制
  - Note: `set_diff_palette_override` が現状未使用で `dead_code` warning が出るため `#[allow(dead_code)]` を付与した（挙動変更なし）。

- File: `codex-rs/tui/src/chatwidget/custom_tests.rs`
  - Resolution: テスト修正
  - Note: `Op::OverrideTurnContext` に `approvals_reviewer` が必須のため、テスト期待値に `approvals_reviewer: None` を追加した。

- File: `codex-rs/cli/src/main.rs`
  - Resolution: 警告抑制
  - Note: `#[cfg(test)]` の agents_md capture helper が一部のビルド構成で未使用になり `dead_code` warning となるため、関数に `#[allow(dead_code)]` を付与した（挙動変更なし）。

- File: `codex-rs/core/src/project_doc.rs`
  - Resolution: custom 機能復元
  - Note: `Config.project_doc_paths`（custom-agents / --agents-md 由来）の明示指定がある場合は、AGENTS.md の自動探索より優先してそのパスを返すよう `discover_project_doc_paths` を修正した。相対パスは `cwd` 基準で解決する。
