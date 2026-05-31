/*
Runtime: shell

Executes shell requests under the orchestrator: asks for approval when needed,
builds sandbox transform inputs, and runs them under the current SandboxAttempt.
*/
pub(crate) mod shell_command;

pub use shell_command::ShellCommandHandler;
pub(crate) use shell_command::ShellCommandHandlerOptions;

use codex_protocol::ThreadId;
use codex_protocol::models::ShellCommandToolCallParams;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::exec::ExecCapturePolicy;
use crate::exec::ExecParams;
use crate::exec_env::create_env;
use crate::exec_policy::ExecApprovalRequest;
use crate::function_tool::FunctionCallError;
use crate::session::turn_context::TurnContext;
use crate::shell::ShellType;
use crate::shell_startup_files::apply_shell_startup_files_env;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::events::ToolEmitter;
use crate::tools::events::ToolEventCtx;
use crate::tools::handlers::apply_granted_turn_permissions;
use crate::tools::handlers::apply_patch::intercept_apply_patch;
use crate::tools::handlers::implicit_granted_permissions;
use crate::tools::handlers::normalize_and_validate_additional_permissions;
use crate::tools::handlers::parse_arguments;
use crate::tools::handlers::parse_arguments_with_base_path;
use crate::tools::handlers::resolve_workdir_base_path;
use crate::tools::handlers::shell_spec::CommandToolOptions;
use crate::tools::handlers::shell_spec::create_shell_command_tool;
use crate::tools::hook_names::HookToolName;
use crate::tools::orchestrator::ToolOrchestrator;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::PostToolUsePayload;
use crate::tools::registry::PreToolUsePayload;
use crate::tools::registry::ToolExecutor;
use crate::tools::runtimes::shell::ShellRequest;
use crate::tools::runtimes::shell::ShellRuntime;
use crate::tools::runtimes::shell::ShellRuntimeBackend;
use crate::tools::sandboxing::ToolCtx;
use codex_features::Feature;
use codex_protocol::models::AdditionalPermissionProfile;
use codex_protocol::protocol::ExecCommandSource;
use codex_tools::ToolName;
use codex_tools::ToolSpec;

#[derive(Default)]
pub struct ShellHandler;

pub struct ContainerExecHandler;

type ShellToolCallParams = ShellCommandToolCallParams;

fn shell_payload_command(payload: &ToolPayload) -> Option<String> {
    let ToolPayload::Function { arguments } = payload else {
        return None;
    };

    parse_arguments::<ShellToolCallParams>(arguments)
        .ok()
        .map(|params| params.command)
}

fn shell_command_payload_command(payload: &ToolPayload) -> Option<String> {
    shell_payload_command(payload)
}

struct RunExecLikeArgs {
    tool_name: ToolName,
    exec_params: ExecParams,
    hook_command: String,
    shell_type: Option<ShellType>,
    additional_permissions: Option<AdditionalPermissionProfile>,
    prefix_rule: Option<Vec<String>>,
    session: Arc<crate::session::session::Session>,
    turn: Arc<TurnContext>,
    tracker: crate::tools::context::SharedTurnDiffTracker,
    call_id: String,
    freeform: bool,
    cancellation_token: CancellationToken,
    shell_runtime_backend: ShellRuntimeBackend,
}

impl ShellHandler {
    fn to_exec_params(
        params: &ShellToolCallParams,
        session: &crate::session::session::Session,
        turn_context: &TurnContext,
        thread_id: ThreadId,
    ) -> Result<ExecParams, FunctionCallError> {
        let shell = session.user_shell();
        let use_login_shell = match (params.login, turn_context.tools_config.allow_login_shell) {
            (Some(true), false) => {
                return Err(FunctionCallError::RespondToModel(
                    "login shell is disabled by config; omit `login` or set it to false."
                        .to_string(),
                ));
            }
            (Some(login), _) => login,
            (None, allow_login_shell) => allow_login_shell,
        };
        let command = shell.derive_exec_args(&params.command, use_login_shell);
        let assistant_shell_environment_policy = turn_context
            .assistant_shell_environment_policy()
            .unwrap_or_else(|err| {
                tracing::warn!(
                    error = %err,
                    "failed to resolve assistant shell environment policy; falling back to current shell policy"
                );
                turn_context.shell_environment_policy.clone()
            });
        let mut env = create_env(&assistant_shell_environment_policy, Some(thread_id));
        apply_shell_startup_files_env(&mut env, shell.shell_type.clone());

        Ok(ExecParams {
            command,
            cwd: turn_context.resolve_path(params.workdir.clone()),
            expiration: params.timeout_ms.into(),
            capture_policy: ExecCapturePolicy::ShellTool,
            env,
            network: turn_context.network.clone(),
            sandbox_permissions: params.sandbox_permissions.unwrap_or_default(),
            windows_sandbox_level: turn_context.windows_sandbox_level,
            windows_sandbox_private_desktop: turn_context
                .config
                .permissions
                .windows_sandbox_private_desktop,
            justification: params.justification.clone(),
            arg0: None,
        })
    }
}

