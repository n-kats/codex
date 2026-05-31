#![allow(non_snake_case)]
use super::*;
use clap::CommandFactory;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn custom__config_toml_read_control__config_toml_file_flag_is_global() {
    let cli = MultitoolCli::try_parse_from(["codex", "exec", "--config", "alt.toml"])
        .expect("parse should succeed");
    assert_eq!(
        cli.config_toml_file,
        Some(std::path::PathBuf::from("alt.toml"))
    );
    assert!(!cli.no_config);
}

#[test]
fn custom__codex_home_cli_flag__flag_is_global() {
    let cli = MultitoolCli::try_parse_from(["codex", "exec", "--codex-home", "/tmp/codex-home"])
        .expect("parse should succeed");
    assert_eq!(cli.codex_home, Some(PathBuf::from("/tmp/codex-home")));
}

#[test]
fn custom__codex_memory_cli_flag__flag_is_global() {
    let cli =
        MultitoolCli::try_parse_from(["codex", "exec", "--codex-memory", "/tmp/codex-memory"])
            .expect("parse should succeed");
    assert_eq!(cli.codex_memory, Some(PathBuf::from("/tmp/codex-memory")));
}

#[test]
fn custom__shell_startup_files_cli_flag__flag_is_global() {
    let cli = MultitoolCli::try_parse_from(["codex", "exec", "--shell-startup-files", "clean"])
        .expect("parse should succeed");
    assert_eq!(cli.shell_startup_files.as_deref(), Some("clean"));
}

#[test]
fn custom__shell_startup_files_cli_flag__equals_form_is_global() {
    let cli = MultitoolCli::try_parse_from(["codex", "--shell-startup-files=clean"])
        .expect("parse should succeed");
    assert_eq!(cli.shell_startup_files.as_deref(), Some("clean"));
}

#[test]
fn custom__agents_md_restore__flag_is_global() {
    let cli = MultitoolCli::try_parse_from(["codex", "--agents-md", "/tmp/AGENTS.md"])
        .expect("parse should succeed");
    assert_eq!(
        cli.interactive.agents_md,
        vec![PathBuf::from("/tmp/AGENTS.md")]
    );
}

#[test]
fn custom__agents_md_restore__flag_is_global_for_exec() {
    let cli = MultitoolCli::try_parse_from(["codex", "exec", "--agents-md", "/tmp/AGENTS.md"])
        .expect("parse should succeed");
    let subcommand = match cli.subcommand {
        Some(Subcommand::Exec(exec_cli)) => exec_cli,
        other => panic!("expected exec subcommand, got {other:?}"),
    };
    assert_eq!(
        subcommand.shared.agents_md,
        vec![PathBuf::from("/tmp/AGENTS.md")]
    );
}

#[test]
fn custom__codex_home_cli_flag__bootstrap_sets_code_home_env() {
    let _guard = ENV_LOCK.lock().expect("lock env mutation");
    let previous = std::env::var_os("CODEX_HOME");
    bootstrap_home_overrides_from_iter([
        OsString::from("codex"),
        OsString::from("--codex-home"),
        OsString::from("/tmp/codex-home"),
    ]);
    assert_eq!(
        std::env::var_os("CODEX_HOME"),
        Some(OsString::from("/tmp/codex-home"))
    );
    match previous {
        Some(value) => unsafe {
            std::env::set_var("CODEX_HOME", value);
        },
        None => unsafe {
            std::env::remove_var("CODEX_HOME");
        },
    }
}

#[test]
fn custom__codex_memory_cli_flag__bootstrap_sets_code_memory_env() {
    let _guard = ENV_LOCK.lock().expect("lock env mutation");
    let previous = std::env::var_os("CODEX_MEMORIES_HOME");
    bootstrap_home_overrides_from_iter([
        OsString::from("codex"),
        OsString::from("--codex-memory"),
        OsString::from("/tmp/codex-memory"),
    ]);
    assert_eq!(
        std::env::var_os("CODEX_MEMORIES_HOME"),
        Some(OsString::from("/tmp/codex-memory"))
    );
    match previous {
        Some(value) => unsafe {
            std::env::set_var("CODEX_MEMORIES_HOME", value);
        },
        None => unsafe {
            std::env::remove_var("CODEX_MEMORIES_HOME");
        },
    }
}

