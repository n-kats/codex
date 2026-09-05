# 20260807 rebase custom

- File: `codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst`
  - Line: binary file
  - Resolution: upstream 基盤 + custom 差分を統合
  - Note: upstream 側の最新 experimental export を基礎にし、`ThreadSettingsUpdateParams.project_doc_paths` の custom schema 差分だけを加えた。古い custom fixture をそのまま採用すると、upstream の protocol 更新が欠落して schema test が失敗するため。

- File: `codex-rs/cli/src/main.rs`, `codex-rs/cli/src/remote_control_cmd.rs`
  - Line: 1283 / 68, 80, 130
  - Resolution: 手動マージ
  - Note: 上流の PSP 引数と custom の LoaderOverrides 伝播を併存させた。

- File: `codex-rs/cli/src/mcp_cmd/cloud_config.rs`, `codex-rs/tui/src/onboarding/auth.rs`, `codex-rs/tui/src/session_archive_commands.rs`
  - Line: 12, 41 / 1035, 1067, 1077 / 18, 38, 352
  - Resolution: 手動マージ
  - Note: CLI の cloud-managed config 無効化を維持し、ログイン UI の上流 cloud loader と session archive の feature-gated 経路を残した。

- File: `codex-rs/code-mode-runtime/src/service.rs`, `codex-rs/code-mode-runtime/src/service_tests.rs`
  - Line: 35, 119 / 15
  - Resolution: 手動マージ後に整理
  - Note: custom の待機時間ヘルパーは残し、参照されていなかった in-process provider は削除した。上流の execution observe mode および session limit 適用は維持した。

- File: `codex-rs/core-skills/src/loader_tests.rs`
  - Line: 144, 2192, 2322
  - Resolution: upstream 優先
  - Note: custom 側の config-layer テストは現行 core-skills loader に存在しない API を参照していたため取り込まず、上流の root-based test helper とテストを採用した。

- File: `codex-rs/core/src/config/mod.rs`, `codex-rs/exec/src/lib.rs`, `codex-rs/tui/src/lib.rs`
  - Line: 2641, 3350 / 70, 387, 488 / 12, 58, 1010, 1116, 1479
  - Resolution: 手動マージ
  - Note: 上流の PSP 設定と custom の project document paths を ConfigOverrides に併存させ、cloud feature 用の補助経路を残した。

- File: `codex-rs/core/src/tools/code_mode/execute_handler.rs`, `codex-rs/core/src/tools/spec_plan.rs`
  - Line: 30, 66 / 682
  - Resolution: 手動マージ
  - Note: 上流の cached runtime/code-mode definitions を維持し、custom の MCP 完了待機による yield 無期限化を追加した。

- File: `codex-rs/core/src/tools/orchestrator.rs`, `codex-rs/core/src/tools/registry.rs`
  - Line: 245 / 262, 644
  - Resolution: 手動マージ後に整理
  - Note: legacy landlock の custom 判定、RegisteredTool、非同期 telemetry tags は維持し、参照されていなかった exposure override は削除した。

- File: `codex-rs/core/src/config/mod.rs`, `codex-rs/core/src/exec.rs`, `codex-rs/core/src/session/turn_context.rs`, `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs`
  - Line: post-rebase warning cleanup
  - Resolution: 手動整理
  - Note: legacy landlock 判定の未使用 `cwd` 引数と exec 側の未使用 runtime policy 変数を削除し、選択された環境の deprecated な session cwd 参照を解消した。

- File: `codex-rs/tui/src/lib.rs`, `codex-rs/core/tests/suite/cli_stream.rs`
  - Line: post-rebase warning cleanup
  - Resolution: feature/test 条件整理
  - Note: cloud 無効時に未使用となる本家の auth helper は cloud feature 限定にし、無効化済み PAT テストの補助定義も同じ条件で除外した。

- File: `codex-rs/core/tests/suite/model_switching.rs`
  - Line: 47
  - Resolution: 手動マージ
  - Note: 上流の test_case と custom の tokio Duration import を両方残した。

- File: `codex-rs/config/src/mcp_edit_tests.rs`
  - Line: file addition
  - Resolution: custom 維持
  - Note: custom 側で追加された MCP 編集テストを復元した。

- File: `codex-rs/core/src/tools/parallel.rs`, `codex-rs/core/src/tools/registry.rs`, `codex-rs/core/src/tools/orchestrator.rs`, `codex-rs/core/src/session/turn_context.rs`, `codex-rs/config/src/loader/tests.rs`
  - Line: post-rebase compile fixes
  - Resolution: upstream API 追従
  - Note: `ToolRouter` の現行保持先、`ToolSearchInfo` の import、非同期 telemetry の borrow、`permission_profile()` accessor、config layer iterator API に合わせて修正した。
