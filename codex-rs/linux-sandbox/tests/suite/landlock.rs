#![cfg(target_os = "linux")]
#![allow(clippy::unwrap_used)]
use codex_core::config::types::ShellEnvironmentPolicy;
use codex_core::error::CodexErr;
use codex_core::error::Result;
use codex_core::error::SandboxErr;
use codex_core::exec::ExecParams;
use codex_core::exec::process_exec_tool_call;
use codex_core::exec_env::create_env;
use codex_core::sandboxing::SandboxPermissions;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::FileSystemSpecialPath;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_protocol::protocol::ReadOnlyAccess;
use codex_protocol::protocol::SandboxPolicy;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::Output;
use std::process::Stdio;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::process::Command;

// At least on GitHub CI, the arm64 tests appear to need longer timeouts.
// Some environments also need a bit more time for short sandboxed commands.

#[cfg(not(target_arch = "aarch64"))]
const SHORT_TIMEOUT_MS: u64 = 1_000;
#[cfg(target_arch = "aarch64")]
const SHORT_TIMEOUT_MS: u64 = 5_000;

#[cfg(not(target_arch = "aarch64"))]
const LONG_TIMEOUT_MS: u64 = 3_000;
#[cfg(target_arch = "aarch64")]
const LONG_TIMEOUT_MS: u64 = 5_000;

#[cfg(not(target_arch = "aarch64"))]
const NETWORK_TIMEOUT_MS: u64 = 2_000;
#[cfg(target_arch = "aarch64")]
const NETWORK_TIMEOUT_MS: u64 = 10_000;

const BWRAP_UNAVAILABLE_ERR: &str = "build-time bubblewrap is not available in this build.";
const BWRAP_PERMISSION_ERR_SNIPPETS: &[&str] = &[
    "setting up uid map: Permission denied",
    "No permissions to create a new namespace",
    "unprivileged user namespaces",
    "unprivileged_userns_clone",
];

fn sysctl_value_is_zero(path: &str) -> bool {
    match std::fs::read_to_string(path) {
        Ok(value) => value.trim() == "0",
        Err(_) => false,
    }
}

fn userns_is_disabled() -> bool {
    // Vendored bubblewrap relies on unprivileged user namespaces. Some CI/container
    // environments disable them, causing bwrap to fail with:
    // "No permissions to create a new namespace ..."
    sysctl_value_is_zero("/proc/sys/kernel/unprivileged_userns_clone")
        || sysctl_value_is_zero("/proc/sys/user/max_user_namespaces")
}

fn create_env_from_core_vars() -> HashMap<String, String> {
    let policy = ShellEnvironmentPolicy::default();
    create_env(&policy, None)
}

#[expect(clippy::print_stdout)]
async fn run_cmd(cmd: &[&str], writable_roots: &[PathBuf], timeout_ms: u64) {
    let output = run_cmd_output(cmd, writable_roots, timeout_ms).await;
    if output.exit_code != 0 {
        println!("stdout:\n{}", output.stdout.text);
        println!("stderr:\n{}", output.stderr.text);
        panic!("exit code: {}", output.exit_code);
    }
}

#[expect(clippy::expect_used)]
async fn run_cmd_output(
    cmd: &[&str],
    writable_roots: &[PathBuf],
    timeout_ms: u64,
) -> codex_core::exec::ExecToolCallOutput {
    run_cmd_result_with_writable_roots(cmd, writable_roots, timeout_ms, false, false)
        .await
        .expect("sandboxed command should execute")
}

async fn run_cmd_result_with_writable_roots(
    cmd: &[&str],
    writable_roots: &[PathBuf],
    timeout_ms: u64,
    use_bwrap_sandbox: bool,
    network_access: bool,
) -> Result<codex_core::exec::ExecToolCallOutput> {
    let use_legacy_landlock = !use_bwrap_sandbox;
    let sandbox_policy = SandboxPolicy::WorkspaceWrite {
        writable_roots: writable_roots
            .iter()
            .map(|p| AbsolutePathBuf::try_from(p.as_path()).unwrap())
            .collect(),
        read_only_access: Default::default(),
        network_access,
        // Exclude tmp-related folders from writable roots because we need a
        // folder that is writable by tests but that we intentionally disallow
        // writing to in the sandbox.
        exclude_tmpdir_env_var: true,
        exclude_slash_tmp: true,
    };
    let file_system_sandbox_policy = FileSystemSandboxPolicy::from(&sandbox_policy);
    let network_sandbox_policy = NetworkSandboxPolicy::from(&sandbox_policy);
    run_cmd_result_with_policies(
        cmd,
        sandbox_policy,
        file_system_sandbox_policy,
        network_sandbox_policy,
        timeout_ms,
        use_legacy_landlock,
    )
    .await
}

