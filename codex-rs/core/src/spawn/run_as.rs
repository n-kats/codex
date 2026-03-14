use crate::spawn::CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR;
use crate::spawn::RunAsUser;
use crate::spawn::SpawnChildRequest;
use crate::spawn::StdioPolicy;
use codex_protocol::permissions::NetworkSandboxPolicy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Child;
use tokio::process::Command;

struct RunAsPreExec {
    uid: libc::uid_t,
    gid: libc::gid_t,
    supplementary_gids: Option<Vec<libc::gid_t>>,
}

impl From<RunAsUser> for RunAsPreExec {
    fn from(value: RunAsUser) -> Self {
        Self {
            uid: value.uid as libc::uid_t,
            gid: value.gid as libc::gid_t,
            supplementary_gids: value
                .supplementary_gids
                .map(|gids| gids.into_iter().map(|gid| gid as libc::gid_t).collect()),
        }
    }
}

struct RunAsRetry {
    run_as: RunAsUser,
    arg0: Option<String>,
    program: String,
    args: Vec<String>,
    cwd: PathBuf,
    network_sandbox_policy: NetworkSandboxPolicy,
    env: HashMap<String, String>,
}

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

    // Best-effort cleanup if a previous run left behind a stale link/file.
    let _ = std::fs::remove_file(&link_path);

    std::os::unix::fs::symlink(program, &link_path)?;
    Ok(link_path)
}

fn apply_run_as_pre_exec(run_as: &RunAsPreExec) -> std::io::Result<()> {
    if let Some(groups) = run_as.supplementary_gids.as_ref() {
        let groups_ptr = groups.as_ptr();
        let groups_ptr = if groups.is_empty() {
            std::ptr::null()
        } else {
            groups_ptr
        };
        if unsafe { libc::setgroups(groups.len(), groups_ptr) } == -1 {
            return Err(std::io::Error::last_os_error());
        }
    }
    if unsafe { libc::setgid(run_as.gid) } == -1 {
        return Err(std::io::Error::last_os_error());
    }
    if unsafe { libc::setuid(run_as.uid) } == -1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

fn configure_unix_pre_exec(cmd: &mut Command, stdio_policy: StdioPolicy, run_as: RunAsUser) {
    unsafe {
        let detach_from_tty = matches!(stdio_policy, StdioPolicy::RedirectForShellTool);
        #[cfg(target_os = "linux")]
        let parent_pid = libc::getpid();
        let run_as = RunAsPreExec::from(run_as);
        cmd.pre_exec(move || {
            if detach_from_tty {
                codex_utils_pty::process_group::detach_from_tty()?;
            }
            apply_run_as_pre_exec(&run_as)?;

            #[cfg(target_os = "linux")]
            {
                codex_utils_pty::process_group::set_parent_death_signal(parent_pid)?;
            }
            Ok(())
        });
    }
}

fn configure_unix_pre_exec_no_run_as(cmd: &mut Command, stdio_policy: StdioPolicy) {
    unsafe {
        let detach_from_tty = matches!(stdio_policy, StdioPolicy::RedirectForShellTool);
        #[cfg(target_os = "linux")]
        let parent_pid = libc::getpid();
        cmd.pre_exec(move || {
            if detach_from_tty {
                codex_utils_pty::process_group::detach_from_tty()?;
            }

            #[cfg(target_os = "linux")]
            {
                codex_utils_pty::process_group::set_parent_death_signal(parent_pid)?;
            }
            Ok(())
        });
    }
}

pub(crate) async fn spawn_child_async_with_run_as(
    request: SpawnChildRequest<'_>,
    run_as: RunAsUser,
) -> std::io::Result<Child> {
    let SpawnChildRequest {
        program,
        args,
        arg0,
        cwd,
        network_sandbox_policy,
        network,
        stdio_policy,
        mut env,
    } = request;

    let retry = RunAsRetry {
        run_as: run_as.clone(),
        arg0: arg0.map(String::from),
        program: program.to_string_lossy().to_string(),
        args: args.clone(),
        cwd: cwd.clone(),
        network_sandbox_policy,
        env: env.clone(),
    };

    let mut cmd = Command::new(&program);
    cmd.arg0(arg0.map_or_else(|| program.to_string_lossy().to_string(), String::from));
    cmd.args(args);
    cmd.current_dir(&cwd);
    if let Some(network) = network {
        network.apply_to_env(&mut env);
    }
    cmd.env_clear();
    cmd.envs(env);

    if !network_sandbox_policy.is_enabled() {
        cmd.env(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR, "1");
    }

    configure_unix_pre_exec(&mut cmd, stdio_policy, run_as);

    match stdio_policy {
        StdioPolicy::RedirectForShellTool => {
            cmd.stdin(Stdio::null());
            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        }
        StdioPolicy::Inherit => {
            cmd.stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
        }
    }

    cmd.kill_on_drop(true);
    match cmd.spawn() {
        Ok(child) => Ok(child),
        Err(err) => try_spawn_with_run_as_sudo(retry, stdio_policy, err),
    }
}

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
    if !retry.network_sandbox_policy.is_enabled() {
        sudo_cmd.arg(format!("{CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR}=1"));
    }
    sudo_cmd.arg(program);
    sudo_cmd.args(retry.args);
    sudo_cmd.current_dir(retry.cwd);
    sudo_cmd.env_clear();
    configure_unix_pre_exec_no_run_as(&mut sudo_cmd, stdio_policy);

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
