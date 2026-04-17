use crate::exec_command::split_command_string;
use codex_app_server_protocol::CodexErrorInfo as AppServerCodexErrorInfo;
use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::ThreadItem;
use codex_app_server_protocol::Turn;
use codex_app_server_protocol::TurnStatus;
use codex_protocol::ThreadId;
use codex_protocol::config_types::ModeKind;
use codex_protocol::items::AgentMessageContent;
use codex_protocol::items::AgentMessageItem;
use codex_protocol::items::ContextCompactionItem;
use codex_protocol::items::ImageGenerationItem;
use codex_protocol::items::PlanItem;
use codex_protocol::items::ReasoningItem;
use codex_protocol::items::TurnItem;
use codex_protocol::items::UserMessageItem;
use codex_protocol::items::WebSearchItem;
use codex_protocol::models::WebSearchAction;
use codex_protocol::protocol::AgentMessageDeltaEvent;
use codex_protocol::protocol::AgentReasoningDeltaEvent;
use codex_protocol::protocol::AgentReasoningRawContentDeltaEvent;
use codex_protocol::protocol::ErrorEvent;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::ExecCommandBeginEvent;
use codex_protocol::protocol::ExecCommandEndEvent;
use codex_protocol::protocol::ExecCommandOutputDeltaEvent;
use codex_protocol::protocol::ExecCommandStatus;
use codex_protocol::protocol::ExecOutputStream;
use codex_protocol::protocol::ItemCompletedEvent;
use codex_protocol::protocol::ItemStartedEvent;
use codex_protocol::protocol::PlanDeltaEvent;
use codex_protocol::protocol::RealtimeConversationClosedEvent;
use codex_protocol::protocol::RealtimeConversationRealtimeEvent;
use codex_protocol::protocol::RealtimeConversationStartedEvent;
use codex_protocol::protocol::RealtimeEvent;
use codex_protocol::protocol::ThreadNameUpdatedEvent;
use codex_protocol::protocol::TokenCountEvent;
use codex_protocol::protocol::TokenUsage;
use codex_protocol::protocol::TokenUsageInfo;
use codex_protocol::protocol::TurnAbortReason;
use codex_protocol::protocol::TurnAbortedEvent;
use codex_protocol::protocol::TurnCompleteEvent;
use codex_protocol::protocol::TurnStartedEvent;
use std::time::Duration;