#[expect(clippy::expect_used)]
async fn run_cmd_result_with_policies(
    cmd: &[&str],
    sandbox_policy: SandboxPolicy,
    file_system_sandbox_policy: FileSystemSandboxPolicy,
    network_sandbox_policy: NetworkSandboxPolicy,
    timeout_ms: u64,
    use_legacy_landlock: bool,
) -> Result<codex_core::exec::ExecToolCallOutput> {
    let cwd = std::env::current_dir().expect("cwd should exist");
    let sandbox_cwd = cwd.clone();
    let params = ExecParams {
        command: cmd.iter().copied().map(str::to_owned).collect(),
        cwd,
        expiration: timeout_ms.into(),
        env: create_env_from_core_vars(),
        network: None,
        sandbox_permissions: SandboxPermissions::UseDefault,
        windows_sandbox_level: WindowsSandboxLevel::Disabled,
        windows_sandbox_private_desktop: false,
        justification: None,
        arg0: None,
        run_as: None,
    };
    let codex_linux_sandbox_exe = Some(super::codex_linux_sandbox_exe());

    process_exec_tool_call(
        params,
        &sandbox_policy,
        &file_system_sandbox_policy,
        network_sandbox_policy,
        sandbox_cwd.as_path(),
        &codex_linux_sandbox_exe,
        use_legacy_landlock,
        None,
    )
    .await
}

fn is_bwrap_unavailable_stderr(stderr: &str) -> bool {
    stderr.contains(BWRAP_UNAVAILABLE_ERR)
}

fn is_bwrap_permission_error(stderr: &str) -> bool {
    BWRAP_PERMISSION_ERR_SNIPPETS
        .iter()
        .any(|snippet| stderr.contains(snippet))
}

async fn run_linux_sandbox_direct_require_bwrap(
    command: &[&str],
    sandbox_policy: &SandboxPolicy,
    env: HashMap<String, String>,
    timeout_ms: u64,
) -> Output {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => panic!("cwd should exist: {err}"),
    };
    let policy_json = match serde_json::to_string(sandbox_policy) {
        Ok(policy_json) => policy_json,
        Err(err) => panic!("policy should serialize: {err}"),
    };

    let mut args = vec![
        "--sandbox-policy-cwd".to_string(),
        cwd.to_string_lossy().to_string(),
        "--sandbox-policy".to_string(),
        policy_json,
        "--".to_string(),
    ];
    args.extend(command.iter().map(|entry| (*entry).to_string()));

    let mut cmd = Command::new(super::codex_linux_sandbox_exe());
    cmd.args(args)
        .current_dir(cwd)
        .env_clear()
        .envs(env)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = match tokio::time::timeout(Duration::from_millis(timeout_ms), cmd.output()).await {
        Ok(output) => output,
        Err(_) => {
            return Output {
                status: std::process::ExitStatus::from_raw(124 << 8),
                stdout: Vec::new(),
                stderr: b"sandbox command timed out".to_vec(),
            };
        }
    };
    match output {
        Ok(output) => output,
        Err(err) => panic!("sandbox command should execute: {err}"),
    }
}

