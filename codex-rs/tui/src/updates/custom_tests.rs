#![allow(non_snake_case)]
use super::is_newer;
use super::parse_current_version;
use pretty_assertions::assert_eq;

#[test]
fn custom__更新チェックカスタム版__customサフィックス版でもsemverと新旧比較できる() {
    assert_eq!(
        parse_current_version("1.2.3-custom-2026-01-25"),
        Some((1, 2, 3))
    );
    assert_eq!(is_newer("1.2.4", "1.2.3-custom-2026-01-25"), Some(true));
    assert_eq!(is_newer("1.2.3", "1.2.3-custom-2026-01-25"), Some(false));
}
