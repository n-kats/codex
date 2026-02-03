// Aggregates all former standalone integration tests as modules.
use codex_arg0::arg0_dispatch;
use ctor::ctor;
use tempfile::TempDir;

#[cfg(unix)]
fn fallback_arg0_dispatch() -> std::io::Result<TempDir> {
    use std::os::unix::fs::symlink;

    let exe = std::env::current_exe()?;
    let temp_dir = tempfile::Builder::new()
        .prefix("codex-arg0-test")
        .tempdir()?;
    let path = temp_dir.path();

    for filename in &["apply_patch", "applypatch", "codex-linux-sandbox"] {
        symlink(&exe, path.join(filename))?;
    }

    let path_element = path.display();
    let updated_path_env_var = match std::env::var("PATH") {
        Ok(existing_path) => format!("{path_element}:{existing_path}"),
        Err(_) => format!("{path_element}"),
    };

    unsafe {
        std::env::set_var("PATH", updated_path_env_var);
    }

    Ok(temp_dir)
}

// This code runs before any other tests are run.
// It allows the test binary to behave like codex and dispatch to apply_patch and codex-linux-sandbox
// based on the arg0.
// NOTE: this doesn't work on ARM
#[ctor]
pub static CODEX_ALIASES_TEMP_DIR: TempDir = unsafe {
    match arg0_dispatch() {
        Some(temp_dir) => temp_dir,
        None => {
            eprintln!(
                "WARNING: arg0_dispatch failed, falling back to a test-only PATH alias (CODEX_HOME={:?}, PATH={:?})",
                std::env::var("CODEX_HOME"),
                std::env::var("PATH")
            );
            #[cfg(unix)]
            {
                fallback_arg0_dispatch()
                    .unwrap_or_else(|err| panic!("failed to build test-only arg0 aliases: {err}"))
            }
            #[cfg(not(unix))]
            {
                panic!("arg0 dispatch failed on non-unix platform")
            }
        }
    }
};

#[cfg(not(target_os = "windows"))]
mod abort_tasks;
mod agent_websocket;
mod apply_patch_cli;
#[cfg(not(target_os = "windows"))]
mod approvals;
mod auth_refresh;
mod cli_stream;
mod client;
mod client_websockets;
mod codex_delegate;
mod collaboration_instructions;
mod compact;
mod compact_remote;
mod compact_resume_fork;
mod deprecation_notice;
mod exec;
mod exec_policy;
mod fork_thread;
mod grep_files;
mod hierarchical_agents;
mod image_rollout;
mod items;
mod json_result;
mod list_dir;
mod list_models;
mod live_cli;
mod model_info_overrides;
mod model_overrides;
mod model_tools;
mod models_cache_ttl;
mod models_etag_responses;
mod otel;
mod pending_input;
mod permissions_messages;
mod personality;
mod personality_migration;
mod prompt_caching;
mod quota_exceeded;
mod read_file;
mod remote_models;
mod request_compression;
mod request_user_input;
mod resume;
mod resume_warning;
mod review;
mod rmcp_client;
mod rollout_list_find;
mod seatbelt;
mod shell_command;
mod shell_serialization;
mod shell_snapshot;
mod skills;
mod sqlite_state;
mod stream_error_allows_next_turn;
mod stream_no_completed;
mod text_encoding_fix;
mod tool_harness;
mod tool_parallelism;
mod tools;
mod truncation;
mod turn_state;
mod undo;
mod unified_exec;
mod unstable_features_warning;
mod user_notification;
mod user_shell_cmd;
mod view_image;
mod web_search;
mod websocket_fallback;
