# 20260808 rebase custom

- File: `codex-rs/app-server-protocol/schema/precomputed/app-server-exports-experimental.json.zst`
  - Line: binary file
  - Resolution: upstream 優先 + custom 差分を再適用
  - Note: 上流の experimental schema export を基礎にし、custom の `ThreadSettingsUpdateParams.project_doc_paths` に対応する `projectDocPaths` を4つの JSON export と TypeScript export に追加した。custom 側の古い export 全体は採用せず、上流で追加された schema 定義を保持した。

- File: `codex-rs/config/src/loader/tests.rs`
  - Line: 269-507
  - Resolution: 手動マージ
  - Note: custom の local layer projection / legacy requirements テストを保持し、上流の `ignore_project_config_skips_project_layers` テストを後ろに追加した。

- File: `codex-rs/core-skills/src/loader_tests.rs`
  - Line: file
  - Resolution: custom 維持
  - Note: 上流側では `core-skills` のこのファイルが削除されていたが、custom 側の skills crate が依存するテストファイルなので、作業ツリーの custom 版を保持した。

- File: `codex-rs/core/src/tools/handlers/mcp.rs`
  - Line: 239-258
  - Resolution: 手動マージ
  - Note: 上流の `mcp_server_name` を残し、custom の MCP 完了待機フックと非同期 `telemetry_tags` を併存させた。
