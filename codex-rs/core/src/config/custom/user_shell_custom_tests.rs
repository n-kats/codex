#![allow(non_snake_case)]

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn custom__user_shell__default_false_does_not_warn() {
    let user_shell = CustomUserShellToml { no_inject: None };
    let mut startup_warnings = Vec::new();

    assert!(!resolve_no_inject(&user_shell, &mut startup_warnings));
    assert!(startup_warnings.is_empty());
}

#[test]
fn custom__user_shell__explicit_false_warns() {
    let user_shell = CustomUserShellToml {
        no_inject: Some(false),
    };
    let mut startup_warnings = Vec::new();

    assert!(!resolve_no_inject(&user_shell, &mut startup_warnings));
    assert_eq!(
        startup_warnings,
        vec![USER_SHELL_NO_INJECT_WARNING.to_string()]
    );
}

#[test]
fn custom__user_shell__explicit_true_does_not_warn() {
    let user_shell = CustomUserShellToml {
        no_inject: Some(true),
    };
    let mut startup_warnings = Vec::new();

    assert!(resolve_no_inject(&user_shell, &mut startup_warnings));
    assert!(startup_warnings.is_empty());
}