async fn should_skip_bwrap_tests() -> bool {
    if userns_is_disabled() {
        return true;
    }

    let output = run_linux_sandbox_direct_require_bwrap(
        &["bash", "-lc", "true"],
        &SandboxPolicy::new_read_only_policy(),
        create_env_from_core_vars(),
        NETWORK_TIMEOUT_MS,
    )
    .await;
    if output.status.success() {
        return false;
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("sandbox command timed out") {
        return true;
    }
    if is_bwrap_unavailable_stderr(stderr.as_ref()) {
        return true;
    }
    if is_bwrap_permission_error(stderr.as_ref()) {
        return true;
    }

    false
}

fn expect_denied(
    result: Result<codex_core::exec::ExecToolCallOutput>,
    context: &str,
) -> codex_core::exec::ExecToolCallOutput {
    match result {
        Ok(output) => {
            assert_ne!(output.exit_code, 0, "{context}: expected nonzero exit code");
            output
        }
        Err(CodexErr::Sandbox(SandboxErr::Denied { output, .. })) => *output,
        Err(err) => panic!("{context}: {err:?}"),
    }
}

#[tokio::test]
async fn test_root_read() {
    run_cmd(&["ls", "-l", "/bin"], &[], SHORT_TIMEOUT_MS).await;
}

#[tokio::test]
#[should_panic]
async fn test_root_write() {
    let tmpfile = NamedTempFile::new().unwrap();
    let tmpfile_path = tmpfile.path().to_string_lossy();
    run_cmd(
        &["bash", "-lc", &format!("echo blah > {tmpfile_path}")],
        &[],
        SHORT_TIMEOUT_MS,
    )
    .await;
}

#[tokio::test]
async fn test_dev_null_write() {
    run_cmd(
        &[
            "bash",
            "--noprofile",
            "--norc",
            "-c",
            "echo blah > /dev/null",
        ],
        &[],
        // We have seen timeouts when running this test in CI on GitHub,
        // so we are using a generous timeout until we can diagnose further.
        LONG_TIMEOUT_MS,
    )
    .await;
}

#[tokio::test]
async fn test_writable_root() {
    let tmpdir = tempfile::tempdir().unwrap();
    let file_path = tmpdir.path().join("test");
    run_cmd(
        &[
            "bash",
            "--noprofile",
            "--norc",
            "-c",
            &format!("echo blah > {}", file_path.to_string_lossy()),
        ],
        &[tmpdir.path().to_path_buf()],
        // We have seen timeouts when running this test in CI on GitHub,
        // so we are using a generous timeout until we can diagnose further.
        LONG_TIMEOUT_MS,
    )
    .await;
}

#[tokio::test]
async fn sandbox_ignores_missing_writable_roots_under_bwrap() {
    if should_skip_bwrap_tests().await {
        eprintln!("skipping bwrap test: bwrap sandbox prerequisites are unavailable");
        return;
    }

    let tempdir = tempfile::tempdir().expect("tempdir");
    let existing_root = tempdir.path().join("existing");
    let missing_root = tempdir.path().join("missing");
    std::fs::create_dir(&existing_root).expect("create existing root");

    let output = run_cmd_result_with_writable_roots(
        &["bash", "-lc", "printf sandbox-ok"],
        &[existing_root, missing_root],
        LONG_TIMEOUT_MS,
        false,
        true,
    )
    .await
    .expect("sandboxed command should execute");

    assert_eq!(output.exit_code, 0);
    assert_eq!(output.stdout.text, "sandbox-ok");
}

#[tokio::test]
async fn test_no_new_privs_is_enabled() {
    let output = run_cmd_output(
        &["bash", "-lc", "grep '^NoNewPrivs:' /proc/self/status"],
        &[],
        SHORT_TIMEOUT_MS,
    )
    .await;
    let line = output
        .stdout
        .text
        .lines()
        .find(|line| line.starts_with("NoNewPrivs:"))
        .unwrap_or("");
    assert_eq!(line.trim(), "NoNewPrivs:\t1");
}

#[tokio::test]
#[should_panic(expected = "Sandbox(Timeout")]
async fn test_timeout() {
    run_cmd(&["sleep", "2"], &[], 50).await;
}

/// Helper that runs `cmd` under the Linux sandbox and asserts that the command
/// does NOT succeed (i.e. returns a non‑zero exit code) **unless** the binary
/// is missing in which case we silently treat it as an accepted skip so the
/// suite remains green on leaner CI images.
#[expect(clippy::expect_used)]
async fn assert_network_blocked(cmd: &[&str]) {
    let cwd = std::env::current_dir().expect("cwd should exist");
    let sandbox_cwd = cwd.clone();
    let params = ExecParams {
        command: cmd.iter().copied().map(str::to_owned).collect(),
        cwd,
        // Give the tool a generous 2-second timeout so even slow DNS timeouts
        // do not stall the suite.
        expiration: NETWORK_TIMEOUT_MS.into(),
        env: create_env_from_core_vars(),
        network: None,
        sandbox_permissions: SandboxPermissions::UseDefault,
        windows_sandbox_level: WindowsSandboxLevel::Disabled,
        windows_sandbox_private_desktop: false,
        justification: None,
        arg0: None,
        run_as: None,
    };

    let sandbox_policy = SandboxPolicy::new_read_only_policy();
    let codex_linux_sandbox_exe: Option<PathBuf> = Some(super::codex_linux_sandbox_exe());
    let result = process_exec_tool_call(
        params,
        &sandbox_policy,
        &FileSystemSandboxPolicy::from(&sandbox_policy),
        NetworkSandboxPolicy::from(&sandbox_policy),
        sandbox_cwd.as_path(),
        &codex_linux_sandbox_exe,
        false,
        None,
    )
    .await;

    let output = match result {
        Ok(output) => output,
        Err(CodexErr::Sandbox(SandboxErr::Denied { output, .. })) => *output,
        _ => {
            panic!("expected sandbox denied error, got: {result:?}");
        }
    };

    dbg!(&output.stderr.text);
    dbg!(&output.stdout.text);
    dbg!(&output.exit_code);

    // A completely missing binary exits with 127.  Anything else should also
    // be non‑zero (EPERM from seccomp will usually bubble up as 1, 2, 13…)
    // If—*and only if*—the command exits 0 we consider the sandbox breached.

    if output.exit_code == 0 {
        panic!(
            "Network sandbox FAILED - {cmd:?} exited 0\nstdout:\n{}\nstderr:\n{}",
            output.stdout.text, output.stderr.text
        );
    }
}

#[tokio::test]
async fn sandbox_blocks_curl() {
    assert_network_blocked(&["curl", "-I", "http://openai.com"]).await;
}

#[tokio::test]
async fn sandbox_blocks_wget() {
    assert_network_blocked(&["wget", "-qO-", "http://openai.com"]).await;
}

#[tokio::test]
async fn sandbox_blocks_ping() {
    // ICMP requires raw socket – should be denied quickly with EPERM.
    assert_network_blocked(&["ping", "-c", "1", "8.8.8.8"]).await;
}

#[tokio::test]
async fn sandbox_blocks_nc() {
    // Zero‑length connection attempt to localhost.
    assert_network_blocked(&["nc", "-z", "127.0.0.1", "80"]).await;
}

#[tokio::test]
async fn sandbox_blocks_git_and_codex_writes_inside_writable_root() {
    if should_skip_bwrap_tests().await {
        eprintln!("skipping bwrap test: vendored bwrap was not built in this environment");
        return;
    }

    let tmpdir = tempfile::tempdir().expect("tempdir");
    let dot_git = tmpdir.path().join(".git");
    let dot_codex = tmpdir.path().join(".codex");
    std::fs::create_dir_all(&dot_git).expect("create .git");
    std::fs::create_dir_all(&dot_codex).expect("create .codex");

    let git_target = dot_git.join("config");
    let codex_target = dot_codex.join("config.toml");

    let git_output = expect_denied(
        run_cmd_result_with_writable_roots(
            &[
                "bash",
                "-lc",
                &format!("echo denied > {}", git_target.to_string_lossy()),
            ],
            &[tmpdir.path().to_path_buf()],
            LONG_TIMEOUT_MS,
            true,
            false,
        )
        .await,
        ".git write should be denied under bubblewrap",
    );

    let codex_output = expect_denied(
        run_cmd_result_with_writable_roots(
            &[
                "bash",
                "-lc",
                &format!("echo denied > {}", codex_target.to_string_lossy()),
            ],
            &[tmpdir.path().to_path_buf()],
            LONG_TIMEOUT_MS,
            true,
            false,
        )
        .await,
        ".codex write should be denied under bubblewrap",
    );
    assert_ne!(git_output.exit_code, 0);
    assert_ne!(codex_output.exit_code, 0);
}

#[tokio::test]
async fn sandbox_blocks_codex_symlink_replacement_attack() {
    if should_skip_bwrap_tests().await {
        eprintln!("skipping bwrap test: vendored bwrap was not built in this environment");
        return;
    }

    use std::os::unix::fs::symlink;

    let tmpdir = tempfile::tempdir().expect("tempdir");
    let decoy = tmpdir.path().join("decoy-codex");
    std::fs::create_dir_all(&decoy).expect("create decoy dir");

    let dot_codex = tmpdir.path().join(".codex");
    symlink(&decoy, &dot_codex).expect("create .codex symlink");

    let codex_target = dot_codex.join("config.toml");

    let codex_output = expect_denied(
        run_cmd_result_with_writable_roots(
            &[
                "bash",
                "-lc",
                &format!("echo denied > {}", codex_target.to_string_lossy()),
            ],
            &[tmpdir.path().to_path_buf()],
            LONG_TIMEOUT_MS,
            true,
            false,
        )
        .await,
        ".codex symlink replacement should be denied",
    );
    assert_ne!(codex_output.exit_code, 0);
}

#[tokio::test]
async fn sandbox_blocks_explicit_split_policy_carveouts_under_bwrap() {
    if should_skip_bwrap_tests().await {
        eprintln!("skipping bwrap test: bwrap sandbox prerequisites are unavailable");
        return;
    }

    let tmpdir = tempfile::tempdir().expect("tempdir");
    let blocked = tmpdir.path().join("blocked");
    std::fs::create_dir_all(&blocked).expect("create blocked dir");
    let blocked_target = blocked.join("secret.txt");
    // These tests bypass the usual legacy-policy bridge, so explicitly keep
    // the sandbox helper binary and minimal runtime paths readable.
    let sandbox_helper_dir = PathBuf::from(env!("CARGO_BIN_EXE_codex-linux-sandbox"))
        .parent()
        .expect("sandbox helper should have a parent")
        .to_path_buf();

    let sandbox_policy = SandboxPolicy::WorkspaceWrite {
        writable_roots: vec![AbsolutePathBuf::try_from(tmpdir.path()).expect("absolute tempdir")],
        read_only_access: Default::default(),
        network_access: true,
        exclude_tmpdir_env_var: true,
        exclude_slash_tmp: true,
    };
    let file_system_sandbox_policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Minimal,
            },
            access: FileSystemAccessMode::Read,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: AbsolutePathBuf::try_from(sandbox_helper_dir.as_path())
                    .expect("absolute helper dir"),
            },
            access: FileSystemAccessMode::Read,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: AbsolutePathBuf::try_from(tmpdir.path()).expect("absolute tempdir"),
            },
            access: FileSystemAccessMode::Write,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: AbsolutePathBuf::try_from(blocked.as_path()).expect("absolute blocked dir"),
            },
            access: FileSystemAccessMode::None,
        },
    ]);
    let output = expect_denied(
        run_cmd_result_with_policies(
            &[
                "bash",
                "-lc",
                &format!("echo denied > {}", blocked_target.to_string_lossy()),
            ],
            sandbox_policy,
            file_system_sandbox_policy,
            NetworkSandboxPolicy::Enabled,
            LONG_TIMEOUT_MS,
            false,
        )
        .await,
        "explicit split-policy carveout should be denied under bubblewrap",
    );

    assert_ne!(output.exit_code, 0);
}

