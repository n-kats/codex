# 20260722-rebase-custom

- File: `codex-rs/Cargo.lock`
  - Line: 2424
  - Resolution: custom 維持
  - Note: custom 側で除去している Cloud Tasks の `codex-cloud-tasks` / client / mock-client パッケージを再追加せず、上流の後続パッケージだけを残した。

- File: `codex-rs/codex-mcp/src/connection_manager.rs`
  - Line: 705
  - Resolution: 手動マージ
  - Note: 上流で `tool_catalog.rs` へ移動した `with_server_metadata` helper は本体側から削除し、custom の `wait_for_mcp_tool_completion` 設定は移動先 helper に再適用した。

- File: `codex-rs/codex-mcp/src/connection_manager/tool_catalog.rs`
  - Line: 265
  - Resolution: custom 維持
  - Note: MCP サーバー metadata の tool 反映時に、custom の `wait_for_mcp_tool_completion` 判定を維持した。

- File: `codex-rs/core/src/util.rs`
  - Line: 21
  - Resolution: custom 維持
  - Note: feedback tags macro の custom 側ドキュメント例を維持した。

- File: `codex-rs/deny.toml`
  - Line: 245
  - Resolution: 手動マージ
  - Note: 上流で不要になった `codex-core-plugins` の reqwest wrapper 例外は削除し、custom 側で必要な `codex-app-server-daemon` と `codex-backend-client` は残した。