pub(super) fn server_notification_thread_events(
    notification: ServerNotification,
) -> Option<(ThreadId, Vec<Event>)> {
    match notification {
        ServerNotification::ThreadTokenUsageUpdated(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            vec![Event {
                id: String::new(),
                msg: EventMsg::TokenCount(TokenCountEvent {
                    info: Some(TokenUsageInfo {
                        total_token_usage: token_usage_from_app_server(
                            notification.token_usage.total,
                        ),
                        last_token_usage: token_usage_from_app_server(
                            notification.token_usage.last,
                        ),
                        model_context_window: notification.token_usage.model_context_window,
                    }),
                    rate_limits: None,
                }),
            }],
        )),
        ServerNotification::Error(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            vec![Event {
                id: String::new(),
                msg: EventMsg::Error(ErrorEvent {
                    message: notification.error.message,
                    codex_error_info: notification
                        .error
                        .codex_error_info
                        .and_then(app_server_codex_error_info_to_core),
                }),
            }],
        )),
        ServerNotification::ThreadNameUpdated(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            vec![Event {
                id: String::new(),
                msg: EventMsg::ThreadNameUpdated(ThreadNameUpdatedEvent {
                    thread_id: ThreadId::from_string(&notification.thread_id).ok()?,
                    thread_name: notification.thread_name,
                }),
            }],
        )),
        ServerNotification::TurnStarted(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            vec![Event {
                id: String::new(),
                msg: EventMsg::TurnStarted(TurnStartedEvent {
                    turn_id: notification.turn.id,
                    model_context_window: None,
                    collaboration_mode_kind: ModeKind::default(),
                }),
            }],
        )),
        ServerNotification::TurnCompleted(notification) => {
            let thread_id = ThreadId::from_string(&notification.thread_id).ok()?;
            let mut events = Vec::new();
            append_terminal_turn_events(
                &mut events,
                &notification.turn,
                /*include_failed_error*/ false,
            );
            Some((thread_id, events))
        }
        ServerNotification::ItemStarted(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            command_execution_started_event(&notification.turn_id, &notification.item).or_else(
                || {
                    Some(vec![Event {
                        id: String::new(),
                        msg: EventMsg::ItemStarted(ItemStartedEvent {
                            thread_id: ThreadId::from_string(&notification.thread_id).ok()?,
                            turn_id: notification.turn_id.clone(),
                            item: thread_item_to_core(&notification.item)?,
                        }),
                    }])
                },
            )?,
        )),
        ServerNotification::ItemCompleted(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            command_execution_completed_event(&notification.turn_id, &notification.item).or_else(
                || {
                    Some(vec![Event {
                        id: String::new(),
                        msg: EventMsg::ItemCompleted(ItemCompletedEvent {
                            thread_id: ThreadId::from_string(&notification.thread_id).ok()?,
                            turn_id: notification.turn_id.clone(),
                            item: thread_item_to_core(&notification.item)?,
                        }),
                    }])
                },
            )?,
        )),
        ServerNotification::CommandExecutionOutputDelta(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            vec![Event {
                id: String::new(),
                msg: EventMsg::ExecCommandOutputDelta(ExecCommandOutputDeltaEvent {
                    call_id: notification.item_id,
                    stream: ExecOutputStream::Stdout,
                    chunk: notification.delta.into_bytes(),
                }),
            }],
        )),
        ServerNotification::AgentMessageDelta(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            vec![Event {
                id: String::new(),
                msg: EventMsg::AgentMessageDelta(AgentMessageDeltaEvent {
                    delta: notification.delta,
                }),
            }],
        )),
        ServerNotification::PlanDelta(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            vec![Event {
                id: String::new(),
                msg: EventMsg::PlanDelta(PlanDeltaEvent {
                    thread_id: notification.thread_id,
                    turn_id: notification.turn_id,
                    item_id: notification.item_id,
                    delta: notification.delta,
                }),
            }],
        )),
        ServerNotification::ReasoningSummaryTextDelta(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            vec![Event {
                id: String::new(),
                msg: EventMsg::AgentReasoningDelta(AgentReasoningDeltaEvent {
                    delta: notification.delta,
                }),
            }],
        )),
        ServerNotification::ReasoningTextDelta(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            vec![Event {
                id: String::new(),
                msg: EventMsg::AgentReasoningRawContentDelta(AgentReasoningRawContentDeltaEvent {
                    delta: notification.delta,
                }),
            }],
        )),
        ServerNotification::ThreadRealtimeStarted(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            vec![Event {
                id: String::new(),
                msg: EventMsg::RealtimeConversationStarted(RealtimeConversationStartedEvent {
                    session_id: notification.session_id,
                    version: notification.version,
                }),
            }],
        )),
        ServerNotification::ThreadRealtimeItemAdded(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            vec![Event {
                id: String::new(),
                msg: EventMsg::RealtimeConversationRealtime(RealtimeConversationRealtimeEvent {
                    payload: RealtimeEvent::ConversationItemAdded(notification.item),
                }),
            }],
        )),
        ServerNotification::ThreadRealtimeOutputAudioDelta(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            vec![Event {
                id: String::new(),
                msg: EventMsg::RealtimeConversationRealtime(RealtimeConversationRealtimeEvent {
                    payload: RealtimeEvent::AudioOut(notification.audio.into()),
                }),
            }],
        )),
        ServerNotification::ThreadRealtimeError(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            vec![Event {
                id: String::new(),
                msg: EventMsg::RealtimeConversationRealtime(RealtimeConversationRealtimeEvent {
                    payload: RealtimeEvent::Error(notification.message),
                }),
            }],
        )),
        ServerNotification::ThreadRealtimeClosed(notification) => Some((
            ThreadId::from_string(&notification.thread_id).ok()?,
            vec![Event {
                id: String::new(),
                msg: EventMsg::RealtimeConversationClosed(RealtimeConversationClosedEvent {
                    reason: notification.reason,
                }),
            }],
        )),
        _ => None,
    }
}

