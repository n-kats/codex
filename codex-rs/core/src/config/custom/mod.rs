mod user_shell;

use codex_config::custom::CustomConfigToml;

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct CustomPermissions {
    pub(crate) user_shell_no_inject: bool,
}

pub(crate) fn resolve_custom_config(
    custom: &CustomConfigToml,
    _shell_environment_policy: &codex_protocol::config_types::ShellEnvironmentPolicy,
    startup_warnings: &mut Vec<String>,
) -> std::io::Result<CustomPermissions> {
    let user_shell_no_inject = user_shell::resolve_no_inject(&custom.user_shell, startup_warnings);

    Ok(CustomPermissions {
        user_shell_no_inject,
    })
}
