#[cfg(unix)]
use std::collections::HashMap;
#[cfg(unix)]
use std::ffi::OsStr;
#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use tokio::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunAsUser {
    pub uid: u32,
    pub gid: u32,
    pub supplementary_gids: Option<Vec<u32>>,
}

#[cfg(unix)]
#[derive(Clone)]
pub(crate) struct RunAsRetry {
    run_as: RunAsUser,
    arg0: Option<String>,
    program: String,
    args: Vec<String>,
    cwd: PathBuf,
    env: HashMap<String, String>,
}

#[cfg(unix)]
impl RunAsRetry {
    pub(crate) fn new(
        run_as: RunAsUser,
        arg0: Option<String>,
        program: String,
        args: Vec<String>,
        cwd: PathBuf,
        env: HashMap<String, String>,
    ) -> Self {
        Self {
            run_as,
            arg0,
            program,
            args,
            cwd,
            env,
        }
    }
}

#[cfg(unix)]
pub(crate) fn apply_unix_run_as(run_as: &RunAsUser) -> std::io::Result<()> {
    unsafe {
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
    Ok(())
}

#[cfg(unix)]
fn ensure_argv0_symlink(program: &str, arg0: &str) -> std::io::Result<PathBuf> {
    use std::os::unix::fs::symlink;

    let temp_dir = tempfile::Builder::new()
        .prefix("codex-run-as-argv0-")
        .tempdir()?;
    let temp_dir = temp_dir.keep();
    let link_path = temp_dir.join(OsStr::new(arg0));
    symlink(program, &link_path)?;
    Ok(link_path)
}

#[cfg(unix)]
pub(crate) fn run_as_sudo_fallback_command(
    retry: RunAsRetry,
    err: std::io::Error,
) -> std::io::Result<Command> {
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
    sudo_cmd.args(&retry.args);
    sudo_cmd.current_dir(&retry.cwd);
    sudo_cmd.env_clear();
    Ok(sudo_cmd)
}