#[tokio::test]
async fn sandbox_reenables_writable_subpaths_under_unreadable_parents() {
    if should_skip_bwrap_tests().await {
        eprintln!("skipping bwrap test: bwrap sandbox prerequisites are unavailable");
        return;
    }

    let tmpdir = tempfile::tempdir().expect("tempdir");
    let blocked = tmpdir.path().join("blocked");
    let allowed = blocked.join("allowed");
    std::fs::create_dir_all(&allowed).expect("create blocked/allowed dir");
    let allowed_target = allowed.join("note.txt");
    // These tests bypass the usual legacy-policy bridge, so explicitly keep
    // the sandbox helper binary and minimal runtime paths readable.
    let sandbox_helper_dir = PathBuf::from(env!("CARGO_BIN_EXE_codex-linux-sandbox"))
        .parent()
        .expect("sandbox helper should have a parent")
        .to_path_buf();

    let sandbox_policy = SandboxPolicy::WorkspaceWrite {
        writable_roots: vec![AbsolutePathBuf::try_from(tmpdir.path()).expect("absolute tempdir")],
        read_only_access: Default::default(),
        network_access: true,
        exclude_tmpdir_env_var: true,
        exclude_slash_tmp: true,
    };
    let file_system_sandbox_policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Minimal,
            },
            access: FileSystemAccessMode::Read,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: AbsolutePathBuf::try_from(sandbox_helper_dir.as_path())
                    .expect("absolute helper dir"),
            },
            access: FileSystemAccessMode::Read,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: AbsolutePathBuf::try_from(tmpdir.path()).expect("absolute tempdir"),
            },
            access: FileSystemAccessMode::Write,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: AbsolutePathBuf::try_from(blocked.as_path()).expect("absolute blocked dir"),
            },
            access: FileSystemAccessMode::None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: AbsolutePathBuf::try_from(allowed.as_path()).expect("absolute allowed dir"),
            },
            access: FileSystemAccessMode::Write,
        },
    ]);
    let output = run_cmd_result_with_policies(
        &[
            "bash",
            "-lc",
            &format!(
                "printf allowed > {} && cat {}",
                allowed_target.to_string_lossy(),
                allowed_target.to_string_lossy()
            ),
        ],
        sandbox_policy,
        file_system_sandbox_policy,
        NetworkSandboxPolicy::Enabled,
        LONG_TIMEOUT_MS,
        false,
    )
    .await
    .expect("nested writable carveout should execute under bubblewrap");
    assert_eq!(output.exit_code, 0);
    assert_eq!(output.stdout.text.trim(), "allowed");
}

