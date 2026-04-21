use super::*;
use codex_api::ApiError;
use codex_api::TransportError;
use pretty_assertions::assert_eq;

#[test]
fn map_api_error_maps_server_overloaded() {
    let err = map_api_error(ApiError::ServerOverloaded);
    assert!(matches!(err, CodexErr::ServerOverloaded));
}

#[test]
fn core_auth_provider_reports_auth_header_attached() {
    let auth = CoreAuthProvider {
        token: Some("access-token".to_string()),
        account_id: None,
        is_fedramp_account: false,
    };

    assert!(auth.auth_header_attached());
    assert_eq!(auth.auth_header_name(), Some("authorization"));
}

#[test]
fn map_api_error_maps_retryable_http_status() {
    let err = map_api_error(ApiError::Transport(TransportError::Http {
        status: http::StatusCode::SERVICE_UNAVAILABLE,
        url: Some("http://example.com/v1/responses".to_string()),
        headers: None,
        body: Some(
            serde_json::json!({
                "error": {
                    "code": "server_is_overloaded"
                }
            })
            .to_string(),
        ),
    }));

    assert!(matches!(err, CodexErr::ServerOverloaded));
}
