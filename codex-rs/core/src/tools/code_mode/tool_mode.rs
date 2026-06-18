use codex_protocol::openai_models::ToolMode;

use crate::session::turn_context::TurnContext;

pub(crate) fn code_mode_enabled(turn_context: &TurnContext) -> bool {
    if matches!(turn_context.model_info.tool_mode, Some(ToolMode::Direct)) {
        return false;
    }

    let features = turn_context.features.get();
    features.enabled(codex_features::Feature::CodeMode)
        || features.enabled(codex_features::Feature::CodeModeOnly)
        || matches!(
            turn_context.model_info.tool_mode,
            Some(ToolMode::CodeMode) | Some(ToolMode::CodeModeOnly)
        )
}

pub(crate) fn code_mode_only_enabled(turn_context: &TurnContext) -> bool {
    if matches!(turn_context.model_info.tool_mode, Some(ToolMode::Direct)) {
        return false;
    }

    let features = turn_context.features.get();
    features.enabled(codex_features::Feature::CodeModeOnly)
        || matches!(
            turn_context.model_info.tool_mode,
            Some(ToolMode::CodeModeOnly)
        )
}

pub(crate) fn code_mode_model_enabled(turn_context: &TurnContext) -> bool {
    if matches!(turn_context.model_info.tool_mode, Some(ToolMode::Direct)) {
        return false;
    }

    if turn_context
        .features
        .get()
        .enabled(codex_features::Feature::MultiAgentV2)
        && turn_context.config.multi_agent_v2.non_code_mode_only
    {
        return false;
    }

    if matches!(
        turn_context.model_info.tool_mode,
        Some(ToolMode::CodeMode) | Some(ToolMode::CodeModeOnly)
    ) {
        return true;
    }

    let features = turn_context.features.get();
    features.enabled(codex_features::Feature::CodeMode)
        || features.enabled(codex_features::Feature::CodeModeOnly)
}

pub(crate) fn code_mode_model_only_enabled(turn_context: &TurnContext) -> bool {
    if matches!(turn_context.model_info.tool_mode, Some(ToolMode::Direct)) {
        return false;
    }

    if turn_context
        .features
        .get()
        .enabled(codex_features::Feature::MultiAgentV2)
        && turn_context.config.multi_agent_v2.non_code_mode_only
    {
        return false;
    }

    if matches!(
        turn_context.model_info.tool_mode,
        Some(ToolMode::CodeModeOnly)
    ) {
        return true;
    }

    let features = turn_context.features.get();
    features.enabled(codex_features::Feature::CodeModeOnly)
}
