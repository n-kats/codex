#![allow(non_snake_case)]

use assert_matches::assert_matches;
use codex_protocol::protocol::Op;
use pretty_assertions::assert_eq;
use tokio::sync::mpsc::error::TryRecvError;

use crate::app_event::AppEvent;
use crate::chatwidget::tests::make_chatwidget_manual_with_sender;

#[tokio::test]
async fn custom__slash_compact__compact_opをemitしconfigをmutateしない() {
    let (mut chat, _app_event_tx, mut rx, mut op_rx) = make_chatwidget_manual_with_sender().await;
    let original_config = chat.config.clone();

    chat.dispatch_command(crate::slash_command::SlashCommand::Compact);

    assert_eq!(chat.config, original_config);
    assert_matches!(rx.try_recv(), Ok(AppEvent::CodexOp(Op::Compact)));
    assert_matches!(rx.try_recv(), Err(TryRecvError::Empty));
    assert_matches!(op_rx.try_recv(), Err(TryRecvError::Empty));
}

#[tokio::test]
async fn custom__slash_new__NewSessionをemitしconfigをmutateしない() {
    let (mut chat, _app_event_tx, mut rx, _op_rx) = make_chatwidget_manual_with_sender().await;
    let original_config = chat.config.clone();

    chat.dispatch_command(crate::slash_command::SlashCommand::New);

    assert_eq!(chat.config, original_config);
    assert_matches!(rx.try_recv(), Ok(AppEvent::NewSession));
    assert_matches!(rx.try_recv(), Err(TryRecvError::Empty));
}

#[tokio::test]
async fn custom__slash_setup_views__popupを開くだけでconfigをmutateしない() {
    for command in [
        crate::slash_command::SlashCommand::Approvals,
        crate::slash_command::SlashCommand::Permissions,
        crate::slash_command::SlashCommand::Statusline,
        crate::slash_command::SlashCommand::Theme,
    ] {
        let (mut chat, _app_event_tx, mut rx, _op_rx) = make_chatwidget_manual_with_sender().await;
        let original_config = chat.config.clone();

        chat.dispatch_command(command);

        assert_eq!(
            chat.config, original_config,
            "command {command:?} mutated config"
        );
        assert!(
            chat.bottom_pane.has_active_view(),
            "command {command:?} should open a popup or setup view"
        );
        assert_matches!(rx.try_recv(), Err(TryRecvError::Empty));
    }
}

#[tokio::test]
async fn custom__slash_switching__設定が吹き飛ばないことを横断確認() {
    for command in [
        crate::slash_command::SlashCommand::New,
        crate::slash_command::SlashCommand::Resume,
        crate::slash_command::SlashCommand::Fork,
        crate::slash_command::SlashCommand::Compact,
        crate::slash_command::SlashCommand::Model,
        crate::slash_command::SlashCommand::Personality,
        crate::slash_command::SlashCommand::Approvals,
        crate::slash_command::SlashCommand::Permissions,
        crate::slash_command::SlashCommand::Statusline,
        crate::slash_command::SlashCommand::Theme,
        crate::slash_command::SlashCommand::Collab,
        crate::slash_command::SlashCommand::Plan,
    ] {
        let (mut chat, _app_event_tx, mut rx, mut op_rx) =
            make_chatwidget_manual_with_sender().await;
        chat.thread_id = Some(codex_protocol::ThreadId::new());
        chat.set_feature_enabled(codex_core::features::Feature::CollaborationModes, true);
        let original_config = chat.config.clone();

        chat.dispatch_command(command);

        assert_eq!(
            chat.config, original_config,
            "command {command:?} mutated config"
        );
        while rx.try_recv().is_ok() {}
        while op_rx.try_recv().is_ok() {}
    }
}

#[cfg(not(target_os = "linux"))]
#[tokio::test]
async fn custom__slash_settings__popupを開くだけでconfigをmutateしない() {
    let (mut chat, _app_event_tx, mut rx, mut op_rx) = make_chatwidget_manual_with_sender().await;
    chat.set_feature_enabled(codex_core::features::Feature::RealtimeConversation, true);
    let original_config = chat.config.clone();

    chat.dispatch_command(crate::slash_command::SlashCommand::Settings);

    assert_eq!(chat.config, original_config);
    assert!(chat.bottom_pane.has_active_view());
    assert_matches!(rx.try_recv(), Err(TryRecvError::Empty));
    assert_matches!(op_rx.try_recv(), Err(TryRecvError::Empty));
}

#[cfg(not(target_os = "linux"))]
#[tokio::test]
async fn custom__slash_realtime__start_opをemitしconfigをmutateしない() {
    let (mut chat, _app_event_tx, mut rx, mut op_rx) = make_chatwidget_manual_with_sender().await;
    chat.set_feature_enabled(codex_core::features::Feature::RealtimeConversation, true);
    let original_config = chat.config.clone();

    chat.dispatch_command(crate::slash_command::SlashCommand::Realtime);

    assert_eq!(chat.config, original_config);
    assert_matches!(rx.try_recv(), Err(TryRecvError::Empty));
    assert_matches!(op_rx.try_recv(), Ok(Op::RealtimeConversationStart(_)));
}

