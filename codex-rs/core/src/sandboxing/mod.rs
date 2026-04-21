/*
Module: sandboxing

Build platform wrappers and produce ExecRequest for execution. Owns low-level
sandbox placement and transformation of portable CommandSpec into a
ready‑to‑spawn environment.
*/

use crate::exec::ExecCapturePolicy;
use crate::exec::ExecExpiration;
use crate::exec::SandboxType;
use crate::exec::StdoutStream;
use crate::exec::WindowsSandboxFilesystemOverrides;
use crate::exec::execute_exec_request;
use crate::protocol::SandboxPolicy;
#[cfg(target_os = "macos")]
use crate::spawn::CODEX_SANDBOX_ENV_VAR;
use crate::spawn::CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR;
use crate::spawn::RunAsUser;
use crate::tools::sandboxing::SandboxablePreference;
use codex_network_proxy::NetworkProxy;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::models::PermissionProfile;
pub use codex_protocol::models::SandboxPermissions;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_sandboxing::landlock::allow_network_for_proxy;
use codex_sandboxing::landlock::create_linux_sandbox_command_args_for_policies;
use codex_sandboxing::policy_transforms::EffectiveSandboxPermissions;
use codex_sandboxing::policy_transforms::effective_file_system_sandbox_policy;
use codex_sandboxing::policy_transforms::effective_network_sandbox_policy;
use codex_sandboxing::policy_transforms::should_require_platform_sandbox;
#[cfg(target_os = "macos")]
use codex_sandboxing::seatbelt::MACOS_PATH_TO_SEATBELT_EXECUTABLE;
#[cfg(target_os = "macos")]
use codex_sandboxing::seatbelt::create_seatbelt_command_args_for_policies;
use codex_utils_absolute_path::AbsolutePathBuf;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use crate::exec::ExecToolCallOutput;

#[derive(Debug)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
    pub expiration: ExecExpiration,
    pub capture_policy: ExecCapturePolicy,
    pub run_as: Option<RunAsUser>,
    pub sandbox_permissions: SandboxPermissions,
    pub additional_permissions: Option<PermissionProfile>,
    pub justification: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct ExecServerEnvConfig {
    pub(crate) policy: codex_exec_server::ExecEnvPolicy,
    pub(crate) local_policy_env: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ExecRequest {
    pub command: Vec<String>,
    pub cwd: AbsolutePathBuf,
    pub env: HashMap<String, String>,
    pub(crate) exec_server_env_config: Option<ExecServerEnvConfig>,
    pub network: Option<NetworkProxy>,
    pub expiration: ExecExpiration,
    pub capture_policy: ExecCapturePolicy,
    pub sandbox: SandboxType,
    pub windows_sandbox_level: WindowsSandboxLevel,
    pub windows_sandbox_private_desktop: bool,
    pub run_as: Option<RunAsUser>,
    pub sandbox_permissions: SandboxPermissions,
    pub sandbox_policy: SandboxPolicy,
    pub file_system_sandbox_policy: FileSystemSandboxPolicy,
    pub network_sandbox_policy: NetworkSandboxPolicy,
    pub(crate) windows_sandbox_filesystem_overrides: Option<WindowsSandboxFilesystemOverrides>,
    pub justification: Option<String>,
    pub arg0: Option<String>,
}

impl ExecRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        command: Vec<String>,
        cwd: AbsolutePathBuf,
        env: HashMap<String, String>,
        network: Option<NetworkProxy>,
        expiration: ExecExpiration,
        capture_policy: ExecCapturePolicy,
        sandbox: SandboxType,
        windows_sandbox_level: WindowsSandboxLevel,
        windows_sandbox_private_desktop: bool,
        run_as: Option<RunAsUser>,
        sandbox_permissions: SandboxPermissions,
        sandbox_policy: SandboxPolicy,
        file_system_sandbox_policy: FileSystemSandboxPolicy,
        network_sandbox_policy: NetworkSandboxPolicy,
        justification: Option<String>,
        arg0: Option<String>,
    ) -> Self {
        Self {
            command,
            cwd,
            env,
            exec_server_env_config: None,
            network,
            expiration,
            capture_policy,
            sandbox,
            windows_sandbox_level,
            windows_sandbox_private_desktop,
            run_as,
            sandbox_permissions,
            sandbox_policy,
            file_system_sandbox_policy,
            network_sandbox_policy,
            windows_sandbox_filesystem_overrides: None,
            justification,
            arg0,
        }
    }
}

