# 20260311-rebase-custom

- File: `.devcontainer/Dockerfile`
  - Line: 15
  - Resolution: custom 側を採用
  - Note: build deps を `pkg-config cmake clang musl-tools libssl-dev libcap-dev libseccomp-dev just` に統一し、競合マーカーを除去した。

- File: `codex-rs/cli/src/login.rs`
  - Line: 130
  - Resolution: 手動マージ
  - Note: `config_toml_file` / `no_config` 引数対応（custom）を維持しつつ、`codex login` の file-backed tracing（上流）を各 login フローで初期化するように統合した。

- File: `codex-rs/core/config.schema.json`
  - Line: 1895
  - Resolution: 手動マージ
  - Note: `default_permissions`（上流）と `custom`（fork 固有）を両方残して schema 競合を解消した。

- File: `codex-rs/core/src/exec.rs`
  - Line: 795
  - Resolution: custom 側を採用
  - Note: `SpawnChildRequest` が `run_as` / `sandbox_policy` を受け取る前提に合わせ、`network_sandbox_policy` ではなく `run_as` / `sandbox_policy` を渡すようにした。

- File: `codex-rs/core/src/landlock.rs`
  - Line: 45
  - Resolution: custom 側を採用
  - Note: `SpawnChildRequest` のフィールドに合わせ `run_as: None` / `sandbox_policy` を渡すようにし、競合マーカーを除去した。

- File: `codex-rs/core/src/sandboxing/mod.rs`
  - Line: 650
  - Resolution: 上流優先
  - Note: Linux sandbox の argv 生成は `create_linux_sandbox_command_args_for_policies` を使う（本番で利用可能な関数）に統一した。

- File: `codex-rs/core/src/seatbelt.rs`
  - Line: 35
  - Resolution: custom 側を採用
  - Note: `SpawnChildRequest` のフィールドに合わせ `run_as: None` / `sandbox_policy` を渡すようにした。

- File: `codex-rs/core/src/spawn.rs`
  - Line: 1
  - Resolution: custom 側を採用
  - Note: `SpawnChildRequest` を `run_as` / `sandbox_policy` ベースにし、run-as（uid/gid）対応を含む実装を採用した。

- File: `codex-rs/core/src/tools/runtimes/apply_patch.rs`
  - Line: 70
  - Resolution: 手動マージ
  - Note: `CommandSpec` に `run_as: None` を追加しつつ、`sandbox_permissions` / `additional_permissions` は request から引き継ぐように統合した。

- File: `codex-rs/core/src/unified_exec/process_manager.rs`
  - Line: 20
  - Resolution: 手動マージ
  - Note: `ExecCommandToolOutput` と `RunAsUser` の import を両立し、PTY spawn は `prepared.env` / `prepared.arg0` を使い `TerminalSize::default()` を渡すようにした。

- File: `codex-rs/exec/src/lib.rs`
  - Line: 1
  - Resolution: custom 側を採用
  - Note: exec の実行パスは app-server client 経由ではなく `ThreadManager` / `CodexThread` ベースの実装を採用した。

- File: `codex-rs/linux-sandbox/tests/suite/landlock.rs`
  - Line: 120
  - Resolution: 手動マージ
  - Note: `codex-linux-sandbox` のパス解決は `super::codex_linux_sandbox_exe()`（環境差に強い）に統一し、競合マーカーと不要な重複ブロックを除去した。

- File: `codex-rs/otel/src/events/session_telemetry.rs`
  - Line: 25
  - Resolution: custom 側を採用
  - Note: `traceparent_context_from_env` + chrono import を残し、競合マーカーを除去した。

- File: `codex-rs/otel/src/lib.rs`
  - Line: 10
  - Resolution: custom 側を採用
  - Note: `OtelManager` 追加に必要な import/定義を残し、競合マーカーを除去。上流で削除された `traces` モジュールは削除した。

- File: `codex-rs/tui/src/app.rs`
  - Line: 2990
  - Resolution: custom 側を採用
  - Note: `UpdateFeatureFlags` は empty を早期 return し、features の constrained 更新 + config への永続化 + Windows sandbox 変更時の turn context override を行う実装を採用した。

- File: `codex-rs/tui/src/lib.rs`
  - Line: 1180
  - Resolution: custom 側を採用
  - Note: `loader_overrides_from_cli` と diff palette override の反映を追加し、競合マーカーを除去した。

- File: `codex-rs/tui/src/multi_agents.rs`
  - Line: 20
  - Resolution: 手動マージ
  - Note: `SpawnRequestSummary`（上流の spawn 表示）を維持しつつ、status dot 表示や補助 struct の重複定義を整理して競合を解消した。

- File: `codex-rs/utils/pty/src/lib.rs`
  - Line: 1
  - Resolution: 手動マージ
  - Note: `DEFAULT_OUTPUT_BYTES_CAP` を残しつつ、Unix の `RunAsUser` struct も公開して両方を併存させた。

