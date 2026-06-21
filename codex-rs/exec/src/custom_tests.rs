use clap::Parser;
use pretty_assertions::assert_eq;

use super::Cli;

#[test]
fn custom__codex_home_cli_flag__flag_is_global() {
    let cli = Cli::try_parse_from(["codex-exec", "--codex-home", "/tmp/codex-home"])
        .expect("parse should succeed");

    assert_eq!(cli.shared.codex_home, Some("/tmp/codex-home".into()));
}

#[test]
fn custom__codex_memory_cli_flag__flag_is_global() {
    let cli = Cli::try_parse_from(["codex-exec", "--codex-memory", "/tmp/codex-memory"])
        .expect("parse should succeed");

    assert_eq!(cli.shared.codex_memory, Some("/tmp/codex-memory".into()));
}

#[test]
fn custom__config_toml_read_control__config_toml_file_flag_is_global() {
    let cli =
        Cli::try_parse_from(["codex-exec", "--config", "alt.toml"]).expect("parse should succeed");

    assert_eq!(cli.shared.config_toml_file, Some("alt.toml".into()));
}

#[test]
fn custom__config_toml_read_control__config_toml_file_conflicts_with_no_config() {
    let err = Cli::try_parse_from(["codex-exec", "--config", "alt.toml", "--no-config"])
        .expect_err("parse should fail");

    assert!(err.to_string().contains("cannot be used with"));
}