pub(super) fn app_server_turns_to_events(thread_id: ThreadId, turns: &[Turn]) -> Vec<Event> {
    turns
        .iter()
        .flat_map(|turn| app_server_turn_to_events(thread_id, turn))
        .collect()
}

fn token_usage_from_app_server(
    value: codex_app_server_protocol::TokenUsageBreakdown,
) -> TokenUsage {
    TokenUsage {
        input_tokens: value.input_tokens,
        cached_input_tokens: value.cached_input_tokens,
        output_tokens: value.output_tokens,
        reasoning_output_tokens: value.reasoning_output_tokens,
        total_tokens: value.total_tokens,
    }
}

fn app_server_codex_error_info_to_core(
    value: AppServerCodexErrorInfo,
) -> Option<codex_protocol::protocol::CodexErrorInfo> {
    Some(match value {
        AppServerCodexErrorInfo::ContextWindowExceeded => {
            codex_protocol::protocol::CodexErrorInfo::ContextWindowExceeded
        }
        AppServerCodexErrorInfo::UsageLimitExceeded => {
            codex_protocol::protocol::CodexErrorInfo::UsageLimitExceeded
        }
        AppServerCodexErrorInfo::ServerOverloaded => {
            codex_protocol::protocol::CodexErrorInfo::ServerOverloaded
        }
        AppServerCodexErrorInfo::HttpConnectionFailed { http_status_code } => {
            codex_protocol::protocol::CodexErrorInfo::HttpConnectionFailed { http_status_code }
        }
        AppServerCodexErrorInfo::ResponseStreamConnectionFailed { http_status_code } => {
            codex_protocol::protocol::CodexErrorInfo::ResponseStreamConnectionFailed {
                http_status_code,
            }
        }
        AppServerCodexErrorInfo::InternalServerError => {
            codex_protocol::protocol::CodexErrorInfo::InternalServerError
        }
        AppServerCodexErrorInfo::Unauthorized => {
            codex_protocol::protocol::CodexErrorInfo::Unauthorized
        }
        AppServerCodexErrorInfo::BadRequest => codex_protocol::protocol::CodexErrorInfo::BadRequest,
        AppServerCodexErrorInfo::ThreadRollbackFailed => {
            codex_protocol::protocol::CodexErrorInfo::ThreadRollbackFailed
        }
        AppServerCodexErrorInfo::SandboxError => {
            codex_protocol::protocol::CodexErrorInfo::SandboxError
        }
        AppServerCodexErrorInfo::ResponseStreamDisconnected { http_status_code } => {
            codex_protocol::protocol::CodexErrorInfo::ResponseStreamDisconnected {
                http_status_code,
            }
        }
        AppServerCodexErrorInfo::ResponseTooManyFailedAttempts { http_status_code } => {
            codex_protocol::protocol::CodexErrorInfo::ResponseTooManyFailedAttempts {
                http_status_code,
            }
        }
        AppServerCodexErrorInfo::ActiveTurnNotSteerable { turn_kind } => {
            codex_protocol::protocol::CodexErrorInfo::ActiveTurnNotSteerable {
                turn_kind: match turn_kind {
                    codex_app_server_protocol::NonSteerableTurnKind::Review => {
                        codex_protocol::protocol::NonSteerableTurnKind::Review
                    }
                    codex_app_server_protocol::NonSteerableTurnKind::Compact => {
                        codex_protocol::protocol::NonSteerableTurnKind::Compact
                    }
                },
            }
        }
        AppServerCodexErrorInfo::Other => codex_protocol::protocol::CodexErrorInfo::Other,
    })
}

