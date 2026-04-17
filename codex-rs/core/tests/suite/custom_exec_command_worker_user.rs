#![allow(non_snake_case)]
use anyhow::Result;
use codex_core::spawn::RunAsUser;
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
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn custom__exec_worker_user__exec_command_tty_false_runs_as_worker_user() -> Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    skip_if_windows!(Ok(()));

    let worker_user = "assistant";
    let (worker_uid, worker_gid) = {
        let Ok(c_user) = std::ffi::CString::new(worker_user) else {
            return Ok(());
        };
        // SAFETY: libc call, CString provides NUL-terminated pointer.
        let pw = unsafe { libc::getpwnam(c_user.as_ptr()) };
        if pw.is_null() {
            return Ok(());
        }
        // SAFETY: pw is non-null and points to a passwd struct.
        unsafe { ((*pw).pw_uid, (*pw).pw_gid) }
    };

    // SAFETY: libc call.
    let invoker_uid = unsafe { libc::geteuid() };
    if worker_uid == invoker_uid {
        return Ok(());
    }

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
        config.exec_run_as = Some(RunAsUser {
            uid: worker_uid as u32,
            gid: worker_gid as u32,
            supplementary_gids: None,
        });
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
        (worker_uid as u32).to_string(),
        "stderr={}",
        end.stderr
    );

    // The mocked model interaction is a two-step sequence (function_call -> follow-up assistant
    // message). Some environments can emit TurnComplete before the follow-up request is fully
    // observed by wiremock, so we wait for both TurnComplete and the expected request count.
    timeout(Duration::from_secs(10), async {
        wait_for_event(&codex, |event| matches!(event, EventMsg::TurnComplete(_))).await;
    })
    .await
    .expect("expected TurnComplete within timeout");

    timeout(Duration::from_secs(10), async {
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn custom__exec_worker_user__shell_runs_as_worker_user() -> Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    skip_if_windows!(Ok(()));

    let worker_user = "assistant";
    let (worker_uid, worker_gid) = {
        let Ok(c_user) = std::ffi::CString::new(worker_user) else {
            return Ok(());
        };
        // SAFETY: libc call, CString provides NUL-terminated pointer.
        let pw = unsafe { libc::getpwnam(c_user.as_ptr()) };
        if pw.is_null() {
            return Ok(());
        }
        // SAFETY: pw is non-null and points to a passwd struct.
        unsafe { ((*pw).pw_uid, (*pw).pw_gid) }
    };

    // SAFETY: libc call.
    let invoker_uid = unsafe { libc::geteuid() };
    if worker_uid == invoker_uid {
        return Ok(());
    }

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
        config.exec_run_as = Some(RunAsUser {
            uid: worker_uid as u32,
            gid: worker_gid as u32,
            supplementary_gids: None,
        });
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
        })
        .await?;

    timeout(Duration::from_secs(10), async {
        wait_for_event(&codex, |event| matches!(event, EventMsg::TurnComplete(_))).await;
    })
    .await
    .expect("expected TurnComplete within timeout");

    timeout(Duration::from_secs(10), async {
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
    let exit_code = output_text
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("Exit code: "))
        .and_then(|code| code.trim().parse::<i32>().ok())
        .unwrap_or(-1);

    if exit_code != 0 {
        let lower = output_text.to_lowercase();
        let is_perm = lower.contains("operation not permitted")
            || lower.contains("permission denied")
            || lower.contains("a password is required")
            || lower.contains("sudo:");
        if is_perm {
            return Ok(());
        }
    }

    assert_eq!(exit_code, 0, "output={output_text}");
    assert!(
        output_text
            .lines()
            .any(|line| line.trim() == (worker_uid as u32).to_string()),
        "expected worker uid in tool output; output={output_text}",
    );

    Ok(())
}
