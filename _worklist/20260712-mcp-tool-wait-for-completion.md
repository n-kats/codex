# MCP ツール単位の LLM ストリーム待機制御

## 背景

現在の `try_run_sampling_request` は、MCP の Future を `in_flight` に追加した後も同じ Responses ストリームを読み続ける。長時間 MCP の実行中にメイン LLM の出力が進むため、MCP を待ってから次の推論へ進む制御が必要。

## 実装内容

1. `McpServerToolConfig` に `wait_for_mcp_tool_completion` を追加した。
2. `McpServerMetadata` から待機対象ツール名を構築し、`ToolInfo` へ反映するようにした。
3. `McpHandler` / `ToolRegistry` / `ToolRouter` 経由で待機設定を参照できるようにした。
4. `try_run_sampling_request` で対象ツールの Future を登録した直後にストリームを打ち切り、既存の `drain_in_flight` 後に `needs_follow_up` を返すようにした。
5. 設定読み込み、シリアライズ、スキーマ、core MCP 回帰テストを追加・更新した。

## 完了条件

- [x] 設定を指定した MCP ツールの完了前に、次の LLM ストリーム読み取りが発生しない。
- [x] 未指定のツールの既存動作を壊さない。
- [x] MCP の設定・スキーマ・core テストが通る。

## 検証結果

- MCP 経由 `codex-config` の `tool_stream_wait` テスト: 1 passed
- MCP 経由 `codex-core` の `mcp` フィルタテスト: 195 passed
- MCP 経由 `cargo fmt --all -- --check`: passed
