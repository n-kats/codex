mod exec;
mod user_shell;

use crate::custom::exec::RunAsUser;
use codex_config::custom::CustomConfigToml;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use codex_protocol::config_types::ShellEnvironmentPolicyInherit;

#[derive(Debug, Clone, Default)]
pub(crate) struct CustomPermissions {
    assistant_shell_environment_policy: Option<ShellEnvironmentPolicy>,
    user_shell_environment_policy: Option<ShellEnvironmentPolicy>,
    pub(crate) user_shell_no_inject: bool,
    pub(crate) exec_run_as: Option<RunAsUser>,
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
    let custom_exec_run_as = exec::resolve_run_as(&custom.exec)?;
    if custom_exec_run_as.is_some()
        && matches!(
            assistant_shell_environment_policy
                .as_ref()
                .unwrap_or(shell_environment_policy)
                .inherit,
            ShellEnvironmentPolicyInherit::All
        )
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "custom.exec is configured, but shell_environment_policy.inherit is 'all'; refusing because a model-run `env`/`printenv` would leak the invoker environment (set inherit = 'core' or 'none', or use include_only).",
        ));
    }
    exec::warn_if_current_user(&custom.exec, custom_exec_run_as.as_ref(), startup_warnings);
    let user_shell_no_inject = user_shell::resolve_no_inject(&custom.user_shell, startup_warnings);

    Ok(CustomPermissions {
        assistant_shell_environment_policy,
        user_shell_environment_policy,
        user_shell_no_inject,
        exec_run_as: custom_exec_run_as,
    })
}
