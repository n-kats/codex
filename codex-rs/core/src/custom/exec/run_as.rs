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
pub(crate) fn run_as_sudo_command(retry: RunAsRetry) -> std::io::Result<Command> {
    let Some(sudo_program) = ["/usr/bin/sudo", "/bin/sudo"]
        .into_iter()
        .find(|path| std::path::Path::new(path).exists())
    else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "sudo is not installed",
        ));
    };
    let Some(env_program) = ["/usr/bin/env", "/bin/env"]
        .into_iter()
        .find(|path| std::path::Path::new(path).exists())
    else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "env is not installed",
        ));
    };

    build_sudo_command(retry, sudo_program, env_program)
}

#[cfg(unix)]
fn build_sudo_command(
    retry: RunAsRetry,
    sudo_program: &str,
    env_program: &str,
) -> std::io::Result<Command> {
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

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn run_as_sudo_command_builds_expected_command_line() {
        let retry = RunAsRetry::new(
            RunAsUser {
                uid: 1234,
                gid: 5678,
                supplementary_gids: Some(vec![5678, 9012]),
            },
            Some("custom-arg0".to_string()),
            "/bin/echo".to_string(),
            vec!["hello".to_string(), "world".to_string()],
            std::env::current_dir().expect("current dir"),
            HashMap::from([
                ("B".to_string(), "2".to_string()),
                ("A".to_string(), "1".to_string()),
            ]),
        );

        let command =
            build_sudo_command(retry, "/tmp/sudo", "/tmp/env").expect("sudo command should build");
        let debug = format!("{command:?}");
        assert!(debug.contains("/tmp/sudo"), "{debug}");
        assert!(debug.contains("/tmp/env"), "{debug}");
        assert!(debug.contains("#1234"), "{debug}");
        assert!(debug.contains("#5678"), "{debug}");
        assert!(debug.contains("A=1"), "{debug}");
        assert!(debug.contains("B=2"), "{debug}");
        assert!(debug.contains("hello"), "{debug}");
        assert!(debug.contains("world"), "{debug}");
        assert!(debug.contains("/tmp/codex-run-as-argv0-"), "{debug}");
        assert!(debug.contains("custom-arg0"), "{debug}");
    }
}
