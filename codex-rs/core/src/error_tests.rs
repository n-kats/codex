use super::*;

#[test]
fn codex_error_retryability_matches_sandbox_errors() {
    assert!(!CodexErr::TurnAborted.is_retryable());
    assert!(CodexErr::Stream("retry me".to_string(), None).is_retryable());
    assert!(!CodexErr::Sandbox(SandboxErr::LandlockRestrict).is_retryable());
}

#[test]
fn codex_error_message_for_user_contains_original_text() {
    let message = get_error_message_ui(&CodexErr::InvalidRequest("bad request".to_string()));
    assert!(message.contains("bad request"));
}
