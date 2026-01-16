use rand::Rng;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tokio::sync::Notify;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::exec_env::create_env;
use crate::protocol::ExecCommandSource;
use crate::sandboxing::ExecEnv;
use crate::sandboxing::SandboxPermissions;
use crate::shell::ShellType;
use crate::spawn::RunAsUser;
use crate::tools::events::ToolEmitter;
use crate::tools::events::ToolEventCtx;
use crate::tools::events::ToolEventStage;
use crate::tools::orchestrator::ToolOrchestrator;
use crate::tools::runtimes::unified_exec::UnifiedExecRequest as UnifiedExecToolRequest;
use crate::tools::runtimes::unified_exec::UnifiedExecRuntime;
use crate::tools::sandboxing::ToolCtx;
use crate::truncate::TruncationPolicy;
use crate::truncate::approx_token_count;
use crate::truncate::formatted_truncate_text;
use crate::unified_exec::ExecCommandRequest;
use crate::unified_exec::MAX_UNIFIED_EXEC_PROCESSES;
use crate::unified_exec::MAX_YIELD_TIME_MS;
use crate::unified_exec::MIN_EMPTY_YIELD_TIME_MS;
use crate::unified_exec::ProcessEntry;
use crate::unified_exec::ProcessStore;
use crate::unified_exec::UnifiedExecContext;
use crate::unified_exec::UnifiedExecError;
use crate::unified_exec::UnifiedExecProcessManager;
use crate::unified_exec::UnifiedExecResponse;
use crate::unified_exec::WARNING_UNIFIED_EXEC_PROCESSES;
use crate::unified_exec::WriteStdinRequest;
use crate::unified_exec::async_watcher::TRAILING_OUTPUT_GRACE;
use crate::unified_exec::async_watcher::emit_exec_end_for_unified_exec;
use crate::unified_exec::async_watcher::spawn_exit_watcher;
use crate::unified_exec::async_watcher::start_streaming_output;
use crate::unified_exec::clamp_yield_time;
use crate::unified_exec::generate_chunk_id;
use crate::unified_exec::head_tail_buffer::HeadTailBuffer;
use crate::unified_exec::process::OutputBuffer;
use crate::unified_exec::process::OutputHandles;
use crate::unified_exec::process::UnifiedExecProcess;
use crate::unified_exec::resolve_max_tokens;

const UNIFIED_EXEC_ENV: [(&str, &str); 10] = [
    ("NO_COLOR", "1"),
    ("TERM", "dumb"),
    ("LANG", "C.UTF-8"),
    ("LC_CTYPE", "C.UTF-8"),
    ("LC_ALL", "C.UTF-8"),
    ("COLORTERM", ""),
    ("PAGER", "cat"),
    ("GIT_PAGER", "cat"),
    ("GH_PAGER", "cat"),
    ("CODEX_CI", "1"),
];

fn apply_unified_exec_env(mut env: HashMap<String, String>) -> HashMap<String, String> {
    for (key, value) in UNIFIED_EXEC_ENV {
        env.insert(key.to_string(), value.to_string());
    }
    env
}

struct PreparedPtyCommand {
    program: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    arg0: Option<String>,
}

#[cfg(unix)]
fn unix_preferred_executable_path(candidates: &[&str], fallback: &str) -> String {
    candidates
        .iter()
        .find_map(|path| {
            std::path::Path::new(path)
                .exists()
                .then_some((*path).to_string())
        })
        .unwrap_or_else(|| fallback.to_string())
}

