use codex_config::custom::CustomUserShellToml;

pub(crate) const USER_SHELL_NO_INJECT_WARNING: &str = "custom.user_shell.no_inject is false (default); `!` (UserShell) commands and their outputs will be injected into the model context and recorded to the local session history. Set custom.user_shell.no_inject=true to disable injection/recording, and avoid secrets in `!` commands/output.";

pub(super) fn resolve_no_inject(
    user_shell: &CustomUserShellToml,
    startup_warnings: &mut Vec<String>,
) -> bool {
    let no_inject = user_shell.no_inject.unwrap_or(false);
    if user_shell.no_inject == Some(false) {
        startup_warnings.push(USER_SHELL_NO_INJECT_WARNING.to_string());
    }
    no_inject
}
