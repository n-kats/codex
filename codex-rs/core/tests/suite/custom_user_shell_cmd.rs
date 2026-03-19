#![allow(non_snake_case)]
use codex_core::spawn::RunAsUser;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::ExecCommandEndEvent;
use codex_protocol::protocol::Op;
use core_test_support::responses::start_mock_server;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use std::str::FromStr;

#[tokio::test]
#[cfg(not(target_os = "windows"))]
async fn custom__exec_worker_user__exec_worker_user設定時もinvokerで実行される() {
    // This test ensures `!` (UserShell) stays invoker-owned, even when model-run command execution
    // is configured to run under a worker user (e.g. "assistant").

    let worker_user = "assistant";
    let (worker_uid, worker_gid) = {
        let Ok(c_user) = std::ffi::CString::new(worker_user) else {
            return;
        };
        // SAFETY: libc call, CString provides NUL-terminated pointer.
        let pw = unsafe { libc::getpwnam(c_user.as_ptr()) };
        if pw.is_null() {
            return;
        }
        // SAFETY: pw is non-null and points to a passwd struct.
        unsafe { ((*pw).pw_uid, (*pw).pw_gid) }
    };

    // SAFETY: libc call.
    let invoker_uid = unsafe { libc::geteuid() };
    if worker_uid == invoker_uid {
        return;
    }

    let server = start_mock_server().await;
    let mut builder = test_codex().with_config(move |config| {
        config.exec_run_as = Some(RunAsUser {
            uid: worker_uid,
            gid: worker_gid,
            supplementary_gids: None,
        });
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
#[cfg(not(target_os = "windows"))]
async fn custom__user_shell_no_inject__bang結果をローカル記録しない() {
    let server = start_mock_server().await;
    let mut builder = test_codex().with_config(move |config| {
        config.user_shell_no_inject = true;
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

    let rollout_path = codex.rollout_path().expect("rollout path");
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