/// Bundled arguments for sandbox transformation.
///
/// This keeps call sites self-documenting when several fields are optional.
pub(crate) struct SandboxTransformRequest<'a> {
    pub spec: CommandSpec,
    pub policy: &'a SandboxPolicy,
    pub file_system_policy: &'a FileSystemSandboxPolicy,
    pub network_policy: NetworkSandboxPolicy,
    pub sandbox: SandboxType,
    pub enforce_managed_network: bool,
    // TODO(viyatb): Evaluate switching this to Option<Arc<NetworkProxy>>
    // to make shared ownership explicit across runtime/sandbox plumbing.
    pub network: Option<&'a NetworkProxy>,
    pub sandbox_policy_cwd: &'a Path,
    pub codex_linux_sandbox_exe: Option<&'a PathBuf>,
    pub use_legacy_landlock: bool,
    pub windows_sandbox_level: WindowsSandboxLevel,
    pub windows_sandbox_private_desktop: bool,
}

pub enum SandboxPreference {
    Auto,
    Require,
    Forbid,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum SandboxTransformError {
    #[error("missing codex-linux-sandbox executable path")]
    MissingLinuxSandboxExecutable,
    #[cfg(not(target_os = "macos"))]
    #[error("seatbelt sandbox is only available on macOS")]
    SeatbeltUnavailable,
}

#[derive(Default)]
pub struct SandboxManager;

impl SandboxManager {
    pub fn new() -> Self {
        Self
    }

    pub(crate) fn select_initial(
        &self,
        file_system_policy: &FileSystemSandboxPolicy,
        network_policy: NetworkSandboxPolicy,
        pref: SandboxablePreference,
        windows_sandbox_level: WindowsSandboxLevel,
        has_managed_network_requirements: bool,
    ) -> SandboxType {
        match pref {
            SandboxablePreference::Forbid => SandboxType::None,
            SandboxablePreference::Require => {
                // Require a platform sandbox when available; on Windows this
                // respects the experimental_windows_sandbox feature.
                codex_sandboxing::get_platform_sandbox(
                    windows_sandbox_level != WindowsSandboxLevel::Disabled,
                )
                .map(Into::into)
                .unwrap_or(SandboxType::None)
            }
            SandboxablePreference::Auto => {
                if should_require_platform_sandbox(
                    file_system_policy,
                    network_policy,
                    has_managed_network_requirements,
                ) {
                    codex_sandboxing::get_platform_sandbox(
                        windows_sandbox_level != WindowsSandboxLevel::Disabled,
                    )
                    .map(Into::into)
                    .unwrap_or(SandboxType::None)
                } else {
                    SandboxType::None
                }
            }
        }
    }