#[cfg(unix)]
async fn sudo_preflight_check(run_as: &RunAsUser) -> Result<(), String> {
    use tokio::process::Command;

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
        .env_clear()
        .output()
        .await
        .map_err(|err| format!("failed to run `{sudo_program}`: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        let code = output.status.code();
        return Err(format!(
            "`{sudo_program} -n -u '{uid}' -g '{gid}' -- {env_program} -i {id_program} -u` failed (exit={code:?}) stderr={stderr:?}",
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout = stdout.trim();
    if stdout != format!("{}", run_as.uid) {
        return Err(format!(
            "`{sudo_program} ... id -u` returned {stdout:?} (expected {})",
            run_as.uid
        ));
    }

    Ok(())
}

#[cfg(unix)]
fn prepare_pty_command(exec_env: &ExecEnv) -> Result<PreparedPtyCommand, UnifiedExecError> {
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
fn prepare_pty_command(exec_env: &ExecEnv) -> Result<PreparedPtyCommand, UnifiedExecError> {
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

struct PreparedProcessHandles {
    writer_tx: mpsc::Sender<Vec<u8>>,
    output_buffer: OutputBuffer,
    output_notify: Arc<Notify>,
    cancellation_token: CancellationToken,
    command: Vec<String>,
    process_id: String,
    tty: bool,
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

    pub(crate) async fn allocate_process_id(&self) -> String {
        loop {
            let mut store = self.process_store.lock().await;

            let process_id = if !cfg!(test) && !cfg!(feature = "deterministic_process_ids") {
                // production mode → random
                rand::rng().random_range(1_000..100_000).to_string()
            } else {
                // test or deterministic mode
                let next = store
                    .reserved_process_ids
                    .iter()
                    .filter_map(|s| s.parse::<i32>().ok())
                    .max()
                    .map(|m| std::cmp::max(m, 999) + 1)
                    .unwrap_or(1000);

                next.to_string()
            };

            if store.reserved_process_ids.contains(&process_id) {
                continue;
            }

            store.reserved_process_ids.insert(process_id.clone());
            return process_id;
        }
    }

    pub(crate) async fn release_process_id(&self, process_id: &str) {
        let mut store = self.process_store.lock().await;
        store.remove(process_id);
    }

    pub(crate) async fn exec_command(
        &self,
        request: ExecCommandRequest,
        context: &UnifiedExecContext,
    ) -> Result<UnifiedExecResponse, UnifiedExecError> {
        let cwd = request
            .workdir
            .clone()
            .unwrap_or_else(|| context.turn.cwd.clone());

        let process = self
            .open_session_with_sandbox(
                &request.command,
                cwd.clone(),
                request.sandbox_permissions,
                request.justification,
                request.tty,
                context,
            )
            .await;

        let process = match process {
            Ok(process) => Arc::new(process),
            Err(err) => {
                self.release_process_id(&request.process_id).await;
                return Err(err);
            }
        };

        let transcript = Arc::new(tokio::sync::Mutex::new(HeadTailBuffer::default()));
        let event_ctx = ToolEventCtx::new(
            context.session.as_ref(),
            context.turn.as_ref(),
            &context.call_id,
            None,
        );
        let emitter = ToolEmitter::unified_exec(
            &request.command,
            cwd.clone(),
            ExecCommandSource::UnifiedExecStartup,
            Some(request.process_id.clone()),
        );
        emitter.emit(event_ctx, ToolEventStage::Begin).await;

        start_streaming_output(&process, context, Arc::clone(&transcript));

        let max_tokens = resolve_max_tokens(request.max_output_tokens);
        let yield_time_ms = clamp_yield_time(request.yield_time_ms);

        let start = Instant::now();
        // For the initial exec_command call, we both stream output to events
        // (via start_streaming_output above) and collect a snapshot here for
        // the tool response body.
        let OutputHandles {
            output_buffer,
            output_notify,
            cancellation_token,
        } = process.output_handles();
        let deadline = start + Duration::from_millis(yield_time_ms);
        let collected = Self::collect_output_until_deadline(
            &output_buffer,
            &output_notify,
            &cancellation_token,
            deadline,
        )
        .await;
        let wall_time = Instant::now().saturating_duration_since(start);

        let text = String::from_utf8_lossy(&collected).to_string();
        let output = formatted_truncate_text(&text, TruncationPolicy::Tokens(max_tokens));
        let exit_code = process.exit_code();
        let has_exited = process.has_exited()
            || exit_code.is_some()
            || process.cancellation_token().is_cancelled();
        let chunk_id = generate_chunk_id();
        let process_id = request.process_id.clone();
        if has_exited {
            // Short‑lived command: emit ExecCommandEnd immediately using the
            // same helper as the background watcher, so all end events share
            // one implementation.
            let exit = exit_code.unwrap_or(-1);
            emit_exec_end_for_unified_exec(
                Arc::clone(&context.session),
                Arc::clone(&context.turn),
                context.call_id.clone(),
                request.command.clone(),
                cwd,
                Some(process_id),
                Arc::clone(&transcript),
                output.clone(),
                exit,
                wall_time,
            )
            .await;

            self.release_process_id(&request.process_id).await;
            process.check_for_sandbox_denial_with_text(&text).await?;
        } else {
            // Long‑lived command: persist the process so write_stdin can reuse
            // it, and register a background watcher that will emit
            // ExecCommandEnd when the PTY eventually exits (even if no further
            // tool calls are made).
            self.store_process(
                Arc::clone(&process),
                context,
                &request.command,
                cwd.clone(),
                start,
                process_id,
                request.tty,
                Arc::clone(&transcript),
            )
            .await;
        };

        let original_token_count = approx_token_count(&text);
        let response = UnifiedExecResponse {
            event_call_id: context.call_id.clone(),
            chunk_id,
            wall_time,
            output,
            raw_output: collected,
            process_id: if has_exited {
                None
            } else {
                Some(request.process_id.clone())
            },
            exit_code,
            original_token_count: Some(original_token_count),
            session_command: Some(request.command.clone()),
        };

        Ok(response)
    }

    pub(crate) async fn write_stdin(
        &self,
        request: WriteStdinRequest<'_>,
    ) -> Result<UnifiedExecResponse, UnifiedExecError> {
        let process_id = request.process_id.to_string();

        let PreparedProcessHandles {
            writer_tx,
            output_buffer,
            output_notify,
            cancellation_token,
            command: session_command,
            process_id,
            tty,
            ..
        } = self.prepare_process_handles(process_id.as_str()).await?;

        if !request.input.is_empty() {
            if !tty {
                return Err(UnifiedExecError::StdinClosed);
            }
            Self::send_input(&writer_tx, request.input.as_bytes()).await?;
            // Give the remote process a brief window to react so that we are
            // more likely to capture its output in the poll below.
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let max_tokens = resolve_max_tokens(request.max_output_tokens);
        let yield_time_ms = {
            let time_ms = clamp_yield_time(request.yield_time_ms);
            if request.input.is_empty() {
                time_ms.clamp(MIN_EMPTY_YIELD_TIME_MS, MAX_YIELD_TIME_MS)
            } else {
                time_ms
            }
        };
        let start = Instant::now();
        let deadline = start + Duration::from_millis(yield_time_ms);
        let collected = Self::collect_output_until_deadline(
            &output_buffer,
            &output_notify,
            &cancellation_token,
            deadline,
        )
        .await;
        let wall_time = Instant::now().saturating_duration_since(start);

        let text = String::from_utf8_lossy(&collected).to_string();
        let output = formatted_truncate_text(&text, TruncationPolicy::Tokens(max_tokens));
        let original_token_count = approx_token_count(&text);
        let chunk_id = generate_chunk_id();

        // After polling, refresh_process_state tells us whether the PTY is
        // still alive or has exited and been removed from the store; we thread
        // that through so the handler can tag TerminalInteraction with an
        // appropriate process_id and exit_code.
        let status = self.refresh_process_state(process_id.as_str()).await;
        let (process_id, exit_code, event_call_id) = match status {
            ProcessStatus::Alive {
                exit_code,
                call_id,
                process_id,
            } => (Some(process_id), exit_code, call_id),
            ProcessStatus::Exited { exit_code, entry } => {
                let call_id = entry.call_id.clone();
                let exit = exit_code.unwrap_or(-1);
                let output_drained = entry.process.output_drained_notify();
                let _ = tokio::time::timeout(TRAILING_OUTPUT_GRACE * 2, output_drained.notified())
                    .await;

                if !entry.end_emitted.swap(true, Ordering::SeqCst) {
                    let duration = Instant::now().saturating_duration_since(entry.started_at);
                    emit_exec_end_for_unified_exec(
                        Arc::clone(&entry.session_ref),
                        Arc::clone(&entry.turn_ref),
                        entry.call_id.clone(),
                        entry.command.clone(),
                        entry.cwd.clone(),
                        Some(entry.process_id.clone()),
                        Arc::clone(&entry.transcript),
                        output.clone(),
                        exit,
                        duration,
                    )
                    .await;
                }

                (None, exit_code, call_id)
            }
            ProcessStatus::Unknown => {
                return Err(UnifiedExecError::UnknownProcessId {
                    process_id: request.process_id.to_string(),
                });
            }
        };

        let response = UnifiedExecResponse {
            event_call_id,
            chunk_id,
            wall_time,
            output,
            raw_output: collected,
            process_id,
            exit_code,
            original_token_count: Some(original_token_count),
            session_command: Some(session_command.clone()),
        };

        Ok(response)
    }

    async fn refresh_process_state(&self, process_id: &str) -> ProcessStatus {
        let mut store = self.process_store.lock().await;
        let Some(entry) = store.processes.get(process_id) else {
            return ProcessStatus::Unknown;
        };

        let exit_code = entry.process.exit_code();
        let process_id = entry.process_id.clone();

        let has_exited = entry.process.has_exited()
            || exit_code.is_some()
            || entry.process.cancellation_token().is_cancelled();
        if has_exited {
            // If we only observe an exit code (but `has_exited()` hasn't flipped yet),
            // proactively signal the background watchers so we don't miss the
            // ExecCommandEnd event.
            if exit_code.is_some() {
                entry.process.cancellation_token().cancel();
            }
            let Some(entry) = store.remove(&process_id) else {
                return ProcessStatus::Unknown;
            };
            ProcessStatus::Exited {
                exit_code,
                entry: Box::new(entry),
            }
        } else {
            ProcessStatus::Alive {
                exit_code,
                call_id: entry.call_id.clone(),
                process_id,
            }
        }
    }

    async fn prepare_process_handles(
        &self,
        process_id: &str,
    ) -> Result<PreparedProcessHandles, UnifiedExecError> {
        let mut store = self.process_store.lock().await;
        let entry =
            store
                .processes
                .get_mut(process_id)
                .ok_or(UnifiedExecError::UnknownProcessId {
                    process_id: process_id.to_string(),
                })?;
        entry.last_used = Instant::now();
        let OutputHandles {
            output_buffer,
            output_notify,
            cancellation_token,
        } = entry.process.output_handles();

        Ok(PreparedProcessHandles {
            writer_tx: entry.process.writer_sender(),
            output_buffer,
            output_notify,
            cancellation_token,
            command: entry.command.clone(),
            process_id: entry.process_id.clone(),
            tty: entry.tty,
        })
    }

    async fn send_input(
        writer_tx: &mpsc::Sender<Vec<u8>>,
        data: &[u8],
    ) -> Result<(), UnifiedExecError> {
        writer_tx
            .send(data.to_vec())
            .await
            .map_err(|_| UnifiedExecError::WriteToStdin)
    }

    #[allow(clippy::too_many_arguments)]
    async fn store_process(
        &self,
        process: Arc<UnifiedExecProcess>,
        context: &UnifiedExecContext,
        command: &[String],
        cwd: PathBuf,
        started_at: Instant,
        process_id: String,
        tty: bool,
        transcript: Arc<tokio::sync::Mutex<HeadTailBuffer>>,
    ) {
        let end_emitted = Arc::new(AtomicBool::new(false));
        let entry = ProcessEntry {
            process: Arc::clone(&process),
            session_ref: Arc::clone(&context.session),
            turn_ref: Arc::clone(&context.turn),
            call_id: context.call_id.clone(),
            process_id: process_id.clone(),
            command: command.to_vec(),
            tty,
            cwd: cwd.clone(),
            started_at,
            transcript: Arc::clone(&transcript),
            end_emitted: Arc::clone(&end_emitted),
            last_used: started_at,
        };
        let number_processes = {
            let mut store = self.process_store.lock().await;
            Self::prune_processes_if_needed(&mut store);
            store.processes.insert(process_id.clone(), entry);
            store.processes.len()
        };

        if number_processes >= WARNING_UNIFIED_EXEC_PROCESSES {
            context
                .session
                .record_model_warning(
                    format!("The maximum number of unified exec processes you can keep open is {WARNING_UNIFIED_EXEC_PROCESSES} and you currently have {number_processes} processes open. Reuse older processes or close them to prevent automatic pruning of old processes"),
                    &context.turn
                )
                .await;
        };

        spawn_exit_watcher(
            Arc::clone(&process),
            Arc::clone(&context.session),
            Arc::clone(&context.turn),
            context.call_id.clone(),
            command.to_vec(),
            cwd,
            process_id,
            transcript,
            end_emitted,
            started_at,
        );
    }

    pub(crate) async fn open_session_with_exec_env(
        &self,
        env: &ExecEnv,
        tty: bool,
    ) -> Result<UnifiedExecProcess, UnifiedExecError> {
        if let Some(run_as) = env.run_as.clone() {
            self.preflight_exec_command_sudo_worker_user(run_as).await?;
        }
        let prepared = prepare_pty_command(env)?;

        let spawn_result = if tty {
            codex_utils_pty::pty::spawn_process(
                prepared.program.as_str(),
                &prepared.args,
                env.cwd.as_path(),
                &prepared.env,
                &prepared.arg0,
            )
            .await
        } else {
            codex_utils_pty::pipe::spawn_process_no_stdin(
                prepared.program.as_str(),
                &prepared.args,
                env.cwd.as_path(),
                &prepared.env,
                &prepared.arg0,
            )
            .await
        };
        let spawned =
            spawn_result.map_err(|err| UnifiedExecError::create_process(err.to_string()))?;
        UnifiedExecProcess::from_spawned(spawned, env.sandbox).await
    }

    pub(super) async fn open_session_with_sandbox(
        &self,
        command: &[String],
        cwd: PathBuf,
        sandbox_permissions: SandboxPermissions,
        justification: Option<String>,
        tty: bool,
        context: &UnifiedExecContext,
    ) -> Result<UnifiedExecProcess, UnifiedExecError> {
        let shell_type = command
            .first()
            .map(|program| crate::shell::detect_shell_type(&PathBuf::from(program)))
            .flatten()
            .unwrap_or(ShellType::Sh);

        let mut env = apply_unified_exec_env(create_env(&context.turn.shell_environment_policy));
        crate::shell_startup_files::apply_shell_startup_files_env(&mut env, shell_type);
        let features = context.session.features();
        let mut orchestrator = ToolOrchestrator::new();
        let mut runtime = UnifiedExecRuntime::new(self);
        let exec_approval_requirement = context
            .session
            .services
            .exec_policy
            .create_exec_approval_requirement_for_command(
                &features,
                command,
                context.turn.approval_policy,
                &context.turn.sandbox_policy,
                sandbox_permissions,
            )
            .await;
        let req = UnifiedExecToolRequest::new(
            command.to_vec(),
            cwd,
            env,
            tty,
            sandbox_permissions,
            justification,
            exec_approval_requirement,
        );
        let tool_ctx = ToolCtx {
            session: context.session.as_ref(),
            turn: context.turn.as_ref(),
            call_id: context.call_id.clone(),
            tool_name: "exec_command".to_string(),
        };
        orchestrator
            .run(
                &mut runtime,
                &req,
                &tool_ctx,
                context.turn.as_ref(),
                context.turn.approval_policy,
            )
            .await
            .map_err(|e| UnifiedExecError::create_process(format!("{e:?}")))
    }

    pub(super) async fn collect_output_until_deadline(
        output_buffer: &OutputBuffer,
        output_notify: &Arc<Notify>,
        cancellation_token: &CancellationToken,
        deadline: Instant,
    ) -> Vec<u8> {
        const POST_EXIT_OUTPUT_GRACE: Duration = Duration::from_millis(50);

        let mut collected: Vec<u8> = Vec::with_capacity(4096);
        let mut exit_signal_received = cancellation_token.is_cancelled();
        loop {
            let drained_chunks: Vec<Vec<u8>>;
            let mut wait_for_output = None;
            {
                let mut guard = output_buffer.lock().await;
                drained_chunks = guard.drain_chunks();
                if drained_chunks.is_empty() {
                    wait_for_output = Some(output_notify.notified());
                }
            }

            if drained_chunks.is_empty() {
                exit_signal_received |= cancellation_token.is_cancelled();
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining == Duration::ZERO {
                    break;
                }

                let notified = wait_for_output.unwrap_or_else(|| output_notify.notified());
                if exit_signal_received {
                    let grace = remaining.min(POST_EXIT_OUTPUT_GRACE);
                    if tokio::time::timeout(grace, notified).await.is_err() {
                        break;
                    }
                    continue;
                }

                tokio::pin!(notified);
                let exit_notified = cancellation_token.cancelled();
                tokio::pin!(exit_notified);
                tokio::select! {
                    _ = &mut notified => {}
                    _ = &mut exit_notified => exit_signal_received = true,
                    _ = tokio::time::sleep(remaining) => break,
                }
                continue;
            }

            for chunk in drained_chunks {
                collected.extend_from_slice(&chunk);
            }

            exit_signal_received |= cancellation_token.is_cancelled();
            if Instant::now() >= deadline {
                break;
            }
        }

        collected
    }

    fn prune_processes_if_needed(store: &mut ProcessStore) -> bool {
        if store.processes.len() < MAX_UNIFIED_EXEC_PROCESSES {
            return false;
        }

        let meta: Vec<(String, Instant, bool)> = store
            .processes
            .iter()
            .map(|(id, entry)| (id.clone(), entry.last_used, entry.process.has_exited()))
            .collect();

        if let Some(process_id) = Self::process_id_to_prune_from_meta(&meta) {
            if let Some(entry) = store.remove(&process_id) {
                entry.process.terminate();
            }
            return true;
        }

        false
    }

    // Centralized pruning policy so we can easily swap strategies later.
    fn process_id_to_prune_from_meta(meta: &[(String, Instant, bool)]) -> Option<String> {
        if meta.is_empty() {
            return None;
        }

        let mut by_recency = meta.to_vec();
        by_recency.sort_by_key(|(_, last_used, _)| Reverse(*last_used));
        let protected: HashSet<String> = by_recency
            .iter()
            .take(8)
            .map(|(process_id, _, _)| process_id.clone())
            .collect();

        let mut lru = meta.to_vec();
        lru.sort_by_key(|(_, last_used, _)| *last_used);

        if let Some((process_id, _, _)) = lru
            .iter()
            .find(|(process_id, _, exited)| !protected.contains(process_id) && *exited)
        {
            return Some(process_id.clone());
        }

        lru.into_iter()
            .find(|(process_id, _, _)| !protected.contains(process_id))
            .map(|(process_id, _, _)| process_id)
    }

    pub(crate) async fn terminate_all_processes(&self) {
        let entries: Vec<ProcessEntry> = {
            let mut processes = self.process_store.lock().await;
            let entries: Vec<ProcessEntry> = processes
                .processes
                .drain()
                .map(|(_, entry)| entry)
                .collect();
            processes.reserved_process_ids.clear();
            entries
        };

        for entry in entries {
            entry.process.terminate();
        }
    }
}

enum ProcessStatus {
    Alive {
        exit_code: Option<i32>,
        call_id: String,
        process_id: String,
    },
    Exited {
        exit_code: Option<i32>,
        entry: Box<ProcessEntry>,
    },
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::ExecExpiration;
    use crate::exec::SandboxType;
    use pretty_assertions::assert_eq;
    use tokio::time::Duration;
    use tokio::time::Instant;

    #[test]
    fn unified_exec_env_injects_defaults() {
        let env = apply_unified_exec_env(HashMap::new());
        let expected = HashMap::from([
            ("NO_COLOR".to_string(), "1".to_string()),
            ("TERM".to_string(), "dumb".to_string()),
            ("LANG".to_string(), "C.UTF-8".to_string()),
            ("LC_CTYPE".to_string(), "C.UTF-8".to_string()),
            ("LC_ALL".to_string(), "C.UTF-8".to_string()),
            ("COLORTERM".to_string(), String::new()),
            ("PAGER".to_string(), "cat".to_string()),
            ("GIT_PAGER".to_string(), "cat".to_string()),
            ("GH_PAGER".to_string(), "cat".to_string()),
            ("CODEX_CI".to_string(), "1".to_string()),
        ]);

        assert_eq!(env, expected);
    }

    #[test]
    fn unified_exec_env_overrides_existing_values() {
        let mut base = HashMap::new();
        base.insert("NO_COLOR".to_string(), "0".to_string());
        base.insert("PATH".to_string(), "/usr/bin".to_string());

        let env = apply_unified_exec_env(base);

        assert_eq!(env.get("NO_COLOR"), Some(&"1".to_string()));
        assert_eq!(env.get("PATH"), Some(&"/usr/bin".to_string()));
    }

    #[test]
    fn pruning_prefers_exited_processes_outside_recently_used() {
        let now = Instant::now();
        let id = |n: i32| n.to_string();
        let meta = vec![
            (id(1), now - Duration::from_secs(40), false),
            (id(2), now - Duration::from_secs(30), true),
            (id(3), now - Duration::from_secs(20), false),
            (id(4), now - Duration::from_secs(19), false),
            (id(5), now - Duration::from_secs(18), false),
            (id(6), now - Duration::from_secs(17), false),
            (id(7), now - Duration::from_secs(16), false),
            (id(8), now - Duration::from_secs(15), false),
            (id(9), now - Duration::from_secs(14), false),
            (id(10), now - Duration::from_secs(13), false),
        ];

        let candidate = UnifiedExecProcessManager::process_id_to_prune_from_meta(&meta);

        assert_eq!(candidate, Some(id(2)));
    }

    #[test]
    fn pruning_falls_back_to_lru_when_no_exited() {
        let now = Instant::now();
        let id = |n: i32| n.to_string();
        let meta = vec![
            (id(1), now - Duration::from_secs(40), false),
            (id(2), now - Duration::from_secs(30), false),
            (id(3), now - Duration::from_secs(20), false),
            (id(4), now - Duration::from_secs(19), false),
            (id(5), now - Duration::from_secs(18), false),
            (id(6), now - Duration::from_secs(17), false),
            (id(7), now - Duration::from_secs(16), false),
            (id(8), now - Duration::from_secs(15), false),
            (id(9), now - Duration::from_secs(14), false),
            (id(10), now - Duration::from_secs(13), false),
        ];

        let candidate = UnifiedExecProcessManager::process_id_to_prune_from_meta(&meta);

        assert_eq!(candidate, Some(id(1)));
    }

    #[test]
    fn pruning_protects_recent_processes_even_if_exited() {
        let now = Instant::now();
        let id = |n: i32| n.to_string();
        let meta = vec![
            (id(1), now - Duration::from_secs(40), false),
            (id(2), now - Duration::from_secs(30), false),
            (id(3), now - Duration::from_secs(20), true),
            (id(4), now - Duration::from_secs(19), false),
            (id(5), now - Duration::from_secs(18), false),
            (id(6), now - Duration::from_secs(17), false),
            (id(7), now - Duration::from_secs(16), false),
            (id(8), now - Duration::from_secs(15), false),
            (id(9), now - Duration::from_secs(14), false),
            (id(10), now - Duration::from_secs(13), true),
        ];

        let candidate = UnifiedExecProcessManager::process_id_to_prune_from_meta(&meta);

        // (10) is exited but among the last 8; we should drop the LRU outside that set.
        assert_eq!(candidate, Some(id(1)));
    }

    #[cfg(unix)]
    #[test]
    fn prepare_pty_command_keeps_original_when_no_run_as() {
        let exec_env = ExecEnv {
            command: vec!["bash".to_string(), "-lc".to_string(), "echo hi".to_string()],
            cwd: PathBuf::from("/tmp"),
            env: HashMap::from([("B".to_string(), "2".to_string())]),
            expiration: ExecExpiration::DefaultTimeout,
            sandbox: SandboxType::None,
            run_as: None,
            sandbox_permissions: SandboxPermissions::UseDefault,
            justification: None,
            arg0: Some("codex-linux-sandbox".to_string()),
        };

        let prepared = prepare_pty_command(&exec_env).expect("prepare_pty_command");

        assert_eq!(prepared.program, "bash".to_string());
        assert_eq!(
            prepared.args,
            vec!["-lc".to_string(), "echo hi".to_string()]
        );
        assert_eq!(prepared.env, exec_env.env);
        assert_eq!(prepared.arg0, exec_env.arg0);
    }

    #[cfg(unix)]
    #[test]
    fn prepare_pty_command_wraps_with_sudo_and_env_i_when_run_as_configured() {
        let exec_env = ExecEnv {
            command: vec!["bash".to_string(), "-lc".to_string(), "echo hi".to_string()],
            cwd: PathBuf::from("/tmp"),
            env: HashMap::from([
                ("B".to_string(), "2".to_string()),
                ("A".to_string(), "1".to_string()),
            ]),
            expiration: ExecExpiration::DefaultTimeout,
            sandbox: SandboxType::None,
            run_as: Some(crate::spawn::RunAsUser {
                uid: 1001,
                gid: 1002,
                supplementary_gids: None,
            }),
            sandbox_permissions: SandboxPermissions::UseDefault,
            justification: None,
            arg0: Some("codex-linux-sandbox".to_string()),
        };

        let prepared = prepare_pty_command(&exec_env).expect("prepare_pty_command");

        assert!(prepared.program.ends_with("sudo"));
        assert_eq!(prepared.env, HashMap::new());
        assert_eq!(prepared.arg0, None);

        let args = &prepared.args;
        assert_eq!(args.get(0), Some(&"-n".to_string()));
        assert_eq!(args.get(1), Some(&"-u".to_string()));
        assert_eq!(args.get(2), Some(&"#1001".to_string()));
        assert_eq!(args.get(3), Some(&"-g".to_string()));
        assert_eq!(args.get(4), Some(&"#1002".to_string()));
        assert_eq!(args.get(5), Some(&"--".to_string()));
        assert!(args.get(6).expect("env program").ends_with("env"));
        assert_eq!(args.get(7), Some(&"-i".to_string()));
        assert_eq!(args.get(8), Some(&"A=1".to_string()));
        assert_eq!(args.get(9), Some(&"B=2".to_string()));
        assert_eq!(args.get(10), Some(&"bash".to_string()));
        assert_eq!(args.get(11), Some(&"-lc".to_string()));
        assert_eq!(args.get(12), Some(&"echo hi".to_string()));
    }
}
