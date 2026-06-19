mod run_as;

use crate::session::turn_context::TurnContext;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use std::collections::HashMap;

pub(crate) use run_as::RunAsUser;
#[cfg(unix)]
pub(crate) use run_as::RunAsRetry;
#[cfg(unix)]
pub(crate) use run_as::apply_unix_run_as;
#[cfg(unix)]
pub(crate) use run_as::run_as_sudo_fallback_command;

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

pub(crate) fn run_as_for_assistant_shell(turn_context: &TurnContext) -> Option<RunAsUser> {
    turn_context.config.permissions.custom.exec_run_as.clone()
}