fn append_terminal_turn_events(events: &mut Vec<Event>, turn: &Turn, include_failed_error: bool) {
    match turn.status {
        TurnStatus::Completed => events.push(Event {
            id: String::new(),
            msg: EventMsg::TurnComplete(TurnCompleteEvent {
                turn_id: turn.id.clone(),
                last_agent_message: None,
            }),
        }),
        TurnStatus::Interrupted => events.push(Event {
            id: String::new(),
            msg: EventMsg::TurnAborted(TurnAbortedEvent {
                turn_id: Some(turn.id.clone()),
                reason: TurnAbortReason::Interrupted,
            }),
        }),
        TurnStatus::Failed => {
            if include_failed_error && let Some(error) = &turn.error {
                events.push(Event {
                    id: String::new(),
                    msg: EventMsg::Error(ErrorEvent {
                        message: error.message.clone(),
                        codex_error_info: error
                            .codex_error_info
                            .clone()
                            .and_then(app_server_codex_error_info_to_core),
                    }),
                });
            }
            events.push(Event {
                id: String::new(),
                msg: EventMsg::TurnComplete(TurnCompleteEvent {
                    turn_id: turn.id.clone(),
                    last_agent_message: None,
                }),
            });
        }
        TurnStatus::InProgress => {}
    }
}

fn app_server_turn_to_events(thread_id: ThreadId, turn: &Turn) -> Vec<Event> {
    let mut events = Vec::new();
    events.push(Event {
        id: String::new(),
        msg: EventMsg::TurnStarted(TurnStartedEvent {
            turn_id: turn.id.clone(),
            model_context_window: None,
            collaboration_mode_kind: ModeKind::default(),
        }),
    });
    for item in &turn.items {
        events.extend(app_server_turn_item_to_events(thread_id, &turn.id, item));
    }
    append_terminal_turn_events(&mut events, turn, /*include_failed_error*/ true);
    events
}

fn app_server_turn_item_to_events(
    thread_id: ThreadId,
    turn_id: &str,
    item: &ThreadItem,
) -> Vec<Event> {
    if let Some(events) = command_execution_started_event(turn_id, item)
        .and_then(|started| command_execution_completed_event(turn_id, item).map(|completed| (started, completed)))
    {
        return events.0.into_iter().chain(events.1).collect();
    }

    if let Some(item) = thread_item_to_core(item) {
        return vec![Event {
            id: String::new(),
            msg: EventMsg::ItemCompleted(ItemCompletedEvent {
                thread_id,
                turn_id: turn_id.to_string(),
                item,
            }),
        }];
    }

    Vec::new()
}

fn command_execution_started_event(turn_id: &str, item: &ThreadItem) -> Option<Vec<Event>> {
    let ThreadItem::CommandExecution {
        id,
        command,
        cwd,
        process_id,
        source,
        command_actions,
        ..
    } = item
    else {
        return None;
    };

    Some(vec![Event {
        id: String::new(),
        msg: EventMsg::ExecCommandBegin(ExecCommandBeginEvent {
            call_id: id.clone(),
            process_id: process_id.clone(),
            turn_id: turn_id.to_string(),
            command: split_command_string(command),
            cwd: cwd.clone(),
            parsed_cmd: command_actions
                .iter()
                .cloned()
                .map(codex_app_server_protocol::CommandAction::into_core)
                .collect(),
            source: source.to_core(),
            interaction_input: None,
        }),
    }])
}

fn command_execution_completed_event(turn_id: &str, item: &ThreadItem) -> Option<Vec<Event>> {
    let ThreadItem::CommandExecution {
        id,
        command,
        cwd,
        process_id,
        source,
        status,
        command_actions,
        aggregated_output,
        exit_code,
        duration_ms,
    } = item
    else {
        return None;
    };

    if matches!(
        status,
        codex_app_server_protocol::CommandExecutionStatus::InProgress
    ) {
        return Some(Vec::new());
    }

    let status = match status {
        codex_app_server_protocol::CommandExecutionStatus::InProgress => return Some(Vec::new()),
        codex_app_server_protocol::CommandExecutionStatus::Completed => {
            ExecCommandStatus::Completed
        }
        codex_app_server_protocol::CommandExecutionStatus::Failed => ExecCommandStatus::Failed,
        codex_app_server_protocol::CommandExecutionStatus::Declined => ExecCommandStatus::Declined,
    };

    let duration = Duration::from_millis(
        duration_ms
            .and_then(|value| u64::try_from(value).ok())
            .unwrap_or_default(),
    );
    let aggregated_output = aggregated_output.clone().unwrap_or_default();

    Some(vec![Event {
        id: String::new(),
        msg: EventMsg::ExecCommandEnd(ExecCommandEndEvent {
            call_id: id.clone(),
            process_id: process_id.clone(),
            turn_id: turn_id.to_string(),
            command: split_command_string(command),
            cwd: cwd.clone(),
            parsed_cmd: command_actions
                .iter()
                .cloned()
                .map(codex_app_server_protocol::CommandAction::into_core)
                .collect(),
            source: source.to_core(),
            interaction_input: None,
            stdout: String::new(),
            stderr: String::new(),
            aggregated_output: aggregated_output.clone(),
            exit_code: exit_code.unwrap_or(-1),
            duration,
            formatted_output: aggregated_output,
            status,
        }),
    }])
}

