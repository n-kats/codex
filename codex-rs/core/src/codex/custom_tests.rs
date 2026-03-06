#![allow(non_snake_case)]
use super::SessionSettingsUpdate;
use super::handlers;
use super::make_session_and_context_with_rx;
use crate::protocol::CodexErrorInfo;
use crate::protocol::Event;
use crate::protocol::EventMsg;

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

#[tokio::test]
async fn custom__custom_agents__relative_pathは現在のsession_cwdで解決される() {
    let (session, _turn_context, _rx) = make_session_and_context_with_rx().await;
    let cwd = tempfile::tempdir().expect("create temp dir");
    let custom_doc = cwd.path().join("docs").join("AGENTS.override.md");
    std::fs::create_dir_all(custom_doc.parent().expect("parent")).expect("create docs dir");
    std::fs::write(&custom_doc, "custom instructions").expect("write custom doc");

    handlers::override_turn_context(
        session.as_ref(),
        "set-custom-agents".to_string(),
        SessionSettingsUpdate {
            cwd: Some(cwd.path().to_path_buf()),
            project_doc_paths: Some(Some(vec!["docs/AGENTS.override.md".into()])),
            ..Default::default()
        },
    )
    .await;

    let turn = session.new_default_turn().await;
    let instructions = turn
        .user_instructions
        .as_deref()
        .expect("user instructions should be present after custom override");
    assert!(instructions.starts_with("custom instructions"));
}

#[tokio::test]
async fn custom__custom_agents__override後は次ターンでfull_context再注入できるようbaselineをクリアする()
 {
    let (session, _turn_context, _rx) = make_session_and_context_with_rx().await;
    let cwd = tempfile::tempdir().expect("create temp dir");
    let custom_doc = cwd.path().join("custom.md");
    std::fs::write(&custom_doc, "custom instructions").expect("write custom doc");

    let turn_context = session.new_default_turn().await;
    {
        let mut state = session.state.lock().await;
        state.set_reference_context_item(Some(turn_context.to_turn_context_item()));
    }
    assert!(
        session.reference_context_item().await.is_some(),
        "expected baseline to exist before override"
    );

    handlers::override_turn_context(
        session.as_ref(),
        "set-custom-agents".to_string(),
        SessionSettingsUpdate {
            cwd: Some(cwd.path().to_path_buf()),
            project_doc_paths: Some(Some(vec![custom_doc])),
            ..Default::default()
        },
    )
    .await;

    assert!(
        session.reference_context_item().await.is_none(),
        "expected baseline cleared so next turn can inject updated user instructions"
    );
}

async fn wait_for_bad_request_error(rx: &async_channel::Receiver<Event>, sub_id: &str) -> String {
    let deadline = std::time::Duration::from_secs(2);
    let start = std::time::Instant::now();
    loop {
        let remaining = deadline.saturating_sub(start.elapsed());
        let evt = tokio::time::timeout(remaining, rx.recv())
            .await
            .expect("timeout waiting for event")
            .expect("event");
        if evt.id == sub_id
            && let EventMsg::Error(payload) = evt.msg
            && payload.codex_error_info == Some(CodexErrorInfo::BadRequest)
        {
            return payload.message;
        }
    }
}

#[tokio::test]
async fn custom__custom_agents__存在しないpathはbad_requestで拒否され既存設定を維持する() {
    let (session, _turn_context, rx) = make_session_and_context_with_rx().await;
    let cwd = tempfile::tempdir().expect("create temp dir");
    let custom_doc = cwd.path().join("custom.md");
    std::fs::write(&custom_doc, "custom instructions").expect("write custom doc");

    handlers::override_turn_context(
        session.as_ref(),
        "set-valid-custom".to_string(),
        SessionSettingsUpdate {
            cwd: Some(cwd.path().to_path_buf()),
            project_doc_paths: Some(Some(vec![custom_doc])),
            ..Default::default()
        },
    )
    .await;

    let before_invalid = session.new_default_turn().await;
    let before_invalid_instructions = before_invalid
        .user_instructions
        .as_deref()
        .expect("instructions should exist after valid custom-agents");
    assert!(before_invalid_instructions.starts_with("custom instructions"));

    let bad_sub_id = "invalid-missing-path";
    handlers::override_turn_context(
        session.as_ref(),
        bad_sub_id.to_string(),
        SessionSettingsUpdate {
            cwd: Some(cwd.path().to_path_buf()),
            project_doc_paths: Some(Some(vec!["missing.md".into()])),
            ..Default::default()
        },
    )
    .await;

    let error_message = wait_for_bad_request_error(&rx, bad_sub_id).await;
    assert!(
        error_message.contains("invalid /custom-agents path"),
        "unexpected error message: {error_message}"
    );

    let after_invalid = session.new_default_turn().await;
    let after_invalid_instructions = after_invalid
        .user_instructions
        .as_deref()
        .expect("instructions should remain after invalid custom-agents");
    assert!(after_invalid_instructions.starts_with("custom instructions"));
}

#[tokio::test]
async fn custom__custom_agents__directory指定はbad_requestで拒否され既存設定を維持する() {
    let (session, _turn_context, rx) = make_session_and_context_with_rx().await;
    let cwd = tempfile::tempdir().expect("create temp dir");
    let custom_doc = cwd.path().join("custom.md");
    std::fs::write(&custom_doc, "custom instructions").expect("write custom doc");

    handlers::override_turn_context(
        session.as_ref(),
        "set-valid-custom-dir-case".to_string(),
        SessionSettingsUpdate {
            cwd: Some(cwd.path().to_path_buf()),
            project_doc_paths: Some(Some(vec![custom_doc])),
            ..Default::default()
        },
    )
    .await;

    let before_invalid = session.new_default_turn().await;
    let before_invalid_instructions = before_invalid
        .user_instructions
        .as_deref()
        .expect("instructions should exist after valid custom-agents");
    assert!(before_invalid_instructions.starts_with("custom instructions"));

    let bad_sub_id = "invalid-directory-path";
    std::fs::create_dir_all(cwd.path().join("docs")).expect("create docs dir");
    handlers::override_turn_context(
        session.as_ref(),
        bad_sub_id.to_string(),
        SessionSettingsUpdate {
            cwd: Some(cwd.path().to_path_buf()),
            project_doc_paths: Some(Some(vec!["docs".into()])),
            ..Default::default()
        },
    )
    .await;

    let error_message = wait_for_bad_request_error(&rx, bad_sub_id).await;
    assert!(
        error_message.contains("invalid /custom-agents path (not a file)"),
        "unexpected error message: {error_message}"
    );

    let after_invalid = session.new_default_turn().await;
    let after_invalid_instructions = after_invalid
        .user_instructions
        .as_deref()
        .expect("instructions should remain after invalid custom-agents");
    assert!(after_invalid_instructions.starts_with("custom instructions"));
}
