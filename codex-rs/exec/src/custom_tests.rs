#![allow(non_snake_case)]
use super::*;
use pretty_assertions::assert_eq;

#[test]
fn custom__config_toml_read_control__config_toml_file_flag_is_global() {
    let cli = TopCli::parse_from([
        "codex-exec",
        "resume",
        "--config",
        "alt.toml",
        "--last",
        "2+2",
    ]);

    assert_eq!(
        cli.config_toml_file,
        Some(std::path::PathBuf::from("alt.toml"))
    );
    assert!(!cli.no_config);
}

#[test]
fn custom__config_toml_read_control__config_toml_file_conflicts_with_no_config() {
    let err = TopCli::try_parse_from(["codex-exec", "--config", "alt.toml", "--no-config"])
        .expect_err("parse should fail");

    assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
}
