//! Apply Patch runtime: executes verified patches under the orchestrator.
//!
//! Assumes `apply_patch` verification/approval happened upstream. Reuses the
//! selected turn environment filesystem for both local and remote turns, with
//! sandboxing enforced by the explicit filesystem sandbox context.
use crate::exec::is_likely_sandbox_denied;
use crate::guardian::GuardianApprovalRequest;
use crate::guardian::review_approval_request;
use crate::spawn::RunAsRetry;
use crate::spawn::{self};
use crate::tools::hook_names::HookToolName;
use crate::tools::sandboxing::Approvable;
use crate::tools::sandboxing::ApprovalCtx;
use crate::tools::sandboxing::ExecApprovalRequirement;
use crate::tools::sandboxing::PermissionRequestPayload;
use crate::tools::sandboxing::SandboxAttempt;
use crate::tools::sandboxing::Sandboxable;
use crate::tools::sandboxing::ToolCtx;
use crate::tools::sandboxing::ToolError;
use crate::tools::sandboxing::ToolRuntime;
use crate::tools::sandboxing::with_cached_approval;
use codex_apply_patch::AppliedPatchDelta;
use codex_apply_patch::ApplyPatchAction;
use codex_exec_server::FileSystemSandboxContext;
use codex_protocol::error::CodexErr;
use codex_protocol::error::SandboxErr;
use codex_protocol::exec_output::ExecToolCallOutput;
use codex_protocol::exec_output::StreamOutput;
use codex_protocol::models::AdditionalPermissionProfile;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::FileChange;
use codex_protocol::protocol::ReviewDecision;
use codex_sandboxing::SandboxType;
use codex_sandboxing::SandboxablePreference;
use codex_sandboxing::policy_transforms::effective_permission_profile;
use codex_utils_absolute_path::AbsolutePathBuf;
use futures::future::BoxFuture;
#[cfg(unix)]
use std::collections::HashMap;
#[cfg(unix)]
use std::io;
#[cfg(unix)]
use std::io::Write;
use std::path::PathBuf;
#[cfg(unix)]
use std::process::Stdio;
use std::time::Instant;
#[cfg(unix)]
use tempfile::NamedTempFile;

#[derive(Debug)]
pub struct ApplyPatchRequest {
    pub action: ApplyPatchAction,
    pub file_paths: Vec<AbsolutePathBuf>,
    pub changes: std::collections::HashMap<PathBuf, FileChange>,
    pub exec_approval_requirement: ExecApprovalRequirement,
    pub additional_permissions: Option<AdditionalPermissionProfile>,
    pub permissions_preapproved: bool,
    pub turn_environment: crate::session::turn_context::TurnEnvironment,
    pub run_as: Option<crate::spawn::RunAsUser>,
}

#[derive(Default)]
pub struct ApplyPatchRuntime {
    committed_delta: AppliedPatchDelta,
}

#[derive(Debug)]
pub struct ApplyPatchRuntimeOutput {
    pub exec_output: ExecToolCallOutput,
    pub delta: AppliedPatchDelta,
}

