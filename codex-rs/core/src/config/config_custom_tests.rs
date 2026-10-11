#![allow(non_snake_case)]

use crate::config::Config;
use crate::config::ConfigOverrides;
use codex_config::config_toml::ConfigToml;
use codex_utils_absolute_path::AbsolutePathBuf;
use core_test_support::TempDirExt;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

#[test]
fn custom__theme__tui_theme_defaults_to_none() {
    let cfg = r#"
[tui]
"#;
    let parsed = toml::from_str::<ConfigToml>(cfg).expect("TOML deserialization should succeed");
    assert_eq!(parsed.tui.as_ref().and_then(|t| t.theme.as_deref()), None);
}

#[test]
fn custom__theme__diff_deserializes_from_toml() {
    let cfg = r##"
[custom.theme.diff]
enabled = true
line_bg = false
gutter = true
sign = false
content = true
add_line_bg = "#102030"
del_line_bg = "402010"
"##;
    let parsed = toml::from_str::<ConfigToml>(cfg).expect("TOML deserialization should succeed");
    let diff = parsed.custom.theme.diff.expect("custom diff theme");

    assert_eq!(diff.enabled, Some(true));
    assert_eq!(diff.line_bg, Some(false));
    assert_eq!(diff.gutter, Some(true));
    assert_eq!(diff.sign, Some(false));
    assert_eq!(diff.content, Some(true));
    assert_eq!(diff.add_line_bg.as_deref(), Some("#102030"));
    assert_eq!(diff.del_line_bg.as_deref(), Some("402010"));
}

#[tokio::test]
async fn custom__agents_md__override_resolves_relative_paths_against_cwd() -> std::io::Result<()> {
    let codex_home = TempDir::new()?;
    let workspace = TempDir::new()?;

    let config = Config::load_from_base_config_with_overrides(
        ConfigToml::default(),
        ConfigOverrides {
            cwd: Some(workspace.path().to_path_buf()),
            project_doc_paths: vec![std::path::PathBuf::from("docs/AGENTS.md")],
            ..Default::default()
        },
        codex_home.abs(),
    )
    .await?;

    assert_eq!(
        config.project_doc_paths,
        vec![AbsolutePathBuf::try_from(
            workspace.path().join("docs/AGENTS.md")
        )?]
    );

    Ok(())
}
