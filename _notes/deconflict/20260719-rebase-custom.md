# 20260719-rebase-custom

- File: `codex-rs/core/src/tools/registry.rs`
  - Line: 403
  - Resolution: 手動マージ
  - Note: 上流の `dispatch_any_with_terminal_outcome` 本体を維持し、custom 側で追加した MCP ツール完了待ち判定の `waits_for_mcp_tool_completion` と、その既存 dispatch API の互換入口 `dispatch_any` を残した。