#[cfg(not(target_os = "linux"))]
#[tokio::test]
async fn custom__slash_realtime__close_opをemitしconfigをmutateしない() {
    let (mut chat, _app_event_tx, mut rx, mut op_rx) = make_chatwidget_manual_with_sender().await;
    chat.set_feature_enabled(codex_core::features::Feature::RealtimeConversation, true);

    chat.realtime_conversation.phase =
        crate::chatwidget::realtime::RealtimeConversationPhase::Active;
    let original_config = chat.config.clone();

    chat.dispatch_command(crate::slash_command::SlashCommand::Realtime);

    assert_eq!(chat.config, original_config);
    assert_matches!(rx.try_recv(), Err(TryRecvError::Empty));
    assert_matches!(op_rx.try_recv(), Ok(Op::RealtimeConversationClose));
}

#[tokio::test]
async fn custom__slash_model__modelとeffort以外の設定を触らない() {
    use codex_protocol::openai_models::ModelPreset;
    use codex_protocol::openai_models::ReasoningEffort as ReasoningEffortConfig;
    use codex_protocol::openai_models::ReasoningEffortPreset;
    use codex_protocol::openai_models::default_input_modalities;

    let (mut chat, _app_event_tx, mut rx, mut op_rx) = make_chatwidget_manual_with_sender().await;
    chat.thread_id = Some(codex_protocol::ThreadId::new());

    let preset = ModelPreset {
        id: "codex-auto-fast-custom".to_string(),
        model: "codex-auto-fast-custom".to_string(),
        display_name: "codex-auto-fast-custom".to_string(),
        description: "Custom auto model used for regression coverage".to_string(),
        default_reasoning_effort: ReasoningEffortConfig::Low,
        supported_reasoning_efforts: vec![ReasoningEffortPreset {
            effort: ReasoningEffortConfig::Low,
            description: "Low reasoning".to_string(),
        }],
        supports_personality: false,
        is_default: false,
        upgrade: None,
        show_in_picker: true,
        availability_nux: None,
        supported_in_api: true,
        input_modalities: default_input_modalities(),
    };

    chat.open_model_popup_with_presets(vec![preset]);
    chat.handle_key_event(crossterm::event::KeyEvent::from(
        crossterm::event::KeyCode::Enter,
    ));

    let events = std::iter::from_fn(|| rx.try_recv().ok()).collect::<Vec<_>>();
    assert!(
        events.iter().any(|event| matches!(event, AppEvent::UpdateModel(model) if model == "codex-auto-fast-custom")),
        "expected model update event; events: {events:?}"
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            AppEvent::UpdateReasoningEffort(Some(ReasoningEffortConfig::Low))
        )),
        "expected reasoning update event; events: {events:?}"
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            AppEvent::PersistModelSelection { model, effort: Some(ReasoningEffortConfig::Low) }
                if model == "codex-auto-fast-custom"
        )),
        "expected persistence event; events: {events:?}"
    );
    assert!(
        events.iter().all(|event| !matches!(
            event,
            AppEvent::UpdatePersonality(_)
                | AppEvent::UpdateCollaborationMode(_)
                | AppEvent::UpdateAskForApprovalPolicy(_)
                | AppEvent::UpdateSandboxPolicy(_)
                | AppEvent::UpdateApprovalsReviewer(_)
        )),
        "expected model selection to leave other settings untouched; events: {events:?}"
    );
    assert_matches!(op_rx.try_recv(), Err(TryRecvError::Empty));
}

#[tokio::test]
async fn custom__slash_personality__personality更新と永続化を行う() {
    let (mut chat, _app_event_tx, mut rx, mut op_rx) = make_chatwidget_manual_with_sender().await;
    chat.thread_id = Some(codex_protocol::ThreadId::new());
    chat.set_feature_enabled(codex_core::features::Feature::Personality, true);
    chat.set_personality(codex_protocol::config_types::Personality::Friendly);

    chat.open_personality_popup();
    chat.handle_key_event(crossterm::event::KeyEvent::from(
        crossterm::event::KeyCode::Down,
    ));
    chat.handle_key_event(crossterm::event::KeyEvent::from(
        crossterm::event::KeyCode::Enter,
    ));

    let events = std::iter::from_fn(|| rx.try_recv().ok()).collect::<Vec<_>>();
    assert!(
        events.iter().any(|event| matches!(
            event,
            AppEvent::CodexOp(Op::OverrideTurnContext {
                personality: Some(codex_protocol::config_types::Personality::Pragmatic),
                ..
            })
        )),
        "expected personality override op; events: {events:?}"
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            AppEvent::UpdatePersonality(codex_protocol::config_types::Personality::Pragmatic)
        )),
        "expected personality update event; events: {events:?}"
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            AppEvent::PersistPersonalitySelection {
                personality: codex_protocol::config_types::Personality::Pragmatic,
            }
        )),
        "expected personality persistence event; events: {events:?}"
    );
    assert_matches!(op_rx.try_recv(), Err(TryRecvError::Empty));
}
