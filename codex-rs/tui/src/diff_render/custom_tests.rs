#![allow(non_snake_case)]
use pretty_assertions::assert_eq;

use super::DiffColorLevel;
use super::DiffLineType;
use super::DiffPaletteOverride;
use super::DiffTheme;
use super::diff_palette_override;
use super::fallback_diff_backgrounds;
use super::rgb_color;
use super::set_diff_palette_override;
use super::style_add;
use super::style_del;
use super::style_gutter_for;
use super::style_line_bg_for;
use super::style_sign_add;
use super::style_sign_del;
use ratatui::style::Style;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::OnceLock;

static DIFF_PALETTE_TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

struct PaletteOverrideGuard {
    prev: DiffPaletteOverride,
    _lock_guard: MutexGuard<'static, ()>,
}

impl PaletteOverrideGuard {
    fn set(override_palette: DiffPaletteOverride) -> Self {
        let lock = DIFF_PALETTE_TEST_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("diff palette test mutex poisoned");
        let prev = diff_palette_override();
        set_diff_palette_override(override_palette);
        Self {
            prev,
            _lock_guard: lock,
        }
    }
}

impl Drop for PaletteOverrideGuard {
    fn drop(&mut self) {
        set_diff_palette_override(self.prev);
    }
}

#[test]
fn custom__差分テーマ色__追加削除の背景色を置換できる() {
    let _guard = PaletteOverrideGuard::set(DiffPaletteOverride {
        add_line_bg_rgb: Some((1, 2, 3)),
        del_line_bg_rgb: Some((4, 5, 6)),
        ..Default::default()
    });
    let dark_backgrounds = fallback_diff_backgrounds(DiffTheme::Dark, DiffColorLevel::TrueColor);
    let light_backgrounds = fallback_diff_backgrounds(DiffTheme::Light, DiffColorLevel::TrueColor);

    assert_eq!(
        style_line_bg_for(DiffLineType::Insert, dark_backgrounds),
        Style::default().bg(rgb_color((1, 2, 3)))
    );
    assert_eq!(
        style_line_bg_for(DiffLineType::Delete, light_backgrounds),
        Style::default().bg(rgb_color((4, 5, 6)))
    );
}

#[test]
fn custom__差分テーマ色__色指定を全体無効化できる() {
    let _guard = PaletteOverrideGuard::set(DiffPaletteOverride {
        enabled: false,
        ..Default::default()
    });
    let light_backgrounds = fallback_diff_backgrounds(DiffTheme::Light, DiffColorLevel::TrueColor);
    let dark_backgrounds = fallback_diff_backgrounds(DiffTheme::Dark, DiffColorLevel::TrueColor);

    assert_eq!(
        style_line_bg_for(DiffLineType::Insert, light_backgrounds),
        Style::default()
    );
    assert_eq!(
        style_gutter_for(
            DiffLineType::Delete,
            DiffTheme::Light,
            DiffColorLevel::TrueColor
        ),
        Style::default()
    );
    assert_eq!(
        style_sign_add(DiffTheme::Dark, DiffColorLevel::TrueColor, dark_backgrounds),
        Style::default()
    );
    assert_eq!(
        style_del(DiffTheme::Dark, DiffColorLevel::TrueColor, dark_backgrounds),
        Style::default()
    );
}

#[test]
fn custom__差分テーマ色__色指定を部分無効化できる() {
    let _guard = PaletteOverrideGuard::set(DiffPaletteOverride {
        line_bg_enabled: false,
        gutter_enabled: false,
        sign_enabled: false,
        content_enabled: false,
        ..Default::default()
    });
    let dark_backgrounds = fallback_diff_backgrounds(DiffTheme::Dark, DiffColorLevel::Ansi256);
    let light_backgrounds = fallback_diff_backgrounds(DiffTheme::Light, DiffColorLevel::TrueColor);
    let dark_truecolor_backgrounds =
        fallback_diff_backgrounds(DiffTheme::Dark, DiffColorLevel::TrueColor);

    assert_eq!(
        style_line_bg_for(DiffLineType::Delete, dark_backgrounds),
        Style::default()
    );
    assert_eq!(
        style_gutter_for(
            DiffLineType::Insert,
            DiffTheme::Light,
            DiffColorLevel::TrueColor
        ),
        Style::default()
    );
    assert_eq!(
        style_sign_del(
            DiffTheme::Light,
            DiffColorLevel::TrueColor,
            light_backgrounds
        ),
        Style::default()
    );
    assert_eq!(
        style_add(
            DiffTheme::Dark,
            DiffColorLevel::TrueColor,
            dark_truecolor_backgrounds,
        ),
        Style::default()
    );
}
