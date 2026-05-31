use std::collections::HashMap;
use std::io;
use std::io::ErrorKind;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::AtomicBool;

use anyhow::Result;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::RunAsUser;
use crate::process::ChildTerminator;
use crate::process::ProcessHandle;
use crate::process::SpawnedProcess;

#[cfg(target_os = "linux")]
use libc;

#[cfg(unix)]
#[derive(Clone)]
struct RunAsRetry {
    run_as: RunAsUser,
    arg0: Option<String>,
    program: String,
    args: Vec<String>,
    cwd: std::path::PathBuf,
    env: HashMap<String, String>,
}

#[cfg(unix)]
fn ensure_argv0_symlink(program: &str, arg0: &str) -> io::Result<std::path::PathBuf> {
    let arg0 = std::path::Path::new(arg0)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::other("arg0 must be a valid UTF-8 file name"))?;
    if arg0.is_empty() {
        return Err(io::Error::other("arg0 must be non-empty"));
    }

    let dir = std::path::PathBuf::from("/tmp/codex-argv0");
    std::fs::create_dir_all(&dir)?;
    let link_path = dir.join(arg0);

    if let Ok(target) = std::fs::read_link(&link_path) {
        if target == std::path::Path::new(program) {
            return Ok(link_path);
        }
    }
    let _ = std::fs::remove_file(&link_path);
    std::os::unix::fs::symlink(program, &link_path)?;
    Ok(link_path)
}

