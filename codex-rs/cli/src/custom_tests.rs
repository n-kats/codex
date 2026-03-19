#![allow(non_snake_case)]
use super::*;
use clap::CommandFactory;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

fn temp_dir_path(suffix: &str) -> (PathBuf, String) {
    let dir = std::env::temp_dir().join(suffix);
    let arg = dir.to_string_lossy().to_string();
    (dir, arg)
}

#[test]
fn custom__agents_md__複数指定を順序どおり取得できる() {
    let cli = MultitoolCli::try_parse_from([
        "codex",
        "--agents-md",
        "AGENTS.md",
        "--agents-md",
        "docs/AGENTS.override.md",
    ])
    .expect("parse should succeed");

    assert_eq!(
        cli.agents_md,
        vec![
            PathBuf::from("AGENTS.md"),
            PathBuf::from("docs/AGENTS.override.md"),
        ]
    );
}

#[test]
fn custom__agents_md__execサブコマンドでも解釈できる() {
    let cli = MultitoolCli::try_parse_from(["codex", "--agents-md", "AGENTS.md", "exec", "hello"])
        .expect("parse should succeed");

    assert_eq!(cli.agents_md, vec![PathBuf::from("AGENTS.md")]);
}

#[tokio::test]
async fn custom__agents_md__interactive起動時にtuiへ引き継がれる() {
    clear_interactive_tui_agents_md_capture_for_test();

    let interactive =
        TuiCli::try_parse_from(["codex"]).expect("interactive args should parse correctly");
    let agents_md = vec![
        PathBuf::from("AGENTS.md"),
        PathBuf::from("docs/AGENTS.override.md"),
    ];

    run_interactive_tui(
        interactive,
        Arg0DispatchPaths::default(),
        codex_core::config_loader::LoaderOverrides::default(),
        agents_md.clone(),
    )
    .await
    .expect("interactive bootstrap should succeed in test stub");

    assert_eq!(
        take_interactive_tui_agents_md_capture_for_test(),
        Some(agents_md)
    );
}

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
    assert_eq!(overrides.user_config_path, Some(PathBuf::from("alt.toml")));
}

#[test]
fn custom__codex_home_cli_flag__フラグが解釈できる() {
    let (dir, arg) = temp_dir_path("codex-home");
    let cli = MultitoolCli::try_parse_from(["codex", "--codex-home", arg.as_str()])
        .expect("parse should succeed");
    assert_eq!(cli.codex_home, Some(dir));
}

#[test]
fn custom__codex_home_cli_flag__サブコマンド後でも解釈できる() {
    let (dir, arg) = temp_dir_path("codex-home");
    let cli = MultitoolCli::try_parse_from([
        "codex",
        "exec",
        "--codex-home",
        arg.as_str(),
        "--json",
        "hello",
    ])
    .expect("parse should succeed");
    assert_eq!(cli.codex_home, Some(dir));
}

#[test]
fn custom__codex_home_cli_flag__イコール形式でも解釈できる() {
    let (dir, arg) = temp_dir_path("codex-home");
    let arg = format!("--codex-home={arg}");
    let cli = MultitoolCli::try_parse_from(["codex", arg.as_str()]).expect("parse should succeed");
    assert_eq!(cli.codex_home, Some(dir));
}

#[test]
fn custom__codex_memory_cli_flag__サブコマンド後でも解釈できる() {
    let (dir, arg) = temp_dir_path("codex-memories");
    let cli = MultitoolCli::try_parse_from([
        "codex",
        "exec",
        "--codex-memory",
        arg.as_str(),
        "--json",
        "hello",
    ])
    .expect("parse should succeed");
    assert_eq!(cli.codex_memory, Some(dir));
}

#[test]
fn custom__codex_memory_cli_flag__イコール形式でも解釈できる() {
    let (dir, arg) = temp_dir_path("codex-memories");
    let arg = format!("--codex-memory={arg}");
    let cli = MultitoolCli::try_parse_from(["codex", arg.as_str()]).expect("parse should succeed");
    assert_eq!(cli.codex_memory, Some(dir));
}

#[test]
fn custom__shell_startup_files_cli_flag__サブコマンド後でも解釈できる() {
    let cli = MultitoolCli::try_parse_from([
        "codex",
        "exec",
        "--shell-startup-files",
        "clean",
        "--json",
        "hello",
    ])
    .expect("parse should succeed");
    assert_eq!(cli.shell_startup_files.as_deref(), Some("clean"));
}

#[test]
fn custom__shell_startup_files_cli_flag__イコール形式でも解釈できる() {
    let cli = MultitoolCli::try_parse_from(["codex", "--shell-startup-files=clean"])
        .expect("parse should succeed");
    assert_eq!(cli.shell_startup_files.as_deref(), Some("clean"));
}

#[test]
fn custom__custom_flags__helpに表示される() {
    let help = MultitoolCli::command().render_long_help().to_string();
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
}
