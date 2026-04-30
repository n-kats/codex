#![cfg(not(target_os = "windows"))]
#![allow(non_snake_case)]

use anyhow::Result;
use codex_features::Feature;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::SandboxPolicy;
use codex_protocol::user_input::UserInput;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::skip_if_no_network;
use core_test_support::skip_if_sandbox;
use core_test_support::skip_if_windows;
use core_test_support::test_codex::TestCodexHarness;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use core_test_support::wait_for_event_match;
use pretty_assertions::assert_eq;
use regex_lite::Regex;
use serde_json::json;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::time::timeout;

#[cfg(unix)]
fn resolve_worker_user(worker_user: &str) -> Option<(u32, u32)> {
    let Ok(c_user) = std::ffi::CString::new(worker_user) else {
        return None;
    };
    // SAFETY: libc call, CString provides a NUL-terminated pointer.
    let pw = unsafe { libc::getpwnam(c_user.as_ptr()) };
    if pw.is_null() {
        return None;
    }
    // SAFETY: `pw` is non-null and points to a valid passwd struct.
    let (uid, gid) = unsafe { ((*pw).pw_uid, (*pw).pw_gid) };
    Some((uid, gid))
}

#[cfg(unix)]
fn assistant_worker_user() -> Option<(&'static str, u32, u32)> {
    let worker_user = "assistant";
    let (worker_uid, worker_gid) = resolve_worker_user(worker_user)?;
    Some((worker_user, worker_uid, worker_gid))
}

