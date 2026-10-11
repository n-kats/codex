# 2026-03-03 Rebase Custom Report

- Rebase base: local `fork-origin/main` (`c2d008aca59697aff3a65949e7ffed06bd715dea`)
- Pre-rebase snapshot branch: `20260303`
- Comparison branch: `tmp-rebase`
- Note: `git fetch fork-origin` failed in this environment because `github.com` could not be resolved, so this rebase used the locally available `fork-origin/main`.

## Range-diff review

- Reviewed: `git range-diff a0e86c69fe35e8ccfbed7ed87f07914d467ec488..tmp-rebase c2d008aca59697aff3a65949e7ffed06bd715dea..custom`
- Summary: one `!` patch (`custom changes` -> `custom changes`) due conflict resolution while moving the squashed custom patch onto the newer base.
- Main conflict resolution areas:
  - `codex-rs/app-server/src/codex_message_processor.rs`
  - `codex-rs/cli/src/mcp_cmd.rs`
  - `codex-rs/core/src/codex.rs`
  - `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs`
  - `codex-rs/mcp-server/src/lib.rs`
  - `codex-rs/tui/src/diff_render.rs`

## Custom spec checklist

- `CODEX_ADDITIONAL_PROMPT_DIRS`
  - 実装: `codex-rs/core/src/custom_prompts.rs:8`, `codex-rs/core/src/custom_prompts.rs:27`
  - テスト: `codex-rs/core/src/custom_prompts.rs:195`
  - 検証: `make verify-additional-prompt-dirs-env`

- Custom diff palette override
  - 実装: `codex-rs/tui/src/diff_render.rs:171`, `codex-rs/tui/src/diff_render.rs:324`
  - テスト: `codex-rs/tui/src/diff_render/custom_tests.rs:51`
  - 検証: `cd codex-rs && cargo test -p codex-tui custom__差分テーマ色__`

- MCP `--no-config` / `--config-toml-file` loader overrides
  - 実装: `codex-rs/cli/src/mcp_cmd.rs:42`, `codex-rs/cli/src/mcp_cmd.rs:172`
  - テスト: なし
  - 検証: `cd codex-rs && cargo test -p codex-cli`

- Worker-user command execution
  - 実装: `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs:13`, `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs:639`
  - テスト: `codex-rs/core/tests/suite/custom_exec_command_worker_user.rs:64`
  - 検証: `make verify-command-exec-worker-user`

- Shell startup file isolation
  - 実装: `codex-rs/core/src/shell_startup_files.rs:34`, `codex-rs/core/src/shell_startup_files.rs:62`
  - テスト: `codex-rs/core/src/shell_startup_files/custom_tests.rs:42`
  - 検証: `make verify-linux-default-shell`

- Custom agents / project doc override
  - 実装: `codex-rs/tui/src/chatwidget.rs:3982`, `codex-rs/core/src/codex.rs:3947`
  - テスト: `codex-rs/tui/src/chatwidget/custom_tests.rs:27`, `codex-rs/core/src/codex/custom_tests.rs:10`
  - 検証: `cd codex-rs && cargo test custom__custom_agents__`

- Custom test segregation rule
  - 実装: `_docs/custom_notes/custom_tests/README.md:11`, `_docs/custom_notes/custom_tests/README.md:35`
  - テスト: `codex-rs/tui/src/diff_render/custom_tests.rs:51`, `codex-rs/core/src/codex/custom_tests.rs:10`
  - 検証: `cd codex-rs && cargo test custom__`
