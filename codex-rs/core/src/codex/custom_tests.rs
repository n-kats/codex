#![allow(non_snake_case)]
use super::SessionSettingsUpdate;
use super::handlers;
use super::make_session_and_context_with_rx;

#[tokio::test]
async fn custom__custom_agents__override_turn_context_project_doc_paths変更時にuser_instructionsを再生成する()
 {
    let (session, _turn_context, _rx) = make_session_and_context_with_rx().await;
    let cwd = tempfile::tempdir().expect("create temp dir");
    let auto_doc = cwd.path().join("AGENTS.md");
    let custom_doc = cwd.path().join("custom.md");
    std::fs::write(&auto_doc, "auto instructions").expect("write auto doc");
    std::fs::write(&custom_doc, "custom instructions").expect("write custom doc");

    handlers::override_turn_context(
        session.as_ref(),
        "override-custom".to_string(),
        SessionSettingsUpdate {
            cwd: Some(cwd.path().to_path_buf()),
            project_doc_paths: Some(Some(vec![custom_doc])),
            ..Default::default()
        },
    )
    .await;

    let after_custom = session.new_default_turn().await;
    let after_custom_instructions = after_custom
        .user_instructions
        .as_deref()
        .expect("user instructions should be present after custom override");
    assert!(after_custom_instructions.starts_with("custom instructions"));

    handlers::override_turn_context(
        session.as_ref(),
        "override-clear".to_string(),
        SessionSettingsUpdate {
            cwd: Some(cwd.path().to_path_buf()),
            project_doc_paths: Some(None),
            ..Default::default()
        },
    )
    .await;

    let after_clear = session.new_default_turn().await;
    let after_clear_instructions = after_clear
        .user_instructions
        .as_deref()
        .expect("user instructions should be present after clearing override");
    assert!(after_clear_instructions.starts_with("auto instructions"));
}