#[cfg(unix)]
fn parse_exec_output_text(output_text: &str) -> (Option<i32>, &str) {
    static EXIT_CODE_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = EXIT_CODE_REGEX.get_or_init(|| {
        Regex::new(
            r"(?s)^(?:.*?)(?:Exit code:\s*(-?\d+)|Process exited with code (-?\d+)).*?Output:\n(.*)$",
        )
        .expect("valid exec output regex")
    });

    if let Some(captures) = regex.captures(output_text) {
        let exit_code = captures
            .get(1)
            .or_else(|| captures.get(2))
            .and_then(|value| value.as_str().parse::<i32>().ok());
        let output = captures.get(3).map(|value| value.as_str()).unwrap_or("");
        (exit_code, output)
    } else {
        (None, output_text)
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn custom__exec_worker_user__exec_command_tty_false_runs_as_worker_user() -> Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    skip_if_windows!(Ok(()));

    let Some((worker_user, worker_uid, worker_gid)) = assistant_worker_user() else {
        return Ok(());
    };

    let call_id = "uexec-worker-user-id-u";
    let args = json!({
        "cmd": "id -u",
        "yield_time_ms": 250,
        "tty": false,
    });

    let builder = test_codex().with_config(move |config| {
        config.use_experimental_unified_exec_tool = true;
        config
            .features
            .enable(Feature::UnifiedExec)
            .expect("test config should allow unified exec");
        config.custom.exec.worker_user = Some(worker_user.to_string());
        config.custom.exec.worker_uid = Some(worker_uid);
        config.custom.exec.worker_gid = Some(worker_gid);
    });
    let harness = TestCodexHarness::with_builder(builder).await?;

    let request_log = mount_sse_sequence(
        harness.server(),
        vec![
            sse(vec![
                ev_response_created("resp-1"),
                ev_function_call(call_id, "exec_command", &serde_json::to_string(&args)?),
                ev_completed("resp-1"),
            ]),
            sse(vec![
                ev_response_created("resp-2"),
                ev_assistant_message("msg-1", "ok"),
                ev_completed("resp-2"),
            ]),
        ],
    )
    .await;

    let test = harness.test();
    let codex = test.codex.clone();
    let cwd = test.cwd_path().to_path_buf();
    let session_model = test.session_configured.model.clone();

    codex
        .submit(Op::UserTurn {
            environments: None,
            items: vec![UserInput::Text {
                text: "run id -u via exec_command".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            cwd,
            approval_policy: AskForApproval::Never,
            approvals_reviewer: None,
            sandbox_policy: SandboxPolicy::DangerFullAccess,
            model: session_model,
            effort: None,
            summary: None,
            service_tier: None,
            collaboration_mode: None,
            personality: None,
            permission_profile: None,
        })
        .await?;

    let end = wait_for_event_match(&codex, |event| match event {
        EventMsg::ExecCommandEnd(end) if end.call_id == call_id => Some(end.clone()),
        _ => None,
    })
    .await;

    if end.exit_code != 0 {
        let stderr = end.stderr.to_lowercase();
        if stderr.contains("operation not permitted") || stderr.contains("permission denied") {
            return Ok(());
        }
    }

    assert_eq!(end.exit_code, 0, "stderr={}", end.stderr);
    assert_eq!(
        end.stdout.trim(),
        worker_uid.to_string(),
        "stderr={}",
        end.stderr
    );

    timeout(Duration::from_secs(60), async {
        wait_for_event(&codex, |event| matches!(event, EventMsg::TurnComplete(_))).await;
    })
    .await
    .expect("expected TurnComplete within timeout");

    timeout(Duration::from_secs(60), async {
        loop {
            if request_log.requests().len() >= 2 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("expected two /v1/responses requests within timeout");

    let output_text = request_log
        .function_call_output_text(call_id)
        .unwrap_or_default();
    let (exit_code, output_body) = parse_exec_output_text(&output_text);

    if exit_code.unwrap_or(-1) != 0 {
        let lower = output_text.to_lowercase();
        let is_perm = lower.contains("operation not permitted")
            || lower.contains("permission denied")
            || lower.contains("a password is required")
            || lower.contains("sudo:");
        if is_perm {
            return Ok(());
        }
    }

    assert_eq!(exit_code, Some(0), "output={output_text}");
    assert!(
        output_body
            .lines()
            .any(|line| line.trim() == worker_uid.to_string()),
        "expected worker uid in tool output; output={output_text}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn custom__exec_worker_user__shell_runs_as_worker_user() -> Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    skip_if_windows!(Ok(()));

    let Some((worker_user, worker_uid, worker_gid)) = assistant_worker_user() else {
        return Ok(());
    };

    let call_id = "shell-worker-user-id-u";
    let args = json!({
        "command": ["/usr/bin/id", "-u"],
        "timeout_ms": 1_000,
    });

    let builder = test_codex().with_config(move |config| {
        config
            .features
            .enable(Feature::ShellTool)
            .expect("test config should allow shell tool");
        config.custom.exec.worker_user = Some(worker_user.to_string());
        config.custom.exec.worker_uid = Some(worker_uid);
        config.custom.exec.worker_gid = Some(worker_gid);
    });
    let harness = TestCodexHarness::with_builder(builder).await?;

    let request_log = mount_sse_sequence(
        harness.server(),
        vec![
            sse(vec![
                ev_response_created("resp-1"),
                ev_function_call(call_id, "shell", &serde_json::to_string(&args)?),
                ev_completed("resp-1"),
            ]),
            sse(vec![
                ev_response_created("resp-2"),
                ev_assistant_message("msg-1", "ok"),
                ev_completed("resp-2"),
            ]),
        ],
    )
    .await;

    let test = harness.test();
    let codex = test.codex.clone();
    let cwd = test.cwd_path().to_path_buf();
    let session_model = test.session_configured.model.clone();

    codex
        .submit(Op::UserTurn {
            environments: None,
            items: vec![UserInput::Text {
                text: "run id -u via shell".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            cwd,
            approval_policy: AskForApproval::Never,
            approvals_reviewer: None,
            sandbox_policy: SandboxPolicy::DangerFullAccess,
            model: session_model,
            effort: None,
            summary: None,
            service_tier: None,
            collaboration_mode: None,
            personality: None,
            permission_profile: None,
        })
        .await?;

    let end = wait_for_event_match(&codex, |event| match event {
        EventMsg::ExecCommandEnd(end) if end.call_id == call_id => Some(end.clone()),
        _ => None,
    })
    .await;

    if end.exit_code != 0 {
        let stderr = end.stderr.to_lowercase();
        if stderr.contains("operation not permitted") || stderr.contains("permission denied") {
            return Ok(());
        }
    }

    assert_eq!(end.exit_code, 0, "stderr={}", end.stderr);
    assert_eq!(
        end.stdout.trim(),
        worker_uid.to_string(),
        "stderr={}",
        end.stderr
    );

    timeout(Duration::from_secs(60), async {
        wait_for_event(&codex, |event| matches!(event, EventMsg::TurnComplete(_))).await;
    })
    .await
    .expect("expected TurnComplete within timeout");

    timeout(Duration::from_secs(60), async {
        loop {
            if request_log.requests().len() >= 2 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("expected two /v1/responses requests within timeout");

    Ok(())
}
