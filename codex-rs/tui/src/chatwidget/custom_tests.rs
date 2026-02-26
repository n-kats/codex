#![allow(non_snake_case)]
use crate::app_event::AppEvent;
use crate::chatwidget::tests::make_chatwidget_manual_with_sender;
use codex_protocol::protocol::Op;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use pretty_assertions::assert_eq;
use tempfile::tempdir;
use tokio::sync::mpsc::error::TryRecvError;

fn next_override_turn_context_event(rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>) -> Op {
    loop {
        match rx.try_recv() {
            Ok(AppEvent::CodexOp(op @ Op::OverrideTurnContext { .. })) => return op,
            Ok(_) => continue,
            Err(TryRecvError::Empty) => {
                panic!("expected OverrideTurnContext app event but queue was empty")
            }
            Err(TryRecvError::Disconnected) => {
                panic!("expected OverrideTurnContext app event but channel closed")
            }
        }
    }
}

#[tokio::test]
async fn custom__custom_agents__slash_custom_agents_指定パスをsessionのproject_doc_pathsへ反映する()
{
    let (mut chat, _app_event_tx, mut rx, _op_rx) = make_chatwidget_manual_with_sender().await;
    let tempdir = tempdir().expect("tempdir");
    let custom_doc = tempdir.path().join("docs").join("AGENTS.override.md");
    std::fs::create_dir_all(custom_doc.parent().expect("parent")).expect("create docs dir");
    std::fs::write(&custom_doc, "custom").expect("write custom doc");
    chat.config.cwd = tempdir.path().to_path_buf();

    chat.bottom_pane.set_composer_text(
        "/custom-agents docs/AGENTS.override.md".to_string(),
        Vec::new(),
        Vec::new(),
    );
    chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    let op = next_override_turn_context_event(&mut rx);
    assert_eq!(
        op,
        Op::OverrideTurnContext {
            cwd: None,
            approval_policy: None,
            sandbox_policy: None,
            windows_sandbox_level: None,
            model: None,
            effort: None,
            summary: None,
            collaboration_mode: None,
            personality: None,
            project_doc_paths: Some(Some(vec![custom_doc])),
        }
    );
}

#[tokio::test]
async fn custom__custom_agents__slash_custom_agents_clear_自動探索へ戻す() {
    let (mut chat, _app_event_tx, mut rx, _op_rx) = make_chatwidget_manual_with_sender().await;

    chat.bottom_pane
        .set_composer_text("/custom-agents clear".to_string(), Vec::new(), Vec::new());
    chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    let op = next_override_turn_context_event(&mut rx);
    assert_eq!(
        op,
        Op::OverrideTurnContext {
            cwd: None,
            approval_policy: None,
            sandbox_policy: None,
            windows_sandbox_level: None,
            model: None,
            effort: None,
            summary: None,
            collaboration_mode: None,
            personality: None,
            project_doc_paths: Some(None),
        }
    );
}

#[test]
fn custom__custom_agents__custom_testsモジュールがコンパイルできる() {
    // Touch AppEvent so imports stay honest.
    let _ = std::mem::size_of::<AppEvent>();
}
