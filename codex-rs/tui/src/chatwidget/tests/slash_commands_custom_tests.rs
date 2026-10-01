#![allow(non_snake_case)]

use super::*;
use assert_matches::assert_matches;

#[tokio::test]
async fn custom__slash_command__custom_agents_sends_project_doc_paths_override() {
    let (mut chat, _rx, mut op_rx) = make_chatwidget_manual(Some("gpt-5.4")).await;
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let agents_path = temp_dir.path().join("CUSTOM_AGENTS.md");
    std::fs::write(&agents_path, "custom instructions").expect("write custom agents");
    chat.config.cwd = AbsolutePathBuf::from_absolute_path(temp_dir.path()).expect("absolute cwd");
    chat.current_cwd = Some(temp_dir.path().to_path_buf());

    chat.dispatch_command_with_args(
        SlashCommand::CustomAgents,
        "CUSTOM_AGENTS.md".to_string(),
        Vec::new(),
    );

    let expected_path = agents_path.canonicalize().expect("canonical path");
    assert_matches!(
        op_rx.try_recv(),
        Ok(Op::OverrideTurnContext {
            project_doc_paths: Some(project_doc_paths),
            ..
        }) if project_doc_paths == vec![expected_path]
    );
    assert_matches!(op_rx.try_recv(), Err(TryRecvError::Empty));
}

#[tokio::test]
async fn custom__slash_command__custom_agents_clear_restores_auto_discovery() {
    let (mut chat, _rx, mut op_rx) = make_chatwidget_manual(Some("gpt-5.4")).await;

    chat.dispatch_command_with_args(SlashCommand::CustomAgents, "clear".to_string(), Vec::new());

    assert_matches!(
        op_rx.try_recv(),
        Ok(Op::OverrideTurnContext {
            project_doc_paths: Some(project_doc_paths),
            ..
        }) if project_doc_paths.is_empty()
    );
    assert_matches!(op_rx.try_recv(), Err(TryRecvError::Empty));
}
