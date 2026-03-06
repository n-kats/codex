#![allow(non_snake_case)]
use super::Config;
use super::ConfigOverrides;
use super::ConfigToml;
use super::CustomConfigToml;
use super::CustomThemeDiffToml;
use super::CustomThemeToml;
use tempfile::TempDir;

use pretty_assertions::assert_eq;

#[test]
fn custom__差分テーマ色__tomlの色設定を読み込める() {
    let cfg = r##"
[custom.theme.diff]
enabled = false
line_bg = false
gutter = false
sign = true
content = true
add_line_bg = "#102030"
del_line_bg = "#a0b0c0"
"##;
    let parsed = toml::from_str::<ConfigToml>(cfg).expect("TOML deserialization should succeed");
    let custom = parsed.custom.expect("expected custom section");
    let theme = custom.theme.expect("expected custom.theme");
    assert_eq!(theme.diff.enabled, Some(false));
    assert_eq!(theme.diff.line_bg, Some(false));
    assert_eq!(theme.diff.gutter, Some(false));
    assert_eq!(theme.diff.sign, Some(true));
    assert_eq!(theme.diff.content, Some(true));
    assert_eq!(theme.diff.add_line_bg.as_deref(), Some("#102030"));
    assert_eq!(theme.diff.del_line_bg.as_deref(), Some("#a0b0c0"));
}

#[test]
fn custom__差分テーマ色__不正な16進色はエラーになる() -> std::io::Result<()> {
    let codex_home = TempDir::new()?;
    let cfg = ConfigToml {
        custom: Some(CustomConfigToml {
            theme: Some(CustomThemeToml {
                diff: CustomThemeDiffToml {
                    enabled: None,
                    line_bg: None,
                    gutter: None,
                    sign: None,
                    content: None,
                    add_line_bg: Some("#12zz00".to_string()),
                    del_line_bg: None,
                },
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let err = Config::load_from_base_config_with_overrides(
        cfg,
        ConfigOverrides::default(),
        codex_home.path().to_path_buf(),
    )
    .expect_err("invalid hex color should fail");
    assert!(err.to_string().contains("custom.theme.diff.add_line_bg"));
    Ok(())
}

#[test]
fn custom__差分テーマ色__正しい16進色を実行時設定へ反映する() -> std::io::Result<()> {
    let codex_home = TempDir::new()?;
    let cfg = ConfigToml {
        custom: Some(CustomConfigToml {
            theme: Some(CustomThemeToml {
                diff: CustomThemeDiffToml {
                    enabled: Some(false),
                    line_bg: Some(false),
                    gutter: Some(true),
                    sign: Some(false),
                    content: Some(true),
                    add_line_bg: Some("#010203".to_string()),
                    del_line_bg: Some("0a0b0c".to_string()),
                },
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let config = Config::load_from_base_config_with_overrides(
        cfg,
        ConfigOverrides::default(),
        codex_home.path().to_path_buf(),
    )?;
    assert_eq!(config.custom_diff_add_line_bg, Some((1, 2, 3)));
    assert_eq!(config.custom_diff_del_line_bg, Some((10, 11, 12)));
    assert!(!config.custom_diff_enabled);
    assert!(!config.custom_diff_line_bg_enabled);
    assert!(config.custom_diff_gutter_enabled);
    assert!(!config.custom_diff_sign_enabled);
    assert!(config.custom_diff_content_enabled);
    Ok(())
}

#[test]
fn custom__差分テーマ色__フラグ未指定時は有効が既定値() -> std::io::Result<()> {
    let codex_home = TempDir::new()?;
    let cfg = ConfigToml::default();
    let config = Config::load_from_base_config_with_overrides(
        cfg,
        ConfigOverrides::default(),
        codex_home.path().to_path_buf(),
    )?;

    assert!(config.custom_diff_enabled);
    assert!(config.custom_diff_line_bg_enabled);
    assert!(config.custom_diff_gutter_enabled);
    assert!(config.custom_diff_sign_enabled);
    assert!(config.custom_diff_content_enabled);
    Ok(())
}