#[cfg(unix)]
fn try_spawn_with_run_as_sudo(
    retry: RunAsRetry,
    stdin_mode: PipeStdinMode,
    inherited_fds: &[i32],
    err: io::Error,
) -> io::Result<SpawnedProcess> {
    if unsafe { libc::geteuid() } == 0
        || (err.kind() != io::ErrorKind::PermissionDenied
            && err.raw_os_error() != Some(libc::EPERM))
        || !inherited_fds.is_empty()
    {
        return Err(err);
    }

    let Some(sudo_program) = ["/usr/bin/sudo", "/bin/sudo"]
        .into_iter()
        .find(|path| std::path::Path::new(path).exists())
    else {
        return Err(err);
    };
    let Some(env_program) = ["/usr/bin/env", "/bin/env"]
        .into_iter()
        .find(|path| std::path::Path::new(path).exists())
    else {
        return Err(err);
    };

    let uid = format!("#{}", retry.run_as.uid);
    let gid = format!("#{}", retry.run_as.gid);
    let program = if let Some(arg0) = retry.arg0.as_deref() {
        ensure_argv0_symlink(retry.program.as_str(), arg0)?
            .to_string_lossy()
            .to_string()
    } else {
        retry.program
    };

    let mut command = Command::new(sudo_program);
    command.args([
        "-n",
        "-u",
        uid.as_str(),
        "-g",
        gid.as_str(),
        "--",
        env_program,
    ]);
    command.arg("-i");

    let mut env_kv: Vec<_> = retry.env.iter().collect();
    env_kv.sort_unstable_by_key(|(key, _)| *key);
    for (key, value) in env_kv {
        command.arg(format!("{key}={value}"));
    }
    command.arg(program);
    command.args(retry.args);
    command.current_dir(retry.cwd);
    command.env_clear();
    command.stdin(match stdin_mode {
        PipeStdinMode::Piped => Stdio::piped(),
        PipeStdinMode::Null => Stdio::null(),
    });
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.kill_on_drop(true);

    let mut child = command.spawn()?;
    let pid = child
        .id()
        .ok_or_else(|| io::Error::other("missing child pid"))?;
    #[cfg(unix)]
    let process_group_id = pid;

    let stdin = child.stdin.take();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let (writer_tx, mut writer_rx) = mpsc::channel::<Vec<u8>>(128);
    let (stdout_tx, stdout_rx) = mpsc::channel::<Vec<u8>>(128);
    let (stderr_tx, stderr_rx) = mpsc::channel::<Vec<u8>>(128);
    let writer_handle = if let Some(stdin) = stdin {
        tokio::spawn(async move {
            let mut writer = stdin;
            while let Some(bytes) = writer_rx.recv().await {
                let _ = writer.write_all(&bytes).await;
                let _ = writer.flush().await;
            }
        })
    } else {
        drop(writer_rx);
        tokio::spawn(async {})
    };

    let stdout_handle = stdout.map(|stdout| {
        let stdout_tx = stdout_tx.clone();
        tokio::spawn(async move {
            read_output_stream(BufReader::new(stdout), stdout_tx).await;
        })
    });
    let stderr_handle = stderr.map(|stderr| {
        let stderr_tx = stderr_tx.clone();
        tokio::spawn(async move {
            read_output_stream(BufReader::new(stderr), stderr_tx).await;
        })
    });
    let mut reader_abort_handles = Vec::new();
    if let Some(handle) = stdout_handle.as_ref() {
        reader_abort_handles.push(handle.abort_handle());
    }
    if let Some(handle) = stderr_handle.as_ref() {
        reader_abort_handles.push(handle.abort_handle());
    }
    let reader_handle = tokio::spawn(async move {
        if let Some(handle) = stdout_handle {
            let _ = handle.await;
        }
        if let Some(handle) = stderr_handle {
            let _ = handle.await;
        }
    });

    let (exit_tx, exit_rx) = oneshot::channel::<i32>();
    let exit_status = Arc::new(AtomicBool::new(false));
    let wait_exit_status = Arc::clone(&exit_status);
    let exit_code = Arc::new(StdMutex::new(None));
    let wait_exit_code = Arc::clone(&exit_code);
    let wait_handle: JoinHandle<()> = tokio::spawn(async move {
        let code = match child.wait().await {
            Ok(status) => status.code().unwrap_or(-1),
            Err(_) => -1,
        };
        wait_exit_status.store(true, std::sync::atomic::Ordering::SeqCst);
        if let Ok(mut guard) = wait_exit_code.lock() {
            *guard = Some(code);
        }
        let _ = exit_tx.send(code);
    });

    let handle = ProcessHandle::new(
        writer_tx,
        Box::new(PipeChildTerminator {
            #[cfg(windows)]
            pid,
            #[cfg(unix)]
            process_group_id,
        }),
        reader_handle,
        reader_abort_handles,
        writer_handle,
        wait_handle,
        exit_status,
        exit_code,
        /*pty_handles*/ None,
        /*resizer*/ None,
    );

    Ok(SpawnedProcess {
        session: handle,
        stdout_rx,
        stderr_rx,
        exit_rx,
    })
}

struct PipeChildTerminator {
    #[cfg(windows)]
    pid: u32,
    #[cfg(unix)]
    process_group_id: u32,
}

impl ChildTerminator for PipeChildTerminator {
    fn kill(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        {
            crate::process_group::kill_process_group(self.process_group_id)
        }

        #[cfg(windows)]
        {
            kill_process(self.pid)
        }

        #[cfg(not(any(unix, windows)))]
        {
            Ok(())
        }
    }
}

#[cfg(windows)]
fn kill_process(pid: u32) -> io::Result<()> {
    unsafe {
        let handle = winapi::um::processthreadsapi::OpenProcess(
            winapi::um::winnt::PROCESS_TERMINATE,
            0,
            pid,
        );
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let success = winapi::um::processthreadsapi::TerminateProcess(handle, 1);
        let err = io::Error::last_os_error();
        winapi::um::handleapi::CloseHandle(handle);
        if success == 0 { Err(err) } else { Ok(()) }
    }
}

