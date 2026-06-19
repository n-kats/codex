use crate::custom::exec::RunAsUser;
use codex_config::custom::CustomExecToml;
use std::io::ErrorKind;

const CUSTOM_EXEC_CURRENT_USER_WARNING: &str = "custom.exec.* resolves to the current user; model-triggered commands will run as the invoker user (same as `!`/UserShell), so privilege separation is not in effect. Configure a different worker user/uid/gid to enable separation.";

pub(super) fn resolve_run_as(exec: &CustomExecToml) -> std::io::Result<Option<RunAsUser>> {
    let worker_user = exec
        .worker_user
        .as_ref()
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());

    #[cfg(not(unix))]
    {
        if worker_user.is_some() || exec.worker_uid.is_some() || exec.worker_gid.is_some() {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "custom.exec is only supported on unix targets",
            ));
        }
        Ok(None)
    }

    #[cfg(unix)]
    {
        let ids = match (exec.worker_uid, exec.worker_gid) {
            (None, None) => None,
            (Some(uid), Some(gid)) => Some(RunAsUser {
                uid,
                gid,
                supplementary_gids: None,
            }),
            _ => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "custom.exec.worker_uid and custom.exec.worker_gid must be set together",
                ));
            }
        };

        let user_ids = if let Some(user) = worker_user {
            Some(resolve_user_to_ids(user)?)
        } else {
            None
        };

        match (user_ids, ids) {
            (None, None) => Ok(None),
            (Some(user_ids), None) => Ok(Some(user_ids)),
            (None, Some(ids)) => Ok(Some(ids)),
            (Some(user_ids), Some(ids)) => {
                if user_ids.uid == ids.uid && user_ids.gid == ids.gid {
                    Ok(Some(user_ids))
                } else {
                    Err(std::io::Error::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "custom.exec.worker_user resolved to uid/gid {}/{} but custom.exec.worker_uid/gid is {}/{}",
                            user_ids.uid, user_ids.gid, ids.uid, ids.gid
                        ),
                    ))
                }
            }
        }
    }
}

pub(super) fn warn_if_current_user(
    exec: &CustomExecToml,
    run_as: Option<&RunAsUser>,
    startup_warnings: &mut Vec<String>,
) {
    let configured = exec
        .worker_user
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        || exec.worker_uid.is_some()
        || exec.worker_gid.is_some();
    #[cfg(unix)]
    if configured && run_as.is_some_and(|run_as| run_as.uid == unsafe { libc::geteuid() }) {
        startup_warnings.push(CUSTOM_EXEC_CURRENT_USER_WARNING.to_string());
    }
    #[cfg(not(unix))]
    let _ = (configured, run_as, startup_warnings);
}

#[cfg(unix)]
fn resolve_user_to_ids(user: &str) -> std::io::Result<RunAsUser> {
    use std::ffi::CString;
    use std::ptr;

    let user = CString::new(user).map_err(|_| {
        std::io::Error::new(
            ErrorKind::InvalidInput,
            "custom.exec.worker_user must not contain NUL bytes",
        )
    })?;

    let mut buf_len = 1024usize;
    loop {
        let mut pwd = std::mem::MaybeUninit::<libc::passwd>::uninit();
        let mut result = ptr::null_mut();
        let mut buf = vec![0 as libc::c_char; buf_len];
        let status = unsafe {
            libc::getpwnam_r(
                user.as_ptr(),
                pwd.as_mut_ptr(),
                buf.as_mut_ptr(),
                buf.len(),
                &mut result,
            )
        };
        if status == libc::ERANGE {
            buf_len *= 2;
            continue;
        }
        if status != 0 {
            return Err(std::io::Error::from_raw_os_error(status));
        }
        if result.is_null() {
            return Err(std::io::Error::new(
                ErrorKind::NotFound,
                "custom.exec.worker_user does not exist",
            ));
        }

        let pwd = unsafe { pwd.assume_init() };
        let supplementary_gids = resolve_user_supplementary_gids(&user, pwd.pw_gid)?;
        return Ok(RunAsUser {
            uid: pwd.pw_uid,
            gid: pwd.pw_gid,
            supplementary_gids: Some(supplementary_gids),
        });
    }
}

#[cfg(unix)]
fn resolve_user_supplementary_gids(
    user: &std::ffi::CStr,
    primary_gid: libc::gid_t,
) -> std::io::Result<Vec<u32>> {
    let mut group_count = 0;
    unsafe {
        libc::getgrouplist(
            user.as_ptr(),
            primary_gid,
            std::ptr::null_mut(),
            &mut group_count,
        );
    }
    if group_count <= 0 {
        return Ok(Vec::new());
    }

    let mut groups = vec![0 as libc::gid_t; group_count as usize];
    let result = unsafe {
        libc::getgrouplist(
            user.as_ptr(),
            primary_gid,
            groups.as_mut_ptr(),
            &mut group_count,
        )
    };
    if result >= 0 {
        groups.truncate(group_count as usize);
        return Ok(groups.into_iter().map(|gid| gid as u32).collect());
    }

    Err(std::io::Error::new(
        ErrorKind::Other,
        "failed to resolve custom.exec.worker_user supplementary groups",
    ))
}
