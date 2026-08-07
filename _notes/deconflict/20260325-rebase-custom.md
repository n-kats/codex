# 20260325-rebase-custom

- File: `codex-rs/core/src/sandboxing/mod.rs`
  - Line: 44 / 157 / 312
  - Resolution: custom 維持
  - Note: `CommandSpec` / `SandboxTransformRequest` / `mod_tests.rs` の構成を復元し、`ExecRequest` への変換を custom 版に揃えた。

- File: `codex-rs/core/src/tools/runtimes/mod.rs`
  - Line: 34
  - Resolution: custom 維持
  - Note: `build_command_spec` を残し、`CommandSpec` ベースの runtime 呼び出しを維持した。

- File: `codex-rs/core/src/tools/runtimes/shell.rs`
  - Line: 246
  - Resolution: custom 維持
  - Note: `build_command_spec` + `env_for(spec, ...)` に切り替え、`ExecOptions` 依存を外した。

- File: `codex-rs/core/src/tools/js_repl/mod.rs`
  - Line: 1032
  - Resolution: custom 維持
  - Note: `CommandSpec` へ `run_as` と `sandbox_permissions` を含める custom 版を採用した。

- File: `codex-rs/core/src/tools/runtimes/apply_patch.rs`
  - Line: 89
  - Resolution: custom 維持
  - Note: `CommandSpec` に `run_as: None` と `sandbox_permissions` を含め、最小環境の apply_patch 実行を維持した。

- File: `codex-rs/core/src/tools/runtimes/unified_exec.rs`
  - Line: 213 / 262
  - Resolution: custom 維持
  - Note: zsh-fork/直実行の両方を `build_command_spec` + `env_for(spec, ...)` に揃え、worker user 伝搬を残した。

- File: `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs`
  - Line: 111 / 1049
  - Resolution: custom 維持
  - Note: zsh-fork の事前変換と再実行用 `CommandSpec` を custom 版に戻した。

- File: `codex-rs/exec/src/lib.rs`
  - Line: 63 / 201 / 515 / 542
  - Resolution: custom 維持
  - Note: `read_session_meta_line` と `progress_cursor` を残し、resume の config 再構成フローを custom 版に揃えた。

- File: `codex-rs/sandboxing/src/manager_tests.rs`
  - Line: 80 / 126 / 190
  - Resolution: custom 維持
  - Note: `SandboxCommand` 初期化に `expiration` / `capture_policy` / `run_as` / `sandbox_permissions` を追加し、custom 版のテスト入力に揃えた。