pub(crate) async fn run_exec_like(
    args: RunExecLikeArgs,
) -> Result<FunctionToolOutput, FunctionCallError> {
    let RunExecLikeArgs {
        tool_name,
        exec_params,
        hook_command,
        shell_type,
        additional_permissions,
        prefix_rule,
        session,
        turn,
        tracker,
        call_id,
        freeform,
        cancellation_token,
        shell_runtime_backend,
    } = args;

    let mut exec_params = exec_params;
    let Some(environment) = turn.environments.primary() else {
        return Err(FunctionCallError::RespondToModel(
            "shell is unavailable in this session".to_string(),
        ));
    };
    let fs = environment.environment.get_filesystem();

    let dependency_env = session.dependency_env().await;
    if !dependency_env.is_empty() {
        exec_params.env.extend(dependency_env.clone());
    }
    apply_shell_startup_files_env(
        &mut exec_params.env,
        session.user_shell().shell_type.clone(),
    );

    let assistant_shell_environment_policy = turn
        .assistant_shell_environment_policy()
        .unwrap_or_else(|err| {
            tracing::warn!(
                error = %err,
                "failed to resolve assistant shell environment policy; falling back to current shell policy"
            );
            turn.shell_environment_policy.clone()
        });
    let mut explicit_env_overrides = assistant_shell_environment_policy.r#set.clone();
    for key in dependency_env.keys() {
        if let Some(value) = exec_params.env.get(key) {
            explicit_env_overrides.insert(key.clone(), value.clone());
        }
    }

    let exec_permission_approvals_enabled =
        session.features().enabled(Feature::ExecPermissionApprovals);
    let requested_additional_permissions = additional_permissions.clone();
    let effective_additional_permissions = apply_granted_turn_permissions(
        session.as_ref(),
        turn.cwd.as_path(),
        exec_params.sandbox_permissions,
        additional_permissions,
    )
    .await;
    let additional_permissions_allowed = exec_permission_approvals_enabled
        || (session.features().enabled(Feature::RequestPermissionsTool)
            && effective_additional_permissions.permissions_preapproved);
    let normalized_additional_permissions = implicit_granted_permissions(
        exec_params.sandbox_permissions,
        requested_additional_permissions.as_ref(),
        &effective_additional_permissions,
    )
    .map_or_else(
        || {
            normalize_and_validate_additional_permissions(
                additional_permissions_allowed,
                turn.approval_policy.value(),
                effective_additional_permissions.sandbox_permissions,
                effective_additional_permissions.additional_permissions,
                effective_additional_permissions.permissions_preapproved,
                &exec_params.cwd,
            )
        },
        |permissions| Ok(Some(permissions)),
    )
    .map_err(FunctionCallError::RespondToModel)?;

    if effective_additional_permissions
        .sandbox_permissions
        .requests_sandbox_override()
        && !effective_additional_permissions.permissions_preapproved
        && !matches!(
            turn.approval_policy.value(),
            codex_protocol::protocol::AskForApproval::OnRequest
        )
    {
        let approval_policy = turn.approval_policy.value();
        return Err(FunctionCallError::RespondToModel(format!(
            "approval policy is {approval_policy:?}; reject command — you should not ask for escalated permissions if the approval policy is {approval_policy:?}"
        )));
    }

    let turn_environment = environment.clone();
    if let Some(output) = intercept_apply_patch(
        &exec_params.command,
        &exec_params.cwd,
        fs.as_ref(),
        turn_environment,
        session.clone(),
        turn.clone(),
        Some(&tracker),
        &call_id,
        tool_name.name.as_str(),
    )
    .await?
    {
        return Ok(output);
    }

    let source = ExecCommandSource::Agent;
    let emitter = ToolEmitter::shell(exec_params.command.clone(), exec_params.cwd.clone(), source);
    let event_ctx = ToolEventCtx::new(
        session.as_ref(),
        turn.as_ref(),
        &call_id,
        /*turn_diff_tracker*/ None,
    );
    emitter.begin(event_ctx).await;

    let file_system_sandbox_policy = turn.file_system_sandbox_policy();
    let exec_approval_requirement = session
        .services
        .exec_policy
        .create_exec_approval_requirement_for_command(ExecApprovalRequest {
            command: &exec_params.command,
            approval_policy: turn.approval_policy.value(),
            permission_profile: turn.permission_profile(),
            file_system_sandbox_policy: &file_system_sandbox_policy,
            sandbox_cwd: turn.cwd.as_path(),
            sandbox_permissions: if effective_additional_permissions.permissions_preapproved {
                codex_protocol::models::SandboxPermissions::UseDefault
            } else {
                effective_additional_permissions.sandbox_permissions
            },
            prefix_rule,
        })
        .await;

    let req = ShellRequest {
        command: exec_params.command.clone(),
        shell_type,
        hook_command,
        cwd: exec_params.cwd.clone(),
        timeout_ms: exec_params.expiration.timeout_ms(),
        cancellation_token,
        env: exec_params.env.clone(),
        explicit_env_overrides,
        network: exec_params.network.clone(),
        sandbox_permissions: effective_additional_permissions.sandbox_permissions,
        additional_permissions: normalized_additional_permissions,
        #[cfg(unix)]
        additional_permissions_preapproved: effective_additional_permissions
            .permissions_preapproved,
        justification: exec_params.justification.clone(),
        exec_approval_requirement,
    };
    let mut orchestrator = ToolOrchestrator::new();
    let mut runtime = ShellRuntime::for_shell_command(shell_runtime_backend);
    let tool_ctx = ToolCtx {
        session: session.clone(),
        turn: turn.clone(),
        call_id: call_id.clone(),
        tool_name,
    };
    let out = orchestrator
        .run(
            &mut runtime,
            &req,
            &tool_ctx,
            &turn,
            turn.approval_policy.value(),
        )
        .await
        .map(|result| result.output);
    let event_ctx = ToolEventCtx::new(
        session.as_ref(),
        turn.as_ref(),
        &call_id,
        /*turn_diff_tracker*/ None,
    );
    let post_tool_use_response = out
        .as_ref()
        .ok()
        .map(|output| crate::tools::format_exec_output_str(output, turn.truncation_policy))
        .map(JsonValue::String);
    let content = emitter.finish(event_ctx, out, None).await?;
    Ok(FunctionToolOutput {
        body: vec![
            codex_protocol::models::FunctionCallOutputContentItem::InputText { text: content },
        ],
        success: Some(true),
        post_tool_use_response,
    })
}