    pub(crate) fn transform(
        &self,
        request: SandboxTransformRequest<'_>,
    ) -> Result<ExecRequest, SandboxTransformError> {
        let SandboxTransformRequest {
            mut spec,
            policy,
            file_system_policy,
            network_policy,
            sandbox,
            enforce_managed_network,
            network,
            sandbox_policy_cwd,
            codex_linux_sandbox_exe,
            use_legacy_landlock,
            windows_sandbox_level,
            windows_sandbox_private_desktop,
        } = request;
        let additional_permissions = spec.additional_permissions.take();
        let EffectiveSandboxPermissions {
            sandbox_policy: effective_policy,
        } = EffectiveSandboxPermissions::new(policy, additional_permissions.as_ref());
        let effective_file_system_policy = effective_file_system_sandbox_policy(
            file_system_policy,
            additional_permissions.as_ref(),
        );
        let effective_network_policy =
            effective_network_sandbox_policy(network_policy, additional_permissions.as_ref());
        let mut env = spec.env;
        if !effective_network_policy.is_enabled() {
            env.insert(
                CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR.to_string(),
                "1".to_string(),
            );
        }

        let mut command = Vec::with_capacity(1 + spec.args.len());
        command.push(spec.program);
        command.append(&mut spec.args);

        let (command, sandbox_env, arg0_override) = match sandbox {
            SandboxType::None => (command, HashMap::new(), None),
            #[cfg(target_os = "macos")]
            SandboxType::MacosSeatbelt => {
                let mut seatbelt_env = HashMap::new();
                seatbelt_env.insert(CODEX_SANDBOX_ENV_VAR.to_string(), "seatbelt".to_string());
                let mut args = create_seatbelt_command_args_for_policies(
                    command.clone(),
                    &effective_file_system_policy,
                    effective_network_policy,
                    sandbox_policy_cwd,
                    enforce_managed_network,
                    network,
                );
                let mut full_command = Vec::with_capacity(1 + args.len());
                full_command.push(MACOS_PATH_TO_SEATBELT_EXECUTABLE.to_string());
                full_command.append(&mut args);
                (full_command, seatbelt_env, None)
            }
            #[cfg(not(target_os = "macos"))]
            SandboxType::MacosSeatbelt => return Err(SandboxTransformError::SeatbeltUnavailable),
            SandboxType::LinuxSeccomp => {
                let exe = codex_linux_sandbox_exe
                    .ok_or(SandboxTransformError::MissingLinuxSandboxExecutable)?;
                let allow_proxy_network = allow_network_for_proxy(enforce_managed_network);
                let mut args = create_linux_sandbox_command_args_for_policies(
                    command.clone(),
                    spec.cwd.as_path(),
                    &effective_policy,
                    &effective_file_system_policy,
                    effective_network_policy,
                    sandbox_policy_cwd,
                    use_legacy_landlock,
                    allow_proxy_network,
                );
                let mut full_command = Vec::with_capacity(1 + args.len());
                full_command.push(exe.to_string_lossy().to_string());
                full_command.append(&mut args);
                (
                    full_command,
                    HashMap::new(),
                    Some("codex-linux-sandbox".to_string()),
                )
            }
            // On Windows, the restricted token sandbox executes in-process via the
            // codex-windows-sandbox crate. We leave the command unchanged here and
            // branch during execution based on the sandbox type.
            #[cfg(target_os = "windows")]
            SandboxType::WindowsRestrictedToken => (command, HashMap::new(), None),
            // When building for non-Windows targets, this variant is never constructed.
            #[cfg(not(target_os = "windows"))]
            SandboxType::WindowsRestrictedToken => (command, HashMap::new(), None),
        };

        env.extend(sandbox_env);

        Ok(ExecRequest {
            command,
            cwd: AbsolutePathBuf::from_absolute_path_checked(spec.cwd)
                .expect("command cwd must be absolute"),
            env,
            exec_server_env_config: None,
            network: network.cloned(),
            expiration: spec.expiration,
            capture_policy: spec.capture_policy,
            sandbox,
            windows_sandbox_level,
            windows_sandbox_private_desktop,
            run_as: spec.run_as,
            sandbox_permissions: spec.sandbox_permissions,
            sandbox_policy: effective_policy,
            file_system_sandbox_policy: effective_file_system_policy,
            network_sandbox_policy: effective_network_policy,
            windows_sandbox_filesystem_overrides: None,
            justification: spec.justification,
            arg0: arg0_override,
        })
    }
}

pub async fn execute_env(
    exec_request: ExecRequest,
    stdout_stream: Option<StdoutStream>,
) -> crate::error::Result<ExecToolCallOutput> {
    let effective_policy = exec_request.sandbox_policy.clone();
    execute_exec_request(
        exec_request,
        &effective_policy,
        stdout_stream,
        /*after_spawn*/ None,
    )
    .await
}

pub async fn execute_exec_request_with_after_spawn(
    exec_request: ExecRequest,
    stdout_stream: Option<StdoutStream>,
    after_spawn: Option<Box<dyn FnOnce() + Send>>,
) -> crate::error::Result<ExecToolCallOutput> {
    let effective_policy = exec_request.sandbox_policy.clone();
    execute_exec_request(exec_request, &effective_policy, stdout_stream, after_spawn).await
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
