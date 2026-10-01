# MCP ツール単位の LLM ストリーム待機制御

## 目的

MCP ツールの実行中に、同じ LLM レスポンスストリームが追加出力を生成し続けないようにする。長時間実行される MCP コマンドについて、MCP の完了後に次の LLM 推論へ進む設定をツール単位で指定できるようにする。

## 設定

```toml
[mcp_servers.my_server.tools.long_running_command]
wait_for_mcp_tool_completion = true
```

`false`（既定値）では現在の挙動を維持し、`true` の MCP ツールでは最初のツール呼び出しを受け取った時点で LLM ストリームの読み取りを止める。その後、既存の `drain_in_flight()` で MCP の完了を待ち、結果を履歴へ記録してから必要な次の LLM リクエストを開始する。

## 対象範囲と注意点

- MCP ツールごとの長時間実行・LLM ストリーム継続制御を対象とする。
- MCP サーバーの起動停止、MCP ツール自体のタイムアウト、`supports_parallel_tool_calls` の意味は変更しない。
- `supports_parallel_tool_calls` はツール実行の並列性を制御するだけで、LLM ストリームの継続を止めない。
- `wait_for_mcp_tool_completion` は、LLM がすでに生成したトークンを取り消すものではない。MCP 呼び出し以降のストリーム消費を止める制御である。
- 設定追加時は `codex-rs/core/config.schema.json` を更新する。

## 実装状況

実装済み。設定は MCP ツール一覧から `ToolInfo`、`McpHandler`、`ToolRouter` を経由してサンプリングループへ伝播し、対象ツールの Future を登録した直後に同じ Responses ストリームの読み取りを止める。

## 動作確認

- 設定値がサーバー・ツール単位で読み込まれることを確認する。
- `true` の MCP モックが、ツール完了前に次の LLM リクエストを受けないことを確認する。
- `false` / 未指定の MCP ツールでは既存の並列動作を確認する。
- `codex-rs/core/tests/suite/rmcp_client.rs` と MCP 関連の core テストを実行する。

## つまずきと対処

- `supports_parallel_tool_calls = false` だけでは LLM ストリームの継続を止められない。並列実行の許可と、ツール完了まで次の出力を読まない制御を分けて実装する。
- 設定を MCP サーバー単位にすると不要なコマンドまで待機対象になるため、`tools.<tool>` の設定として保持する。
- 既存の `ToolInfo` 構築箇所を増やさないため、待機ポリシーは MCP ツール一覧の露出処理から `ToolRouter` へ明示的に渡す。

## 関連ファイル

- `codex-rs/config/src/mcp_types.rs`
- `codex-rs/codex-mcp/src/server.rs`
- `codex-rs/codex-mcp/src/connection_manager.rs`
- `codex-rs/codex-mcp/src/tools.rs`
- `codex-rs/core/src/tools/router.rs`
- `codex-rs/core/src/tools/registry.rs`
- `codex-rs/core/src/tools/parallel.rs`
- `codex-rs/core/src/tools/spec_plan.rs`
- `codex-rs/core/src/tools/handlers/mcp.rs`
- `codex-rs/core/src/session/turn.rs`
