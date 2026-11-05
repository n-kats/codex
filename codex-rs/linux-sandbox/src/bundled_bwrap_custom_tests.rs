#![allow(non_snake_case)]

use super::*;
use tempfile::tempdir;

#[test]
fn custom__bundled_bwrap__finds_parent_dev_bwrap_for_cargo_test_binary() {
    let temp_dir = tempdir().expect("temp dir");
    let deps_dir = temp_dir.path().join("deps");
    let exe = deps_dir.join("all-test-binary");
    let expected_bwrap = temp_dir.path().join("bwrap");
    std::fs::create_dir_all(&deps_dir).expect("create deps directory");
    codex_utils_cargo_bin::write_executable(&exe, "").expect("write test executable");
    codex_utils_cargo_bin::write_executable(&expected_bwrap, "").expect("write bwrap executable");

    assert_eq!(
        find_legacy_for_exe(&exe, /*bazel_candidate*/ None),
        Some(AbsolutePathBuf::from_absolute_path(&expected_bwrap).expect("absolute"))
    );
}