async fn read_output_stream<R>(mut reader: R, output_tx: mpsc::Sender<Vec<u8>>)
where
    R: AsyncRead + Unpin,
{
    let mut buf = vec![0u8; 8_192];
    loop {
        match reader.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                let _ = output_tx.send(buf[..n].to_vec()).await;
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
}

#[derive(Clone, Copy)]
enum PipeStdinMode {
    Piped,
    Null,
}

async fn spawn_process_with_stdin_mode(
    program: &str,
    args: &[String],
    cwd: &Path,
    env: &HashMap<String, String>,
    arg0: &Option<String>,
    stdin_mode: PipeStdinMode,
    inherited_fds: &[i32],
    run_as: Option<RunAsUser>,
) -> Result<SpawnedProcess> {
    if program.is_empty() {
        anyhow::bail!("missing program for pipe spawn");
    }

    #[cfg(not(unix))]
    let _ = inherited_fds;

    let mut command = Command::new(program);
    #[cfg(unix)]
    if let Some(arg0) = arg0 {
        command.arg0(arg0);
    }
    #[cfg(target_os = "linux")]
    let parent_pid = unsafe { libc::getpid() };
    #[cfg(unix)]
    let inherited_fds = inherited_fds.to_vec();
    let inherited_fds_for_pre_exec = inherited_fds.clone();
    #[cfg(unix)]
    let run_as_retry = run_as.clone().map(|run_as| RunAsRetry {
        run_as,
        arg0: arg0.clone(),
        program: program.to_string(),
        args: args.to_vec(),
        cwd: cwd.to_path_buf(),
        env: env.clone(),
    });
    #[cfg(unix)]
    unsafe {
        command.pre_exec(move || {
            crate::process_group::detach_from_tty()?;
            #[cfg(target_os = "linux")]
            crate::process_group::set_parent_death_signal(parent_pid)?;
            crate::pty::close_inherited_fds_except(&inherited_fds_for_pre_exec);
            if let Some(run_as) = &run_as {
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
            }
            Ok(())
        });
    }
    #[cfg(not(unix))]
    let _ = arg0;
    command.current_dir(cwd);
    command.env_clear();
    for (key, value) in env {
        command.env(key, value);
    }
    for arg in args {
        command.arg(arg);
    }
    match stdin_mode {
        PipeStdinMode::Piped => {
            command.stdin(Stdio::piped());
        }
        PipeStdinMode::Null => {
            command.stdin(Stdio::null());
        }
    }
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            #[cfg(unix)]
            {
                if let Some(run_as_retry) = run_as_retry {
                    return Ok(try_spawn_with_run_as_sudo(
                        run_as_retry,
                        stdin_mode,
                        &inherited_fds,
                        err,
                    )?);
                }
            }
            return Err(err.into());
        }
    };
    let pid = child
        .id()
        .ok_or_else(|| io::Error::other("missing child pid"))?;
    #[cfg(unix)]
    let process_group_id = pid;

    let stdin = child.stdin.take();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let (writer_tx, mut writer_rx) = mpsc::channel::<Vec<u8>>(128);
    let (stdout_tx, stdout_rx) = mpsc::channel::<Vec<u8>>(128);
    let (stderr_tx, stderr_rx) = mpsc::channel::<Vec<u8>>(128);
    let writer_handle = if let Some(stdin) = stdin {
        tokio::spawn(async move {
            let mut writer = stdin;
            while let Some(bytes) = writer_rx.recv().await {
                let _ = writer.write_all(&bytes).await;
                let _ = writer.flush().await;
            }
        })
    } else {
        drop(writer_rx);
        tokio::spawn(async {})
    };

    let stdout_handle = stdout.map(|stdout| {
        let stdout_tx = stdout_tx.clone();
        tokio::spawn(async move {
            read_output_stream(BufReader::new(stdout), stdout_tx).await;
        })
    });
    let stderr_handle = stderr.map(|stderr| {
        let stderr_tx = stderr_tx.clone();
        tokio::spawn(async move {
            read_output_stream(BufReader::new(stderr), stderr_tx).await;
        })
    });
    let mut reader_abort_handles = Vec::new();
    if let Some(handle) = stdout_handle.as_ref() {
        reader_abort_handles.push(handle.abort_handle());
    }
    if let Some(handle) = stderr_handle.as_ref() {
        reader_abort_handles.push(handle.abort_handle());
    }
    let reader_handle = tokio::spawn(async move {
        if let Some(handle) = stdout_handle {
            let _ = handle.await;
        }
        if let Some(handle) = stderr_handle {
            let _ = handle.await;
        }
    });

    let (exit_tx, exit_rx) = oneshot::channel::<i32>();
    let exit_status = Arc::new(AtomicBool::new(false));
    let wait_exit_status = Arc::clone(&exit_status);
    let exit_code = Arc::new(StdMutex::new(None));
    let wait_exit_code = Arc::clone(&exit_code);
    let wait_handle: JoinHandle<()> = tokio::spawn(async move {
        let code = match child.wait().await {
            Ok(status) => status.code().unwrap_or(-1),
            Err(_) => -1,
        };
        wait_exit_status.store(true, std::sync::atomic::Ordering::SeqCst);
        if let Ok(mut guard) = wait_exit_code.lock() {
            *guard = Some(code);
        }
        let _ = exit_tx.send(code);
    });

    let handle = ProcessHandle::new(
        writer_tx,
        Box::new(PipeChildTerminator {
            #[cfg(windows)]
            pid,
            #[cfg(unix)]
            process_group_id,
        }),
        reader_handle,
        reader_abort_handles,
        writer_handle,
        wait_handle,
        exit_status,
        exit_code,
        /*pty_handles*/ None,
        /*resizer*/ None,
    );

    Ok(SpawnedProcess {
        session: handle,
        stdout_rx,
        stderr_rx,
        exit_rx,
    })
}

