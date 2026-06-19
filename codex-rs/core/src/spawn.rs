use crate::custom::exec::RunAsUser;
#[cfg(unix)]
use crate::custom::exec::apply_unix_run_as;
#[cfg(unix)]
use crate::custom::exec::run_as_sudo_fallback_command;
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

fn configure_stdio(cmd: &mut Command, stdio_policy: StdioPolicy) {
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
}

#[cfg(unix)]
fn configure_unix_pre_exec(
    cmd: &mut Command,
    stdio_policy: StdioPolicy,
    run_as: Option<RunAsUser>,
) {
    // If this Codex process dies (including being killed via SIGKILL), we want
    // any child processes that were spawned as part of a `"shell"` tool call
    // to also be terminated.
    unsafe {
        let detach_from_tty = matches!(stdio_policy, StdioPolicy::RedirectForShellTool);
        #[cfg(target_os = "linux")]
        let parent_pid = libc::getpid();
        cmd.pre_exec(move || {
            if detach_from_tty {
                codex_utils_pty::process_group::detach_from_tty()?;
            }

            if let Some(run_as) = &run_as {
                apply_unix_run_as(run_as)?;
            }

            // This relies on prctl(2), so it only works on Linux.
            #[cfg(target_os = "linux")]
            {
                // This prctl call effectively requests, "deliver SIGTERM when my
                // current parent dies."
                codex_utils_pty::process_group::set_parent_death_signal(parent_pid)?;
            }
            Ok(())
        });
    }
}

/// Spawns the appropriate child process for the exec params and sandbox settings,
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

    #[cfg(not(unix))]
    let _ = &run_as;

    let mut cmd = Command::new(&program);
    #[cfg(unix)]
    cmd.arg0(arg0.map_or_else(|| program.to_string_lossy().to_string(), String::from));
    cmd.args(&args);
    cmd.current_dir(&cwd);
    if let Some(network) = network {
        network.apply_to_env(&mut env);
    }
    #[cfg(unix)]
    let run_as_retry = run_as.clone().map(|run_as| {
        crate::custom::exec::RunAsRetry::new(
            run_as,
            arg0.map(String::from),
            program.to_string_lossy().to_string(),
            args.clone(),
            cwd.clone().to_path_buf(),
            env.clone(),
        )
    });
    cmd.env_clear();
    cmd.envs(env);

    if !network_sandbox_policy.is_enabled() {
        cmd.env(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR, "1");
    }

    #[cfg(unix)]
    configure_unix_pre_exec(&mut cmd, stdio_policy, run_as);

    configure_stdio(&mut cmd, stdio_policy);

    match cmd.kill_on_drop(true).spawn() {
        Ok(child) => Ok(child),
        Err(err) => {
            #[cfg(unix)]
            {
                if let Some(run_as_retry) = run_as_retry {
                    let mut sudo_cmd = run_as_sudo_fallback_command(run_as_retry, err)?;
                    configure_unix_pre_exec(&mut sudo_cmd, stdio_policy, None);
                    configure_stdio(&mut sudo_cmd, stdio_policy);
                    sudo_cmd.kill_on_drop(true).spawn()
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
