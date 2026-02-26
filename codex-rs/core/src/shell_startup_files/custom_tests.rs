#![allow(non_snake_case)]
use super::ShellStartupFiles;
use super::apply_shell_startup_files_env_with_home;
use super::parse_shell_startup_files;
use crate::shell::ShellType;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn custom__シェル起動ファイル__未指定はdefaultとして解釈する() {
    assert_eq!(parse_shell_startup_files(None), ShellStartupFiles::Default);
    assert_eq!(
        parse_shell_startup_files(Some("default")),
        ShellStartupFiles::Default
    );
    assert_eq!(
        parse_shell_startup_files(Some("unknown")),
        ShellStartupFiles::Default
    );
}

#[test]
fn custom__シェル起動ファイル__clean指定を解釈する() {
    assert_eq!(
        parse_shell_startup_files(Some("clean")),
        ShellStartupFiles::Clean
    );
    assert_eq!(
        parse_shell_startup_files(Some(" CLEAN ")),
        ShellStartupFiles::Clean
    );
}

#[test]
fn custom__シェル起動ファイル__cleanはzshのみ隔離する() {
    let tmp = tempdir().expect("create TempDir");
    let codex_home = tmp.path();

    let mut env = HashMap::new();
    apply_shell_startup_files_env_with_home(&mut env, ShellType::Bash, codex_home);
    assert_eq!(env.get("ZDOTDIR"), None);

    let mut env = HashMap::new();
    apply_shell_startup_files_env_with_home(&mut env, ShellType::Zsh, codex_home);
    let zdotdir = env.get("ZDOTDIR").expect("ZDOTDIR set");
    assert_eq!(
        PathBuf::from(zdotdir),
        codex_home.join("shell_dotfiles").join("empty_zdotdir")
    );
}
