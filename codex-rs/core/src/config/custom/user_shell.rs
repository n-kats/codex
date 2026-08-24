use codex_config::custom::CustomUserShellToml;

pub(crate) const USER_SHELL_NO_INJECT_WARNING: &str = "custom.user_shell.no_inject is false (default); `!` (UserShell) commands and their outputs will be injected into the model context and recorded to the local session history. Set custom.user_shell.no_inject=true to disable injection/recording, and avoid secrets in `!` commands/output.";

pub(super) fn resolve_no_inject(
    user_shell: &CustomUserShellToml,
    startup_warnings: &mut Vec<String>,
) -> bool {
    let no_inject = user_shell.no_inject.unwrap_or(false);
    if !no_inject {
        startup_warnings.push(USER_SHELL_NO_INJECT_WARNING.to_string());
    }
    no_inject
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn resolve_no_inject_warns_for_default_false() {
        let user_shell = CustomUserShellToml { no_inject: None };
        let mut startup_warnings = Vec::new();

        assert!(!resolve_no_inject(&user_shell, &mut startup_warnings));
        assert_eq!(
            startup_warnings,
            vec![USER_SHELL_NO_INJECT_WARNING.to_string()]
        );
    }

    #[test]
    fn resolve_no_inject_does_not_warn_for_explicit_true() {
        let user_shell = CustomUserShellToml {
            no_inject: Some(true),
        };
        let mut startup_warnings = Vec::new();

        assert!(resolve_no_inject(&user_shell, &mut startup_warnings));
        assert!(startup_warnings.is_empty());
    }
}
