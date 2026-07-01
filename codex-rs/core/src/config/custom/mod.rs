mod user_shell;

use codex_config::custom::CustomConfigToml;
use codex_protocol::config_types::ShellEnvironmentPolicy;

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct CustomPermissions {
    assistant_shell_environment_policy: Option<ShellEnvironmentPolicy>,
    user_shell_environment_policy: Option<ShellEnvironmentPolicy>,
    pub(crate) user_shell_no_inject: bool,
}

impl CustomPermissions {
    pub(crate) fn assistant_shell_environment_policy<'a>(
        &'a self,
        base: &'a ShellEnvironmentPolicy,
    ) -> &'a ShellEnvironmentPolicy {
        self.assistant_shell_environment_policy
            .as_ref()
            .unwrap_or(base)
    }

    pub(crate) fn user_shell_environment_policy<'a>(
        &'a self,
        base: &'a ShellEnvironmentPolicy,
    ) -> &'a ShellEnvironmentPolicy {
        self.user_shell_environment_policy
            .as_ref()
            .or(self.assistant_shell_environment_policy.as_ref())
            .unwrap_or(base)
    }
}

pub(crate) fn resolve_custom_config(
    custom: &CustomConfigToml,
    shell_environment_policy: &ShellEnvironmentPolicy,
    startup_warnings: &mut Vec<String>,
) -> std::io::Result<CustomPermissions> {
    let assistant_shell_environment_policy = custom
        .assistant_shell_environment_policy
        .clone()
        .map(ShellEnvironmentPolicy::from);
    let user_shell_environment_policy = custom
        .user_shell_environment_policy
        .clone()
        .map(ShellEnvironmentPolicy::from);
    let _ = shell_environment_policy;
    let user_shell_no_inject = user_shell::resolve_no_inject(&custom.user_shell, startup_warnings);

    Ok(CustomPermissions {
        assistant_shell_environment_policy,
        user_shell_environment_policy,
        user_shell_no_inject,
    })
}
