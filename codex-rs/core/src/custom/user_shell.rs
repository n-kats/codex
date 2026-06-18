use crate::session::turn_context::TurnContext;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use std::collections::HashMap;

pub(crate) fn shell_environment_policy(turn_context: &TurnContext) -> &ShellEnvironmentPolicy {
    turn_context
        .config
        .permissions
        .custom
        .user_shell_environment_policy(&turn_context.config.permissions.shell_environment_policy)
}

pub(crate) fn shell_environment_set(turn_context: &TurnContext) -> HashMap<String, String> {
    shell_environment_policy(turn_context).r#set.clone()
}

pub(crate) fn no_inject(turn_context: &TurnContext) -> bool {
    turn_context.config.permissions.custom.user_shell_no_inject
}