impl ApplyPatchRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn committed_delta(&self) -> &AppliedPatchDelta {
        &self.committed_delta
    }

    fn build_guardian_review_request(
        req: &ApplyPatchRequest,
        call_id: &str,
    ) -> GuardianApprovalRequest {
        GuardianApprovalRequest::ApplyPatch {
            id: call_id.to_string(),
            cwd: req.action.cwd.clone(),
            files: req.file_paths.clone(),
            patch: req.action.patch.clone(),
        }
    }

    fn file_system_sandbox_context_for_attempt(
        req: &ApplyPatchRequest,
        attempt: &SandboxAttempt<'_>,
    ) -> Option<FileSystemSandboxContext> {
        if attempt.sandbox == SandboxType::None {
            return None;
        }

        let permissions =
            effective_permission_profile(attempt.permissions, req.additional_permissions.as_ref());
        Some(FileSystemSandboxContext {
            permissions,
            cwd: Some(attempt.sandbox_cwd.clone()),
            windows_sandbox_level: attempt.windows_sandbox_level,
            windows_sandbox_private_desktop: attempt.windows_sandbox_private_desktop,
            use_legacy_landlock: attempt.use_legacy_landlock,
        })
    }

    #[cfg(unix)]
    fn configure_run_as(cmd: &mut tokio::process::Command, run_as: &crate::spawn::RunAsUser) {
        let run_as = run_as.clone();
        unsafe {
            cmd.pre_exec(move || {
                if let Some(supplementary_gids) = &run_as.supplementary_gids {
                    let supplementary_gids = supplementary_gids
                        .iter()
                        .copied()
                        .map(|gid| gid as libc::gid_t)
                        .collect::<Vec<_>>();
                    if libc::setgroups(supplementary_gids.len(), supplementary_gids.as_ptr()) != 0 {
                        return Err(io::Error::last_os_error());
                    }
                }
                if libc::setgid(run_as.gid as libc::gid_t) != 0 {
                    return Err(io::Error::last_os_error());
                }
                if libc::setuid(run_as.uid as libc::uid_t) != 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    async fn spawn_apply_patch_child_with_run_as(
        req: &ApplyPatchRequest,
        run_as: &crate::spawn::RunAsUser,
        codex_self_exe: &std::path::Path,
        stdin_file: std::fs::File,
    ) -> io::Result<std::process::Output> {
        let mut cmd = tokio::process::Command::new(codex_self_exe);
        cmd.arg0("apply_patch");
        cmd.current_dir(&req.action.cwd);
        cmd.stdin(Stdio::from(stdin_file.try_clone()?));
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        Self::configure_run_as(&mut cmd, run_as);

        match cmd.spawn() {
            Ok(child) => child.wait_with_output().await,
            Err(err)
                if unsafe { libc::geteuid() } != 0
                    && (err.kind() == io::ErrorKind::PermissionDenied
                        || err.raw_os_error() == Some(libc::EPERM)) =>
            {
                let retry = RunAsRetry::new(
                    run_as.clone(),
                    Some("apply_patch".to_string()),
                    codex_self_exe.to_string_lossy().to_string(),
                    Vec::new(),
                    req.action.cwd.clone().to_path_buf(),
                    std::env::vars().collect::<HashMap<_, _>>(),
                );
                let Some(mut sudo_cmd) = spawn::build_run_as_sudo_command(&retry)? else {
                    return Err(err);
                };
                sudo_cmd.current_dir(&req.action.cwd);
                sudo_cmd.stdin(Stdio::from(stdin_file));
                sudo_cmd.stdout(Stdio::piped());
                sudo_cmd.stderr(Stdio::piped());

                let child = sudo_cmd.spawn()?;
                child.wait_with_output().await
            }
            Err(err) => Err(err),
        }
    }

    #[cfg(unix)]
    async fn run_apply_patch_under_run_as(
        req: &ApplyPatchRequest,
        run_as: &crate::spawn::RunAsUser,
        codex_self_exe: &std::path::Path,
    ) -> Result<ApplyPatchRuntimeOutput, ToolError> {
        let mut patch_file = NamedTempFile::new().map_err(|err| {
            ToolError::Codex(CodexErr::Io(io::Error::other(format!(
                "failed to create temporary apply_patch input: {err}"
            ))))
        })?;
        patch_file
            .write_all(req.action.patch.as_bytes())
            .map_err(|err| {
                ToolError::Codex(CodexErr::Io(io::Error::other(format!(
                    "failed to write temporary apply_patch input: {err}"
                ))))
            })?;
        patch_file.flush().map_err(|err| {
            ToolError::Codex(CodexErr::Io(io::Error::other(format!(
                "failed to flush temporary apply_patch input: {err}"
            ))))
        })?;
        let stdin_file = patch_file.reopen().map_err(|err| {
            ToolError::Codex(CodexErr::Io(io::Error::other(format!(
                "failed to reopen temporary apply_patch input: {err}"
            ))))
        })?;
        let output =
            Self::spawn_apply_patch_child_with_run_as(req, run_as, codex_self_exe, stdin_file)
                .await
                .map_err(|err| {
                    ToolError::Codex(CodexErr::Io(io::Error::other(format!(
                        "failed to spawn apply_patch helper under run_as: {err}"
                    ))))
                })?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let exit_code = output.status.code().unwrap_or(-1);
        let exec_output = ExecToolCallOutput {
            exit_code,
            stdout: StreamOutput::new(stdout.clone()),
            stderr: StreamOutput::new(stderr.clone()),
            aggregated_output: StreamOutput::new(format!("{stdout}{stderr}")),
            duration: Instant::now().elapsed(),
            timed_out: false,
        };
        Ok(ApplyPatchRuntimeOutput {
            exec_output,
            delta: AppliedPatchDelta::default(),
        })
    }
}

impl Sandboxable for ApplyPatchRuntime {
    fn sandbox_preference(&self) -> SandboxablePreference {
        SandboxablePreference::Auto
    }
    fn escalate_on_failure(&self) -> bool {
        true
    }
}

impl Approvable<ApplyPatchRequest> for ApplyPatchRuntime {
    type ApprovalKey = AbsolutePathBuf;

    fn approval_keys(&self, req: &ApplyPatchRequest) -> Vec<Self::ApprovalKey> {
        req.file_paths.clone()
    }

    fn start_approval_async<'a>(
        &'a mut self,
        req: &'a ApplyPatchRequest,
        ctx: ApprovalCtx<'a>,
    ) -> BoxFuture<'a, ReviewDecision> {
        let session = ctx.session;
        let turn = ctx.turn;
        let call_id = ctx.call_id.to_string();
        let retry_reason = ctx.retry_reason.clone();
        let approval_keys = self.approval_keys(req);
        let changes = req.changes.clone();
        let guardian_review_id = ctx.guardian_review_id.clone();
        Box::pin(async move {
            if let Some(review_id) = guardian_review_id {
                let action = ApplyPatchRuntime::build_guardian_review_request(req, ctx.call_id);
                return review_approval_request(session, turn, review_id, action, retry_reason)
                    .await;
            }
            if req.permissions_preapproved && retry_reason.is_none() {
                return ReviewDecision::Approved;
            }
            if let Some(reason) = retry_reason {
                let rx_approve = session
                    .request_patch_approval(
                        turn,
                        call_id,
                        changes.clone(),
                        Some(reason),
                        /*grant_root*/ None,
                    )
                    .await;
                return rx_approve.await.unwrap_or_default();
            }

            with_cached_approval(
                &session.services,
                "apply_patch",
                approval_keys,
                || async move {
                    let rx_approve = session
                        .request_patch_approval(
                            turn, call_id, changes, /*reason*/ None, /*grant_root*/ None,
                        )
                        .await;
                    rx_approve.await.unwrap_or_default()
                },
            )
            .await
        })
    }

    fn wants_no_sandbox_approval(&self, policy: AskForApproval) -> bool {
        match policy {
            AskForApproval::Never => false,
            AskForApproval::Granular(granular_config) => granular_config.allows_sandbox_approval(),
            AskForApproval::OnFailure => true,
            AskForApproval::OnRequest => true,
            AskForApproval::UnlessTrusted => true,
        }
    }

    // apply_patch approvals are decided upstream by assess_patch_safety.
    //
    // This override ensures the orchestrator runs the patch approval flow when required instead
    // of falling back to the global exec approval policy.
    fn exec_approval_requirement(
        &self,
        req: &ApplyPatchRequest,
    ) -> Option<ExecApprovalRequirement> {
        Some(req.exec_approval_requirement.clone())
    }

    fn permission_request_payload(
        &self,
        req: &ApplyPatchRequest,
    ) -> Option<PermissionRequestPayload> {
        Some(PermissionRequestPayload {
            tool_name: HookToolName::apply_patch(),
            tool_input: serde_json::json!({ "command": req.action.patch }),
        })
    }
}

impl ToolRuntime<ApplyPatchRequest, ApplyPatchRuntimeOutput> for ApplyPatchRuntime {
    async fn run(
        &mut self,
        req: &ApplyPatchRequest,
        attempt: &SandboxAttempt<'_>,
        ctx: &ToolCtx,
    ) -> Result<ApplyPatchRuntimeOutput, ToolError> {
        let environment = ctx.turn.environments.primary().ok_or_else(|| {
            ToolError::Rejected("apply_patch is unavailable in this session".to_string())
        })?;
        let started_at = Instant::now();
        #[cfg(unix)]
        if let Some(run_as) = req.run_as.as_ref()
            && !environment.environment.is_remote()
        {
            let Some(codex_self_exe) = ctx.turn.codex_self_exe.as_ref() else {
                return Err(ToolError::Rejected(
                    "apply_patch requires codex_self_exe to honor custom.exec.worker_user"
                        .to_string(),
                ));
            };
            let mut output =
                Self::run_apply_patch_under_run_as(req, run_as, codex_self_exe).await?;
            output.exec_output.duration = started_at.elapsed();
            return Ok(output);
        }
        let fs = environment.environment.get_filesystem();
        let sandbox = Self::file_system_sandbox_context_for_attempt(req, attempt);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = codex_apply_patch::apply_patch(
            &req.action.patch,
            &req.action.cwd,
            &mut stdout,
            &mut stderr,
            fs.as_ref(),
            sandbox.as_ref(),
        )
        .await;
        let stdout = String::from_utf8_lossy(&stdout).into_owned();
        let stderr = String::from_utf8_lossy(&stderr).into_owned();
        let exit_code = if result.is_ok() { 0 } else { 1 };
        let output = ExecToolCallOutput {
            exit_code,
            stdout: StreamOutput::new(stdout.clone()),
            stderr: StreamOutput::new(stderr.clone()),
            aggregated_output: StreamOutput::new(format!("{stdout}{stderr}")),
            duration: started_at.elapsed(),
            timed_out: false,
        };
        let result_is_err = result.is_err();
        let delta = match result {
            Ok(delta) => {
                self.committed_delta.append(delta);
                self.committed_delta.clone()
            }
            Err(failure) => failure.into_parts().1,
        };
        if result_is_err && is_likely_sandbox_denied(attempt.sandbox, &output) {
            return Err(ToolError::Codex(CodexErr::Sandbox(SandboxErr::Denied {
                output: Box::new(output),
                network_policy_decision: None,
            })));
        }
        Ok(ApplyPatchRuntimeOutput {
            exec_output: output,
            delta,
        })
    }
}

#[cfg(test)]
#[path = "apply_patch_tests.rs"]
mod tests;
