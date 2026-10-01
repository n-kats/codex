#![allow(non_snake_case)]

use super::*;
use codex_config::custom::CustomThemeDiffToml;
use pretty_assertions::assert_eq;

#[test]
fn custom__diff_theme__override_parses_hex_colors() {
    let (custom, warnings) = parse_custom_diff_theme_override(Some(&CustomThemeDiffToml {
        add_line_bg: Some("#102030".to_string()),
        del_line_bg: Some("402010".to_string()),
        line_bg: Some(false),
        sign: Some(false),
        ..Default::default()
    }));

    assert_eq!(warnings, Vec::<String>::new());
    assert_eq!(
        custom,
        Some(CustomDiffThemeOverride {
            enabled: true,
            line_bg: false,
            gutter: true,
            sign: false,
            content: true,
            add_line_bg: Some((0x10, 0x20, 0x30)),
            del_line_bg: Some((0x40, 0x20, 0x10)),
        })
    );
}

#[test]
fn custom__diff_theme__override_rejects_invalid_hex() {
    let (custom, warnings) = parse_custom_diff_theme_override(Some(&CustomThemeDiffToml {
        add_line_bg: Some("#12345".to_string()),
        del_line_bg: Some("not-hex".to_string()),
        ..Default::default()
    }));

    assert_eq!(
        custom,
        Some(CustomDiffThemeOverride {
            add_line_bg: None,
            del_line_bg: None,
            ..Default::default()
        })
    );
    assert_eq!(
        warnings,
        vec![
            "Ignoring invalid custom.theme.diff.add_line_bg value `#12345`; expected #RRGGBB or RRGGBB.".to_string(),
            "Ignoring invalid custom.theme.diff.del_line_bg value `not-hex`; expected #RRGGBB or RRGGBB.".to_string(),
        ]
    );
}
