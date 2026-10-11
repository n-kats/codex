#![allow(non_snake_case)]

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn custom__mcp_tool_wait__deserializes_server_config_with_tool_stream_wait() {
    let cfg: McpServerConfig = toml::from_str(
        r#"
            command = "echo"

            [tools.search]
            wait_for_mcp_tool_completion = true
        "#,
    )
    .expect("should deserialize MCP tool stream wait setting");

    assert_eq!(
        cfg.tools.get("search"),
        Some(&McpServerToolConfig {
            approval_mode: None,
            output_token_limit: None,
            wait_for_mcp_tool_completion: true,
        })
    );

    let serialized = toml::to_string(&cfg).expect("should serialize MCP tool stream wait setting");
    assert!(serialized.contains("wait_for_mcp_tool_completion = true"));
}
