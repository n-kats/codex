use super::MultitoolCli;
use super::loader_overrides_from_shared;
use clap::Parser;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_cli::SharedCliOptions;
use pretty_assertions::assert_eq;

#[test]
fn custom__codex_home_cli_flag__flag_is_global() {
    let cli = MultitoolCli::try_parse_from(["codex", "--codex-home", "/tmp/codex-home"])
        .expect("parse should succeed");

    assert_eq!(
        cli.interactive.shared.clone().into_inner().codex_home,
        Some("/tmp/codex-home".into())
    );
}

#[test]
fn custom__codex_memory_cli_flag__flag_is_global() {
    let cli = MultitoolCli::try_parse_from(["codex", "--codex-memory", "/tmp/codex-memory"])
        .expect("parse should succeed");

    assert_eq!(
        cli.interactive.shared.clone().into_inner().codex_memory,
        Some("/tmp/codex-memory".into())
    );
}

#[test]
fn custom__config_toml_read_control__config_toml_file_flag_is_global() {
    let cli = MultitoolCli::try_parse_from(["codex", "--config", "alt.toml"])
        .expect("parse should succeed");

    assert_eq!(
        cli.interactive.shared.clone().into_inner().config_toml_file,
        Some("alt.toml".into())
    );
}

#[test]
fn custom__config_toml_read_control__config_toml_file_conflicts_with_no_config() {
    let err = MultitoolCli::try_parse_from(["codex", "--config", "alt.toml", "--no-config"])
        .expect_err("parse should fail");

    assert!(err.to_string().contains("cannot be used with"));
}

#[test]
fn custom__config_toml_read_control__build_loader_overrides_no_config_disables_user_and_project() {
    let shared = SharedCliOptions {
        no_config: true,
        ..Default::default()
    };

    let loader_overrides =
        loader_overrides_from_shared(&shared).expect("loader overrides should build");

    assert!(loader_overrides.ignore_user_config);
    assert!(loader_overrides.ignore_project_config);
    assert_eq!(loader_overrides.user_config_path, None);
    assert_eq!(loader_overrides.user_config_profile, None);
}

#[test]
fn custom__config_toml_read_control__build_loader_overrides_config_sets_user_config_path() {
    let shared = SharedCliOptions {
        config_toml_file: Some("alt.toml".into()),
        ..Default::default()
    };

    let loader_overrides =
        loader_overrides_from_shared(&shared).expect("loader overrides should build");

    assert_eq!(
        loader_overrides.user_config_path,
        Some(
            AbsolutePathBuf::relative_to_current_dir("alt.toml")
                .expect("relative config path should resolve")
        )
    );
}
