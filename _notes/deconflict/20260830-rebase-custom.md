# Rebase conflict resolution — 2026-08-30

## Context

- Upstream base: `63d213884daea50e4f74efc192cdc44f549b67d5`
- Custom commit being replayed: `2a64f1e907`
- The worktree was left in the interactive-rebase conflict state so the caller can complete the index operation separately. `git add` and `git rebase --continue` were intentionally not run.

## Resolution policy

Upstream changes were retained where the surrounding API had moved, while the custom behavior was carried forward at the new extension points:

- kept the upstream code-mode transport removal and retained the custom retry for a busy executable;
- merged upstream MCP `output_token_limit` handling with custom `wait_for_mcp_tool_completion` configuration and serialization;
- kept the upstream `StepSettingsUpdate` session shape and carried custom `project_doc_paths` through it;
- kept the upstream `Instructions`/two-argument agents-manager refresh API and custom project-document cache invalidation;
- retained custom `ContentItemKinds` as an under-development feature disabled by default;
- retained custom TUI command/feature wiring alongside upstream recap and terminal-title behavior;
- retained both upstream `runtimeStatus` and custom protocol source changes for `projectDocPaths`, and merged `projectDocPaths` into the current upstream-generated compressed experimental schema without importing stale generated files from the older custom base.

All textual Rust conflict markers were removed. The generated compressed schema now contains both `runtimeStatus` and `projectDocPaths`.

## Validation

- `cargo fmt --all -- --check` via the configured MCP executor: passed.
- Workspace `cargo check` via the configured MCP executor: passed.
- Focused MCP tests for MCP wait configuration and project-document refresh: passed.
- The first post-clean MCP `make almost` exposed a missing `wait_for_mcp_tool_completion` field in an MCP test fixture; the fixture was updated and the exact test passed.
- The next MCP `make almost` exposed an upstream Cloud Tasks test that conflicts with the custom CLI policy; the upstream test remains registered but ignored, and custom coverage verifies that `codex cloud list` stays unavailable.
- The following MCP `make almost` reached `codex-core --test all`; only the two legacy grandchild context-baseline cases timed out under parallel load. Both exact MCP individual tests passed, so they were added to the root `flaky_test_list.txt`.
- The next MCP `make almost` exposed two managed-proxy tests failing because the nested sandbox could not resolve the fixed `CARGO_BIN_EXE_codex-linux-sandbox` path. The test helper now derives the sibling sandbox executable from the integration test's `current_exe()` and asserts that it exists; no fallback path is used. Both exact MCP individual tests passed, followed by a passing MCP `make almost`.
- That full run then exposed the parallel-load-only `legacy_last_turn` grandchild context-baseline timeout. Its exact MCP individual test passed, so it was added to the same flaky-test block; the final MCP `make almost` passed.
- `git add` and `git rebase --continue` remain intentionally unrun.