#[async_trait::async_trait]
impl ToolExecutor<ToolInvocation> for ShellHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain("shell")
    }

    fn spec(&self) -> ToolSpec {
        create_shell_command_tool(CommandToolOptions {
            allow_login_shell: true,
            exec_permission_approvals_enabled: false,
        })
    }

    async fn handle(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
        let ToolInvocation {
            session,
            turn,
            cancellation_token,
            tracker,
            call_id,
            tool_name,
            payload,
            ..
        } = invocation;

        let ToolPayload::Function { arguments } = payload else {
            return Err(FunctionCallError::RespondToModel(format!(
                "unsupported payload for shell handler: {}",
                tool_name.name.as_str()
            )));
        };

        let cwd = resolve_workdir_base_path(&arguments, &turn.cwd)?;
        let params: ShellToolCallParams = parse_arguments_with_base_path(&arguments, &cwd)?;
        let prefix_rule = params.prefix_rule.clone();
        let exec_params = Self::to_exec_params(
            &params,
            session.as_ref(),
            turn.as_ref(),
            session.conversation_id,
        )?;
        Ok(boxed_tool_output(
            run_exec_like(RunExecLikeArgs {
                tool_name: tool_name.clone(),
                exec_params,
                hook_command: params.command,
                shell_type: Some(session.user_shell().shell_type.clone()),
                additional_permissions: params.additional_permissions.clone(),
                prefix_rule,
                session,
                turn,
                tracker,
                call_id,
                freeform: false,
                cancellation_token,
                shell_runtime_backend: ShellRuntimeBackend::ShellCommandClassic,
            })
            .await?,
        ))
    }
}

impl CoreToolRuntime for ShellHandler {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }

    fn pre_tool_use_payload(&self, invocation: &ToolInvocation) -> Option<PreToolUsePayload> {
        shell_payload_command(&invocation.payload).map(|command| PreToolUsePayload {
            tool_name: HookToolName::bash(),
            tool_input: serde_json::json!({ "command": command }),
        })
    }

    fn post_tool_use_payload(
        &self,
        invocation: &ToolInvocation,
        result: &dyn ToolOutput,
    ) -> Option<PostToolUsePayload> {
        let tool_response =
            result.post_tool_use_response(&invocation.call_id, &invocation.payload)?;
        let command = shell_payload_command(&invocation.payload)?;
        Some(PostToolUsePayload {
            tool_name: HookToolName::bash(),
            tool_use_id: invocation.call_id.clone(),
            tool_input: serde_json::json!({ "command": command }),
            tool_response,
        })
    }
}

#[cfg(test)]
#[path = "shell_tests.rs"]
mod tests;
