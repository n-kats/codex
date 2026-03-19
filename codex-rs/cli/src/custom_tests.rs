#![allow(non_snake_case)]
use super::*;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

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
