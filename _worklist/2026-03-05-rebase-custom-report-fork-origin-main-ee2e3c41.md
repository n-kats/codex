# 2026-03-05 Rebase Custom Report (fork-origin/main `ee2e3c41`)

- Rebase base: local `fork-origin/main` (`ee2e3c415b93207695fcc0b7eb07c9b3777164e2`)
- Pre-rebase snapshot branch: `20260305`
- Comparison branch used: `tmp-rebase`
- Note: network access is restricted in this environment, so this rebase used the locally available `fork-origin/main`.

## Range-diff review

- Reviewed:
  - `git range-diff b200a5f45bf6a81b16c7b2e6f04dbe5eb172863c..tmp-rebase ee2e3c415b93207695fcc0b7eb07c9b3777164e2..custom`
- Summary:
  - one `!` patch (`custom changes` -> `custom changes`)
  - no manual conflict resolution was needed in this rebase run
  - patch drift is explained by upstream changes between `b200a5f45` and `ee2e3c415`

## Custom spec checklist

- `CODEX_ADDITIONAL_PROMPT_DIRS`
  - 実装: `codex-rs/core/src/custom_prompts.rs:8`, `codex-rs/core/src/custom_prompts.rs:24`
  - テスト: `codex-rs/core/src/custom_prompts/custom_tests.rs:11`
  - 検証: `make verify-additional-prompt-dirs-env`

- Custom diff palette override
  - 実装: `codex-rs/tui/src/diff_render.rs:171`, `codex-rs/tui/src/diff_render.rs:324`
  - テスト: `codex-rs/tui/src/diff_render/custom_tests.rs:52`
  - 検証: `cd codex-rs && cargo test -p codex-tui custom__差分テーマ色__`

- MCP `--no-config` / `--config-toml-file` loader overrides
  - 実装: `codex-rs/cli/src/mcp_cmd.rs:42`, `codex-rs/cli/src/mcp_cmd.rs:172`
  - テスト: なし
  - 検証: `cd codex-rs && cargo test -p codex-cli`

- Worker-user command execution
  - 実装: `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs:13`, `codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs:639`
  - テスト: `codex-rs/core/tests/suite/custom_exec_command_worker_user.rs:29`
  - 検証: `make verify-command-exec-worker-user`

- Shell startup file isolation
  - 実装: `codex-rs/core/src/shell_startup_files.rs:24`, `codex-rs/core/src/shell_startup_files.rs:34`
  - テスト: `codex-rs/core/src/shell_startup_files/custom_tests.rs:12`
  - 検証: `make verify-linux-default-shell`

- Custom agents / project doc override
  - 実装: `codex-rs/tui/src/chatwidget.rs:4172`, `codex-rs/core/src/codex.rs:3759`
  - テスト: `codex-rs/tui/src/chatwidget/custom_tests.rs:27`, `codex-rs/core/src/codex/custom_tests.rs:10`
  - 検証: `cd codex-rs && cargo test custom__custom_agents__`

- Custom test segregation rule
  - 実装: `_docs/custom_notes/custom_tests/README.md:10`, `_docs/custom_notes/custom_tests/README.md:26`
  - テスト: `codex-rs/tui/src/diff_render/custom_tests.rs:52`, `codex-rs/core/src/codex/custom_tests.rs:10`
  - 検証: `cd codex-rs && cargo test custom__`