#[tokio::test]
async fn sandbox_blocks_root_read_carveouts_under_bwrap() {
    if should_skip_bwrap_tests().await {
        eprintln!("skipping bwrap test: bwrap sandbox prerequisites are unavailable");
        return;
    }

    let tmpdir = tempfile::tempdir().expect("tempdir");
    let blocked = tmpdir.path().join("blocked");
    std::fs::create_dir_all(&blocked).expect("create blocked dir");
    let blocked_target = blocked.join("secret.txt");
    std::fs::write(&blocked_target, "secret").expect("seed blocked file");

    let sandbox_policy = SandboxPolicy::ReadOnly {
        access: ReadOnlyAccess::FullAccess,
        network_access: true,
    };
    let file_system_sandbox_policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: AbsolutePathBuf::try_from(blocked.as_path()).expect("absolute blocked dir"),
            },
            access: FileSystemAccessMode::None,
        },
    ]);
    let output = expect_denied(
        run_cmd_result_with_policies(
            &[
                "bash",
                "-lc",
                &format!("cat {}", blocked_target.to_string_lossy()),
            ],
            sandbox_policy,
            file_system_sandbox_policy,
            NetworkSandboxPolicy::Enabled,
            LONG_TIMEOUT_MS,
            false,
        )
        .await,
        "root-read carveout should be denied under bubblewrap",
    );

    assert_ne!(output.exit_code, 0);
}

#[tokio::test]
async fn sandbox_blocks_ssh() {
    // Force ssh to attempt a real TCP connection but fail quickly.  `BatchMode`
    // avoids password prompts, and `ConnectTimeout` keeps the hang time low.
    assert_network_blocked(&[
        "ssh",
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=1",
        "github.com",
    ])
    .await;
}

#[tokio::test]
async fn sandbox_blocks_getent() {
    assert_network_blocked(&["getent", "ahosts", "openai.com"]).await;
}

#[tokio::test]
async fn sandbox_blocks_dev_tcp_redirection() {
    // This syntax is only supported by bash and zsh. We try bash first.
    // Fallback generic socket attempt using /bin/sh with bash‑style /dev/tcp.  Not
    // all images ship bash, so we guard against 127 as well.
    assert_network_blocked(&["bash", "-c", "echo hi > /dev/tcp/127.0.0.1/80"]).await;
}
