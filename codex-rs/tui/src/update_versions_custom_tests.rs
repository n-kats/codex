#![allow(non_snake_case)]

use super::is_newer;
use pretty_assertions::assert_eq;

#[test]
fn custom__update_check__suffix_versions_are_comparable_against_plain_semver() {
    assert_eq!(is_newer("1.2.4", "1.2.3-custom-2026-01-25"), Some(true));
    assert_eq!(is_newer("1.2.4", "1.2.3+custom.2026-01-25"), Some(true));
    assert_eq!(is_newer("1.2.3", "1.2.3-custom-2026-01-25"), Some(false));
}
