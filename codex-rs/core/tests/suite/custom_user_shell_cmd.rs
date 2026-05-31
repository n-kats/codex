#![cfg(not(target_os = "windows"))]
#![allow(non_snake_case)]

use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::ExecCommandEndEvent;
use codex_protocol::protocol::Op;
use core_test_support::responses::start_mock_server;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use std::str::FromStr;

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

#[tokio::test]
async fn custom__exec_worker_user__user_shell_stays_invoker_owned_even_with_worker_user() {
    let worker_user = "assistant";
    let Some((worker_uid, worker_gid)) = resolve_worker_user(worker_user) else {
        return;
    };

    // SAFETY: libc call.
    let invoker_uid = unsafe { libc::geteuid() };

    let server = start_mock_server().await;
    let mut builder = test_codex().with_config(move |config| {
        config.custom.exec.worker_user = Some(worker_user.to_string());
        config.custom.exec.worker_uid = Some(worker_uid);
        config.custom.exec.worker_gid = Some(worker_gid);
    });
    let fixture = builder
        .build(&server)
        .await
        .expect("create new conversation");
    let codex = fixture.codex.clone();

    codex
        .submit(Op::RunUserShellCommand {
            command: "id -u".to_string(),
        })
        .await
        .unwrap();

    let msg = wait_for_event(&codex, |ev| matches!(ev, EventMsg::ExecCommandEnd(_))).await;
    let EventMsg::ExecCommandEnd(ExecCommandEndEvent {
        stdout, exit_code, ..
    }) = msg
    else {
        unreachable!()
    };
    assert_eq!(exit_code, 0);
    let stdout = stdout.trim();
    let uid = u32::from_str(stdout).expect("id -u prints a uid");
    assert_eq!(uid, invoker_uid);
    assert!(
        uid != worker_uid,
        "user shell command must not run as worker"
    );
}

#[tokio::test]
async fn custom__user_shell_no_inject__bang_result_not_recorded_locally() {
    let server = start_mock_server().await;
    let mut builder = test_codex().with_config(move |config| {
        config.custom.user_shell.no_inject = Some(true);
    });
    let fixture = builder
        .build(&server)
        .await
        .expect("create new conversation");
    let codex = fixture.codex.clone();

    codex
        .submit(Op::RunUserShellCommand {
            command: "echo injected?".to_string(),
        })
        .await
        .unwrap();

    let _ = wait_for_event(&codex, |ev| matches!(ev, EventMsg::ExecCommandEnd(_))).await;

    let Some(rollout_path) = codex.rollout_path() else {
        return;
    };
    if !rollout_path.exists() {
        return;
    }
    let text = std::fs::read_to_string(&rollout_path).expect("read rollout file");
    assert!(
        !text.contains("<user_shell_command>"),
        "expected no user_shell_command record in rollout, got: {text}"
    );
    assert!(
        !text.contains("echo injected?"),
        "expected raw bang command not to be recorded, got: {text}"
    );
}
