# 20260824 rebase custom

- File: `codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst`
  - Location: generated binary fixture; Git reported a stage 1/2/3 conflict.
  - Resolution: manually merged the generated exports from upstream stage 2 and custom stage 3. Preserved upstream `McpServerConnectionStatus` / `runtimeStatus` and added custom `projectDocPaths` exports.
  - Note: the repository regeneration helper could not run because `cargo` is unavailable in the agent PATH, so the compressed fixture was reconstructed from the two Git stages without dropping either side.
  - Verification: MCP `schema_fixtures_tests::experimental_precomputed_exports_match_generated` passed.
