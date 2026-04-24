use codex_network_proxy::NetworkProxy;
use codex_utils_absolute_path::AbsolutePathBuf;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Child;
use tokio::process::Command;
use tracing::trace;

use codex_protocol::permissions::NetworkSandboxPolicy;

/// Experimental environment variable that will be set to some non-empty value
/// if both of the following are true:
///
/// 1. The process was spawned by Codex as part of a shell tool call.
/// 2. NetworkSandboxPolicy is restricted for the tool call.
///
/// We may try to have just one environment variable for all sandboxing
/// attributes, so this may change in the future.
pub const CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR: &str = "CODEX_SANDBOX_NETWORK_DISABLED";

/// Should be set when the process is spawned under a sandbox. Currently, the
/// value is "seatbelt" for macOS, but it may change in the future to
/// accommodate sandboxing configuration and other sandboxing mechanisms.
pub const CODEX_SANDBOX_ENV_VAR: &str = "CODEX_SANDBOX";

#[derive(Debug, Clone, Copy)]
pub enum StdioPolicy {
    RedirectForShellTool,
    Inherit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunAsUser {
    pub uid: u32,
    pub gid: u32,
    pub supplementary_gids: Option<Vec<u32>>,
}

#[cfg(unix)]
#[derive(Clone)]
struct RunAsRetry {
    run_as: RunAsUser,
    arg0: Option<String>,
    program: String,
    args: Vec<String>,
    cwd: PathBuf,
    env: HashMap<String, String>,
}

#[cfg(unix)]
fn ensure_argv0_symlink(program: &str, arg0: &str) -> std::io::Result<PathBuf> {
    let arg0 = std::path::Path::new(arg0)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| std::io::Error::other("arg0 must be a valid UTF-8 file name"))?;
    if arg0.is_empty() {
        return Err(std::io::Error::other("arg0 must be non-empty"));
    }

    let dir = PathBuf::from("/tmp/codex-argv0");
    std::fs::create_dir_all(&dir)?;
    let link_path = dir.join(arg0);

    if let Ok(target) = std::fs::read_link(&link_path) {
        if target == std::path::Path::new(program) {
            return Ok(link_path);
        }
    }
    let _ = std::fs::remove_file(&link_path);
    std::os::unix::fs::symlink(program, &link_path)?;
    Ok(link_path)
}

#[cfg(unix)]
fn configure_unix_pre_exec(
    cmd: &mut Command,
    stdio_policy: StdioPolicy,
    run_as: Option<RunAsUser>,
) {
    unsafe {
        let detach_from_tty = matches!(stdio_policy, StdioPolicy::RedirectForShellTool);
        #[cfg(target_os = "linux")]
        let parent_pid = libc::getpid();
        cmd.pre_exec(move || {
            if detach_from_tty {
                codex_utils_pty::process_group::detach_from_tty()?;
            }
            if let Some(run_as) = &run_as {
                if let Some(supplementary_gids) = &run_as.supplementary_gids {
                    let supplementary_gids = supplementary_gids
                        .iter()
                        .copied()
                        .map(|gid| gid as libc::gid_t)
                        .collect::<Vec<_>>();
                    if libc::setgroups(supplementary_gids.len(), supplementary_gids.as_ptr()) != 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                if libc::setgid(run_as.gid as libc::gid_t) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if libc::setuid(run_as.uid as libc::uid_t) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }

            #[cfg(target_os = "linux")]
            {
                codex_utils_pty::process_group::set_parent_death_signal(parent_pid)?;
            }
            Ok(())
        });
    }
}

#[cfg(unix)]
fn try_spawn_with_run_as_sudo(
    retry: RunAsRetry,
    stdio_policy: StdioPolicy,
    err: std::io::Error,
) -> std::io::Result<Child> {
    if unsafe { libc::geteuid() } == 0
        || (err.kind() != std::io::ErrorKind::PermissionDenied
            && err.raw_os_error() != Some(libc::EPERM))
    {
        return Err(err);
    }

    let Some(sudo_program) = ["/usr/bin/sudo", "/bin/sudo"]
        .into_iter()
        .find(|path| std::path::Path::new(path).exists())
    else {
        return Err(err);
    };
    let Some(env_program) = ["/usr/bin/env", "/bin/env"]
        .into_iter()
        .find(|path| std::path::Path::new(path).exists())
    else {
        return Err(err);
    };

    let uid = format!("#{}", retry.run_as.uid);
    let gid = format!("#{}", retry.run_as.gid);
    let program = if let Some(arg0) = retry.arg0.as_deref() {
        ensure_argv0_symlink(retry.program.as_str(), arg0)?
            .to_string_lossy()
            .to_string()
    } else {
        retry.program
    };

    let mut sudo_cmd = Command::new(sudo_program);
    sudo_cmd.args([
        "-n",
        "-u",
        uid.as_str(),
        "-g",
        gid.as_str(),
        "--",
        env_program,
    ]);
    sudo_cmd.arg("-i");

    let mut env_kv: Vec<_> = retry.env.iter().collect();
    env_kv.sort_unstable_by_key(|(key, _)| *key);
    for (key, value) in env_kv {
        sudo_cmd.arg(format!("{key}={value}"));
    }
    sudo_cmd.arg(program);
    sudo_cmd.args(retry.args);
    sudo_cmd.current_dir(retry.cwd);
    sudo_cmd.env_clear();
    configure_unix_pre_exec(&mut sudo_cmd, stdio_policy, None);

    match stdio_policy {
        StdioPolicy::RedirectForShellTool => {
            sudo_cmd.stdin(Stdio::null());
            sudo_cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        }
        StdioPolicy::Inherit => {
            sudo_cmd
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
        }
    }

    sudo_cmd.kill_on_drop(true).spawn()
}

/// Spawns the appropriate child process for the ExecParams and SandboxPolicy,
/// ensuring the args and environment variables used to create the `Command`
/// (and `Child`) honor the configuration.
///
/// For now, we take `NetworkSandboxPolicy` as a parameter to spawn_child()
/// because we need to determine whether to set the
/// `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` environment variable.
pub(crate) struct SpawnChildRequest<'a> {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub arg0: Option<&'a str>,
    pub cwd: AbsolutePathBuf,
    pub network_sandbox_policy: NetworkSandboxPolicy,
    pub network: Option<&'a NetworkProxy>,
    pub stdio_policy: StdioPolicy,
    pub env: HashMap<String, String>,
    pub run_as: Option<RunAsUser>,
}