fn thread_item_to_core(item: &ThreadItem) -> Option<TurnItem> {
    match item {
        ThreadItem::UserMessage { id, content } => Some(TurnItem::UserMessage(UserMessageItem {
            id: id.clone(),
            content: content
                .iter()
                .cloned()
                .map(codex_app_server_protocol::UserInput::into_core)
                .collect(),
        })),
        ThreadItem::AgentMessage {
            id,
            text,
            phase,
            memory_citation,
        } => Some(TurnItem::AgentMessage(AgentMessageItem {
            id: id.clone(),
            content: vec![AgentMessageContent::Text { text: text.clone() }],
            phase: phase.clone(),
            memory_citation: memory_citation.clone().map(|citation| {
                codex_protocol::memory_citation::MemoryCitation {
                    entries: citation
                        .entries
                        .into_iter()
                        .map(
                            |entry| codex_protocol::memory_citation::MemoryCitationEntry {
                                path: entry.path,
                                line_start: entry.line_start,
                                line_end: entry.line_end,
                                note: entry.note,
                            },
                        )
                        .collect(),
                    rollout_ids: citation.thread_ids,
                }
            }),
        })),
        ThreadItem::Plan { id, text } => Some(TurnItem::Plan(PlanItem {
            id: id.clone(),
            text: text.clone(),
        })),
        ThreadItem::Reasoning {
            id,
            summary,
            content,
        } => Some(TurnItem::Reasoning(ReasoningItem {
            id: id.clone(),
            summary_text: summary.clone(),
            raw_content: content.clone(),
        })),
        ThreadItem::WebSearch { id, query, action } => Some(TurnItem::WebSearch(WebSearchItem {
            id: id.clone(),
            query: query.clone(),
            action: app_server_web_search_action_to_core(action.clone()?)?,
        })),
        ThreadItem::ImageGeneration {
            id,
            status,
            revised_prompt,
            result,
            saved_path,
        } => Some(TurnItem::ImageGeneration(ImageGenerationItem {
            id: id.clone(),
            status: status.clone(),
            revised_prompt: revised_prompt.clone(),
            result: result.clone(),
            saved_path: saved_path.clone(),
        })),
        ThreadItem::ContextCompaction { id } => {
            Some(TurnItem::ContextCompaction(ContextCompactionItem {
                id: id.clone(),
            }))
        }
        ThreadItem::CommandExecution { .. }
        | ThreadItem::FileChange { .. }
        | ThreadItem::McpToolCall { .. }
        | ThreadItem::DynamicToolCall { .. }
        | ThreadItem::CollabAgentToolCall { .. }
        | ThreadItem::HookPrompt { .. }
        | ThreadItem::ImageView { .. }
        | ThreadItem::EnteredReviewMode { .. }
        | ThreadItem::ExitedReviewMode { .. } => {
            tracing::debug!("ignoring unsupported app-server thread item in TUI bridge");
            None
        }
    }
}

fn app_server_web_search_action_to_core(
    action: codex_app_server_protocol::WebSearchAction,
) -> Option<WebSearchAction> {
    Some(match action {
        codex_app_server_protocol::WebSearchAction::Search { query, queries } => {
            WebSearchAction::Search { query, queries }
        }
        codex_app_server_protocol::WebSearchAction::OpenPage { url } => {
            WebSearchAction::OpenPage { url }
        }
        codex_app_server_protocol::WebSearchAction::FindInPage { url, pattern } => {
            WebSearchAction::FindInPage { url, pattern }
        }
        codex_app_server_protocol::WebSearchAction::Other => WebSearchAction::Other,
    })
}
