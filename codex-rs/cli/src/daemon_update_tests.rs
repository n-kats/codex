use super::*;
use codex_tui::DaemonUpdateSource;
use pretty_assertions::assert_eq;
use std::os::unix::fs::PermissionsExt;

#[test]
fn daemon_handoff_uses_selected_executable_and_propagates_failure() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let executable = dir.path().join("launching CLI");
    let receipt = dir.path().join("launching CLI.args");
    std::fs::write(
        &executable,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$0.args\"\n",
    )?;
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(/*mode*/ 0o700))?;
    for (source, expected) in [
        (
            DaemonUpdateSource::PublicStable,
            "app-server\ndaemon\nupdate\n",
        ),
        (
            DaemonUpdateSource::ThisCli,
            "app-server\ndaemon\nupdate\n--from-cli\n--yes\n",
        ),
    ] {
        run_update_action(UpdateAction::Daemon(source), Some(&executable))?;
        assert_eq!(std::fs::read_to_string(&receipt)?, expected);
    }
    let replacement = executable.with_extension("replacement");
    std::fs::write(&replacement, "#!/bin/sh\nexit 7\n")?;
    std::fs::set_permissions(
        &replacement,
        std::fs::Permissions::from_mode(/*mode*/ 0o700),
    )?;
    // Replace the fixture atomically so a just-finished shell cannot leave the
    // old executable inode temporarily busy on Linux.
    std::fs::rename(replacement, &executable)?;
    let error = run_update_action(
        UpdateAction::Daemon(DaemonUpdateSource::ThisCli),
        Some(&executable),
    )
    .unwrap_err();
    assert!(error.to_string().contains("Daemon update failed"));
    Ok(())
}
