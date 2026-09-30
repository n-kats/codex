#![allow(non_snake_case)]

use super::super::redact_snapshot_exports;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use pretty_assertions::assert_eq;

#[test]
fn custom__shell_snapshot__redact_exports_keeps_only_allowed_exports() {
    let snapshot = "\
# Snapshot file
# Functions

# exports 9
declare -x PATH=\"/usr/bin\"
declare -x HOME=\"/home/user\"
declare -x XDG_CONFIG_HOME=\"/home/user/.config\"
declare -x CODEX_TEST_API_KEY=\"secret\"
declare -x CODEX_TEST_SECRET=\"secret\"
declare -x CODEX_TEST_TOKEN=\"secret\"
declare -x CODEX_TEST_BUILD_HOME=\"/tmp/build\"
declare -x CODEX_TEST_OUT_DIR=\"/tmp/out\"
declare -x CODEX_TEST_CACHE_DIR=\"/tmp/cache\"
";

    let redacted = redact_snapshot_exports(snapshot, &ShellEnvironmentPolicy::default(), false);

    assert_eq!(
        redacted,
        "\
# Snapshot file
# Functions

# exports 3
declare -x PATH=\"/usr/bin\"
declare -x HOME=\"/home/user\"
declare -x XDG_CONFIG_HOME=\"/home/user/.config\"
"
    );
}

#[test]
fn custom__shell_snapshot__redact_exports_preserves_snapshot_without_exports_section() {
    let snapshot = "# Snapshot file\n# Functions\n";

    assert_eq!(
        redact_snapshot_exports(snapshot, &ShellEnvironmentPolicy::default(), false),
        snapshot
    );
}