pub(crate) async fn spawn_child_async(request: SpawnChildRequest<'_>) -> std::io::Result<Child> {
    let SpawnChildRequest {
        program,
        args,
        arg0,
        cwd,
        network_sandbox_policy,
        network,
        stdio_policy,
        mut env,
        run_as,
    } = request;

    trace!(
        "spawn_child_async: {program:?} {args:?} {arg0:?} {cwd:?} {network_sandbox_policy:?} {stdio_policy:?} {env:?}"
    );

    #[cfg(unix)]
    let run_as_retry = run_as.clone().map(|run_as| RunAsRetry {
        run_as,
        arg0: arg0.map(String::from),
        program: program.to_string_lossy().to_string(),
        args: args.clone(),
        cwd: cwd.clone().to_path_buf(),
        env: env.clone(),
    });

    let mut cmd = Command::new(&program);
    #[cfg(unix)]
    cmd.arg0(arg0.map_or_else(|| program.to_string_lossy().to_string(), String::from));
    cmd.args(args);
    cmd.current_dir(cwd);
    if let Some(network) = network {
        network.apply_to_env(&mut env);
    }
    cmd.env_clear();
    cmd.envs(env);

    if !network_sandbox_policy.is_enabled() {
        cmd.env(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR, "1");
    }

    // If this Codex process dies (including being killed via SIGKILL), we want
    // any child processes that were spawned as part of a `"shell"` tool call
    // to also be terminated.

    #[cfg(unix)]
    configure_unix_pre_exec(&mut cmd, stdio_policy, run_as);

    match stdio_policy {
        StdioPolicy::RedirectForShellTool => {
            // Do not create a file descriptor for stdin because otherwise some
            // commands may hang forever waiting for input. For example, ripgrep has
            // a heuristic where it may try to read from stdin as explained here:
            // https://github.com/BurntSushi/ripgrep/blob/e2362d4d5185d02fa857bf381e7bd52e66fafc73/crates/core/flags/hiargs.rs#L1101-L1103
            cmd.stdin(Stdio::null());

            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        }
        StdioPolicy::Inherit => {
            // Inherit stdin, stdout, and stderr from the parent process.
            cmd.stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
        }
    }

    cmd.kill_on_drop(true);
    match cmd.spawn() {
        Ok(child) => Ok(child),
        Err(err) => {
            #[cfg(unix)]
            {
                if let Some(run_as_retry) = run_as_retry {
                    try_spawn_with_run_as_sudo(run_as_retry, stdio_policy, err)
                } else {
                    Err(err)
                }
            }
            #[cfg(not(unix))]
            {
                Err(err)
            }
        }
    }
}