/// Spawn a process using regular pipes (no PTY), returning handles for stdin, split output, and exit.
pub async fn spawn_process(
    program: &str,
    args: &[String],
    cwd: &Path,
    env: &HashMap<String, String>,
    arg0: &Option<String>,
) -> Result<SpawnedProcess> {
    spawn_process_with_stdin_mode(
        program,
        args,
        cwd,
        env,
        arg0,
        PipeStdinMode::Piped,
        &[],
        None,
    )
    .await
}

/// Spawn a process using regular pipes, but close stdin immediately.
pub async fn spawn_process_no_stdin(
    program: &str,
    args: &[String],
    cwd: &Path,
    env: &HashMap<String, String>,
    arg0: &Option<String>,
) -> Result<SpawnedProcess> {
    spawn_process_no_stdin_with_inherited_fds(program, args, cwd, env, arg0, &[]).await
}

/// Spawn a process using regular pipes, close stdin immediately, and preserve
/// selected inherited file descriptors across exec on Unix.
pub async fn spawn_process_no_stdin_with_inherited_fds(
    program: &str,
    args: &[String],
    cwd: &Path,
    env: &HashMap<String, String>,
    arg0: &Option<String>,
    inherited_fds: &[i32],
) -> Result<SpawnedProcess> {
    spawn_process_with_stdin_mode(
        program,
        args,
        cwd,
        env,
        arg0,
        PipeStdinMode::Null,
        inherited_fds,
        None,
    )
    .await
}

/// Spawn a process using regular pipes, close stdin immediately, preserve
/// selected inherited file descriptors across exec, and optionally run as a
/// different uid/gid.
pub async fn spawn_process_no_stdin_with_inherited_fds_and_run_as(
    program: &str,
    args: &[String],
    cwd: &Path,
    env: &HashMap<String, String>,
    arg0: &Option<String>,
    inherited_fds: &[i32],
    run_as: Option<RunAsUser>,
) -> Result<SpawnedProcess> {
    spawn_process_with_stdin_mode(
        program,
        args,
        cwd,
        env,
        arg0,
        PipeStdinMode::Null,
        inherited_fds,
        run_as,
    )
    .await
}