#[test]
fn custom__shell_startup_files_cli_flag__bootstrap_sets_shell_startup_files_env() {
    let _guard = ENV_LOCK.lock().expect("lock env mutation");
    let previous = std::env::var_os("CODEX_SHELL_STARTUP_FILES");
    bootstrap_home_overrides_from_iter([
        OsString::from("codex"),
        OsString::from("--shell-startup-files"),
        OsString::from("clean"),
    ]);
    assert_eq!(
        std::env::var_os("CODEX_SHELL_STARTUP_FILES"),
        Some(OsString::from("clean"))
    );
    match previous {
        Some(value) => unsafe {
            std::env::set_var("CODEX_SHELL_STARTUP_FILES", value);
        },
        None => unsafe {
            std::env::remove_var("CODEX_SHELL_STARTUP_FILES");
        },
    }
}

#[test]
fn custom__shell_startup_files_cli_flag__bootstrap_sets_shell_startup_files_env_from_equals_form() {
    let _guard = ENV_LOCK.lock().expect("lock env mutation");
    let previous = std::env::var_os("CODEX_SHELL_STARTUP_FILES");
    bootstrap_home_overrides_from_iter([
        OsString::from("codex"),
        OsString::from("--shell-startup-files=clean"),
    ]);
    assert_eq!(
        std::env::var_os("CODEX_SHELL_STARTUP_FILES"),
        Some(OsString::from("clean"))
    );
    match previous {
        Some(value) => unsafe {
            std::env::set_var("CODEX_SHELL_STARTUP_FILES", value);
        },
        None => unsafe {
            std::env::remove_var("CODEX_SHELL_STARTUP_FILES");
        },
    }
}

#[test]
fn custom__config_toml_read_control__config_toml_file_conflicts_with_no_config() {
    let err = MultitoolCli::try_parse_from(["codex", "--config", "alt.toml", "--no-config"])
        .expect_err("parse should fail");
    let msg = err.to_string();
    assert!(msg.contains("--config") || msg.contains("--no-config"));
}

#[test]
fn custom__config_toml_read_control__build_loader_overrides_no_config_disables_user_and_project() {
    let overrides = build_loader_overrides(Some(PathBuf::from("alt.toml")), true);
    assert!(overrides.disable_user_config);
    assert!(overrides.disable_project_config);
    assert_eq!(overrides.user_config_path, None);
}

#[test]
fn custom__config_toml_read_control__build_loader_overrides_config_sets_user_config_path() {
    let overrides = build_loader_overrides(Some(PathBuf::from("alt.toml")), false);
    assert!(!overrides.disable_user_config);
    assert!(!overrides.disable_project_config);
    assert_eq!(
        overrides.user_config_path,
        Some(
            AbsolutePathBuf::try_from(
                std::env::current_dir()
                    .expect("current dir")
                    .join("alt.toml"),
            )
            .expect("absolute config path"),
        )
    );
}

#[test]
fn custom__config_toml_read_control__helpに表示される() {
    let help = MultitoolCli::command().render_long_help().to_string();
    assert!(
        help.contains("--config"),
        "expected help to contain --config, got:\n{help}"
    );
    assert!(
        help.contains("--codex-home"),
        "expected help to contain --codex-home, got:\n{help}"
    );
    assert!(
        help.contains("--codex-memory"),
        "expected help to contain --codex-memory, got:\n{help}"
    );
    assert!(
        help.contains("--shell-startup-files"),
        "expected help to contain --shell-startup-files, got:\n{help}"
    );
    assert!(
        help.contains("--no-config"),
        "expected help to contain --no-config, got:\n{help}"
    );
    assert!(
        help.contains("--agents-md"),
        "expected help to contain --agents-md, got:\n{help}"
    );
}
