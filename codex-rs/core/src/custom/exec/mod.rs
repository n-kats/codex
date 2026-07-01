use crate::session::turn_context::TurnContext;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use std::collections::HashMap;

pub(crate) fn assistant_shell_environment_policy(
    turn_context: &TurnContext,
) -> &ShellEnvironmentPolicy {
    turn_context
        .config
        .permissions
        .custom
        .assistant_shell_environment_policy(
            &turn_context.config.permissions.shell_environment_policy,
        )
}

pub(crate) fn assistant_shell_environment_set(
    turn_context: &TurnContext,
) -> HashMap<String, String> {
    assistant_shell_environment_policy(turn_context)
        .r#set
        .clone()
}
