use super::*;
use pretty_assertions::assert_eq;

#[test]
fn validate_mcp_server_name_accepts_simple_names() {
    assert!(validate_mcp_server_name("server-1").is_ok());
    assert!(validate_mcp_server_name("server_1").is_ok());
}

#[test]
fn validate_mcp_server_name_rejects_invalid_names() {
    assert!(validate_mcp_server_name("bad name").is_err());
    assert!(validate_mcp_server_name("bad/name").is_err());
}

#[test]
fn mcp_init_error_display_reports_auth_required() {
    let error = StartupOutcomeError::Failed {
        error: "Auth required for MCP server".to_string(),
    };
    let rendered = mcp_init_error_display("example", None, &error);
    assert_eq!(
        rendered,
        "The example MCP server is not logged in. Run `codex mcp login example`."
    );
}
