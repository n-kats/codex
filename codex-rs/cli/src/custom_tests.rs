#![allow(non_snake_case)]
use super::*;
use clap::CommandFactory;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

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
fn custom__config_toml_read_control__helpに表示される() {
    let help = MultitoolCli::command().render_long_help().to_string();
    assert!(
        help.contains("--config"),
        "expected help to contain --config, got:\n{help}"
    );
    assert!(
        help.contains("--no-config"),
        "expected help to contain --no-config, got:\n{help}"
    );
}
