use std::collections::HashMap;

use crate::sandboxing::ExecRequest;
use crate::unified_exec::SpawnLifecycleHandle;
use crate::unified_exec::UnifiedExecError;
use crate::unified_exec::UnifiedExecProcess;
use crate::unified_exec::UnifiedExecProcessManager;

#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::Stdio;
#[cfg(unix)]
use tokio::process::Command;

#[cfg(unix)]
use crate::spawn::RunAsUser;

struct PreparedPtyCommand {
    program: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    arg0: Option<String>,
}

#[cfg(unix)]
fn unix_preferred_executable_path(preferred: &[&str], fallback: &str) -> String {
    for candidate in preferred {
        if Path::new(candidate).exists() {
            return (*candidate).to_string();
        }
    }
    fallback.to_string()
}

#[cfg(unix)]
async fn sudo_preflight_check(run_as: &RunAsUser) -> Result<(), String> {
    let sudo_program = unix_preferred_executable_path(
        &["/usr/bin/sudo", "/bin/sudo", "/usr/local/bin/sudo"],
        "sudo",
    );
    let env_program =
        unix_preferred_executable_path(&["/usr/bin/env", "/bin/env", "/usr/local/bin/env"], "env");
    let id_program =
        unix_preferred_executable_path(&["/usr/bin/id", "/bin/id", "/usr/local/bin/id"], "id");

    let uid = format!("#{}", run_as.uid);
    let gid = format!("#{}", run_as.gid);
    let output = Command::new(&sudo_program)
        .args([
            "-n",
            "-u",
            uid.as_str(),
            "-g",
            gid.as_str(),
            "--",
            env_program.as_str(),
            "-i",
            id_program.as_str(),
            "-u",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|err| format!("failed to run `{sudo_program}`: {err}"))?;

    if !output.status.success() {
        let code = output.status.code();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!(
            "`{sudo_program} -n -u '{uid}' -g '{gid}' -- {env_program} -i {id_program} -u` failed (exit={code:?}) stderr={stderr:?}"
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let parsed_uid = stdout.parse::<u32>().map_err(|err| {
        format!("failed to parse `{sudo_program} ... id -u` output `{stdout}`: {err}")
    })?;
    if parsed_uid != run_as.uid {
        return Err(format!(
            "`{sudo_program} ... id -u` returned {stdout:?} (expected {})",
            run_as.uid
        ));
    }

    Ok(())
}

#[cfg(unix)]
fn ensure_argv0_symlink(program: &str, arg0: &str) -> std::io::Result<String> {
    let arg0 = std::path::Path::new(arg0)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| std::io::Error::other("arg0 must be a valid UTF-8 file name"))?;
    if arg0.is_empty() {
        return Err(std::io::Error::other("arg0 must be non-empty"));
    }

    let dir = std::path::PathBuf::from("/tmp/codex-argv0");
    std::fs::create_dir_all(&dir)?;
    let link_path = dir.join(arg0);

    if let Ok(target) = std::fs::read_link(&link_path) {
        if target == std::path::Path::new(program) {
            return Ok(link_path.to_string_lossy().to_string());
        }
    }
    let _ = std::fs::remove_file(&link_path);
    std::os::unix::fs::symlink(program, &link_path)?;
    Ok(link_path.to_string_lossy().to_string())
}

#[cfg(unix)]
fn prepare_pty_command(exec_env: &ExecRequest) -> Result<PreparedPtyCommand, UnifiedExecError> {
    let (program, args) = exec_env
        .command
        .split_first()
        .ok_or(UnifiedExecError::MissingCommandLine)?;

    let Some(run_as) = exec_env.run_as.as_ref() else {
        return Ok(PreparedPtyCommand {
            program: program.to_string(),
            args: args.to_vec(),
            env: exec_env.env.clone(),
            arg0: exec_env.arg0.clone(),
        });
    };

    let sudo_program = unix_preferred_executable_path(
        &["/usr/bin/sudo", "/bin/sudo", "/usr/local/bin/sudo"],
        "sudo",
    );
    let env_program =
        unix_preferred_executable_path(&["/usr/bin/env", "/bin/env", "/usr/local/bin/env"], "env");

    // `sudo` resets most of the environment by default, so we pass the desired
    // environment explicitly via `env -i KEY=VALUE ...`.
    let mut sudo_args = vec![
        "-n".to_string(),
        "-u".to_string(),
        format!("#{}", run_as.uid),
        "-g".to_string(),
        format!("#{}", run_as.gid),
        "--".to_string(),
        env_program,
        "-i".to_string(),
    ];

    let mut env_kv: Vec<_> = exec_env.env.iter().collect();
    env_kv.sort_by_key(|(key, _)| *key);
    sudo_args.extend(
        env_kv
            .into_iter()
            .map(|(key, value)| format!("{key}={value}")),
    );

    sudo_args.push(program.to_string());
    sudo_args.extend(args.iter().cloned());

    Ok(PreparedPtyCommand {
        program: sudo_program,
        args: sudo_args,
        env: HashMap::new(),
        arg0: None,
    })
}

#[cfg(not(unix))]
fn prepare_pty_command(exec_env: &ExecRequest) -> Result<PreparedPtyCommand, UnifiedExecError> {
    let (program, args) = exec_env
        .command
        .split_first()
        .ok_or(UnifiedExecError::MissingCommandLine)?;
    Ok(PreparedPtyCommand {
        program: program.to_string(),
        args: args.to_vec(),
        env: exec_env.env.clone(),
        arg0: exec_env.arg0.clone(),
    })
}

impl UnifiedExecProcessManager {
    #[cfg(unix)]
    async fn cached_exec_command_sudo_preflight(&self, run_as: RunAsUser) -> Result<(), String> {
        {
            let guard = self.sudo_preflight.lock().await;
            if let Some(state) = guard.as_ref().filter(|state| {
                state.run_as.uid == run_as.uid
                    && state.run_as.gid == run_as.gid
                    && state.run_as.supplementary_gids == run_as.supplementary_gids
            }) {
                return state.result.clone();
            }
        }

        let result = sudo_preflight_check(&run_as).await;
        let mut guard = self.sudo_preflight.lock().await;
        let warned = guard
            .as_ref()
            .filter(|state| {
                state.run_as.uid == run_as.uid
                    && state.run_as.gid == run_as.gid
                    && state.run_as.supplementary_gids == run_as.supplementary_gids
            })
            .is_some_and(|state| state.warned);
        guard.replace(super::SudoPreflightState {
            run_as,
            result: result.clone(),
            warned,
        });
        result
    }

    #[cfg(not(unix))]
    async fn cached_exec_command_sudo_preflight(&self, _run_as: RunAsUser) -> Result<(), String> {
        Ok(())
    }

    pub(crate) async fn preflight_exec_command_sudo_worker_user(
        &self,
        run_as: RunAsUser,
    ) -> Result<(), UnifiedExecError> {
        self.cached_exec_command_sudo_preflight(run_as)
            .await
            .map_err(|err| {
                UnifiedExecError::create_process(format!(
                    "exec_command requires passwordless sudo to run as the configured worker user: {err}"
                ))
            })
    }

    #[cfg(unix)]
    pub(crate) async fn exec_command_sudo_worker_user_startup_warning(
        &self,
        run_as: RunAsUser,
    ) -> Option<String> {
        {
            let mut guard = self.sudo_preflight.lock().await;
            if let Some(state) = guard.as_mut().filter(|state| {
                state.run_as.uid == run_as.uid
                    && state.run_as.gid == run_as.gid
                    && state.run_as.supplementary_gids == run_as.supplementary_gids
            }) {
                if state.warned {
                    return None;
                }
                if let Err(err) = state.result.as_ref() {
                    state.warned = true;
                    return Some(err.clone());
                }
                return None;
            }
        }

        match sudo_preflight_check(&run_as).await {
            Ok(()) => {
                let mut guard = self.sudo_preflight.lock().await;
                guard.replace(super::SudoPreflightState {
                    run_as,
                    result: Ok(()),
                    warned: false,
                });
                None
            }
            Err(err) => {
                let mut guard = self.sudo_preflight.lock().await;
                guard.replace(super::SudoPreflightState {
                    run_as,
                    result: Err(err.clone()),
                    warned: true,
                });
                Some(err)
            }
        }
    }

    #[cfg(not(unix))]
    pub(crate) async fn exec_command_sudo_worker_user_startup_warning(
        &self,
        _run_as: RunAsUser,
    ) -> Option<String> {
        None
    }

    pub(crate) async fn open_session_with_exec_env_custom(
        &self,
        env: &ExecRequest,
        tty: bool,
        mut spawn_lifecycle: SpawnLifecycleHandle,
    ) -> Result<UnifiedExecProcess, UnifiedExecError> {
        let mut env_override = None;
        #[cfg(unix)]
        {
            if env.run_as.is_some()
                && let Some(arg0) = env.arg0.as_deref()
                && let Some((program, _)) = env.command.split_first()
            {
                let program_name = std::path::Path::new(program).file_name();
                let arg0_name = std::path::Path::new(arg0).file_name();
                if program_name != arg0_name {
                    let link_program = ensure_argv0_symlink(program, arg0).map_err(|err| {
                        UnifiedExecError::create_process(format!(
                            "failed to create argv0 symlink for arg0 `{arg0}`: {err}"
                        ))
                    })?;
                    let mut command = env.command.clone();
                    command[0] = link_program;
                    env_override = Some(ExecRequest {
                        command,
                        cwd: env.cwd.clone(),
                        env: env.env.clone(),
                        network: env.network.clone(),
                        expiration: env.expiration.clone(),
                        sandbox: env.sandbox,
                        windows_sandbox_level: env.windows_sandbox_level,
                        windows_sandbox_private_desktop: env.windows_sandbox_private_desktop,
                        run_as: env.run_as.clone(),
                        sandbox_permissions: env.sandbox_permissions.clone(),
                        sandbox_policy: env.sandbox_policy.clone(),
                        file_system_sandbox_policy: env.file_system_sandbox_policy.clone(),
                        network_sandbox_policy: env.network_sandbox_policy,
                        justification: env.justification.clone(),
                        arg0: None,
                    });
                }
            }
        }
        let env = env_override.as_ref().unwrap_or(env);

        #[cfg(unix)]
        if let Some(run_as) = env.run_as.clone() {
            self.preflight_exec_command_sudo_worker_user(run_as).await?;
        }

        let prepared = prepare_pty_command(env)?;

        let spawn_result = if tty {
            codex_utils_pty::pty::spawn_process(
                prepared.program.as_str(),
                prepared.args.as_slice(),
                env.cwd.as_path(),
                &prepared.env,
                &prepared.arg0,
                codex_utils_pty::TerminalSize::default(),
            )
            .await
        } else {
            codex_utils_pty::pipe::spawn_process_no_stdin(
                prepared.program.as_str(),
                prepared.args.as_slice(),
                env.cwd.as_path(),
                &prepared.env,
                &prepared.arg0,
            )
            .await
        };
        let spawned =
            spawn_result.map_err(|err| UnifiedExecError::create_process(err.to_string()))?;
        spawn_lifecycle.after_spawn();
        UnifiedExecProcess::from_spawned(spawned, env.sandbox, spawn_lifecycle).await
    }
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use super::*;
    use crate::exec::ExecExpiration;
    use crate::exec::SandboxType;
    use crate::protocol::SandboxPolicy;
    use crate::sandboxing::SandboxPermissions;
    use codex_protocol::config_types::WindowsSandboxLevel;
    use codex_protocol::permissions::FileSystemSandboxPolicy;
    use codex_protocol::permissions::NetworkSandboxPolicy;
    use pretty_assertions::assert_eq;

    fn exec_request_with_run_as() -> ExecRequest {
        ExecRequest {
            command: vec!["echo".to_string(), "hello".to_string()],
            cwd: std::path::PathBuf::from("/tmp"),
            env: HashMap::from([("A".to_string(), "B".to_string())]),
            network: None,
            expiration: ExecExpiration::DefaultTimeout,
            sandbox: SandboxType::None,
            windows_sandbox_level: WindowsSandboxLevel::Disabled,
            windows_sandbox_private_desktop: false,
            run_as: Some(RunAsUser {
                uid: 123,
                gid: 456,
                supplementary_gids: Some(vec![789]),
            }),
            sandbox_permissions: SandboxPermissions::UseDefault,
            sandbox_policy: SandboxPolicy::DangerFullAccess,
            file_system_sandbox_policy: FileSystemSandboxPolicy::unrestricted(),
            network_sandbox_policy: NetworkSandboxPolicy::Enabled,
            justification: None,
            arg0: None,
        }
    }

    #[test]
    fn prepare_pty_command_run_as_wraps_with_sudo() {
        let exec_env = exec_request_with_run_as();
        let prepared = prepare_pty_command(&exec_env).expect("prepare");

        assert!(
            prepared.program.ends_with("sudo"),
            "program={}",
            prepared.program
        );
        assert_eq!(prepared.env, HashMap::new());
        assert_eq!(prepared.arg0, None);
        assert!(prepared.args.iter().any(|arg| arg == "-n"));
        assert!(prepared.args.iter().any(|arg| arg == "-u"));
        assert!(prepared.args.iter().any(|arg| arg == "#123"));
        assert!(prepared.args.iter().any(|arg| arg == "-g"));
        assert!(prepared.args.iter().any(|arg| arg == "#456"));
        assert!(prepared.args.iter().any(|arg| arg == "echo"));
        assert!(prepared.args.iter().any(|arg| arg == "hello"));
    }

    #[test]
    fn prepare_pty_command_run_as_does_not_use_env_argv0_flags() {
        let mut exec_env = exec_request_with_run_as();
        exec_env.arg0 = Some("custom-argv0".to_string());
        let prepared = prepare_pty_command(&exec_env).expect("prepare");

        assert_eq!(prepared.arg0, None);
    }
}
