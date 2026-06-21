mod exec;
mod user_shell;

use crate::custom::exec::RunAsUser;
use codex_config::custom::CustomConfigToml;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use codex_protocol::config_types::ShellEnvironmentPolicyInherit;

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct CustomPermissions {
    assistant_shell_environment_policy: Option<ShellEnvironmentPolicy>,
    user_shell_environment_policy: Option<ShellEnvironmentPolicy>,
    pub(crate) user_shell_no_inject: bool,
    pub(crate) exec_run_as: Option<RunAsUser>,
}

impl CustomPermissions {
    pub(crate) fn assistant_shell_environment_policy<'a>(
        &'a self,
        base: &'a ShellEnvironmentPolicy,
    ) -> &'a ShellEnvironmentPolicy {
        self.assistant_shell_environment_policy
            .as_ref()
            .unwrap_or(base)
    }

    pub(crate) fn user_shell_environment_policy<'a>(
        &'a self,
        base: &'a ShellEnvironmentPolicy,
    ) -> &'a ShellEnvironmentPolicy {
        self.user_shell_environment_policy
            .as_ref()
            .or(self.assistant_shell_environment_policy.as_ref())
            .unwrap_or(base)
    }
}

pub(crate) fn resolve_custom_config(
    custom: &CustomConfigToml,
    shell_environment_policy: &ShellEnvironmentPolicy,
    startup_warnings: &mut Vec<String>,
) -> std::io::Result<CustomPermissions> {
    let assistant_shell_environment_policy = custom
        .assistant_shell_environment_policy
        .clone()
        .map(ShellEnvironmentPolicy::from);
    let user_shell_environment_policy = custom
        .user_shell_environment_policy
        .clone()
        .map(ShellEnvironmentPolicy::from);
    let custom_exec_run_as = exec::resolve_run_as(&custom.exec)?;
    if custom_exec_run_as.is_some()
        && matches!(
            assistant_shell_environment_policy
                .as_ref()
                .unwrap_or(shell_environment_policy)
                .inherit,
            ShellEnvironmentPolicyInherit::All
        )
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "custom.exec is configured, but shell_environment_policy.inherit is 'all'; refusing because a model-run `env`/`printenv` would leak the invoker environment (set inherit = 'core' or 'none', or use include_only).",
        ));
    }
    exec::warn_if_current_user(&custom.exec, custom_exec_run_as.as_ref(), startup_warnings);
    let user_shell_no_inject = user_shell::resolve_no_inject(&custom.user_shell, startup_warnings);

    Ok(CustomPermissions {
        assistant_shell_environment_policy,
        user_shell_environment_policy,
        user_shell_no_inject,
        exec_run_as: custom_exec_run_as,
    })
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use codex_config::custom::CustomConfigToml;
    use codex_config::custom::CustomExecToml;
    use codex_protocol::config_types::ShellEnvironmentPolicyInherit;
    use pretty_assertions::assert_eq;

    fn resolve(custom: CustomConfigToml) -> std::io::Result<CustomPermissions> {
        resolve_with_warnings(custom).map(|(custom, _startup_warnings)| custom)
    }

    fn resolve_with_warnings(
        custom: CustomConfigToml,
    ) -> std::io::Result<(CustomPermissions, Vec<String>)> {
        let mut startup_warnings = Vec::new();
        let custom = resolve_custom_config(
            &custom,
            &ShellEnvironmentPolicy {
                inherit: ShellEnvironmentPolicyInherit::Core,
                ..Default::default()
            },
            &mut startup_warnings,
        )?;
        Ok((custom, startup_warnings))
    }

    #[cfg(unix)]
    fn current_user_name() -> String {
        let uid = unsafe { libc::geteuid() };
        let mut buf_len = 1024usize;
        loop {
            let mut pwd = std::mem::MaybeUninit::<libc::passwd>::uninit();
            let mut result = std::ptr::null_mut();
            let mut buf = vec![0 as libc::c_char; buf_len];
            let status = unsafe {
                libc::getpwuid_r(
                    uid,
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
            assert_eq!(status, 0, "current user should resolve through getpwuid_r");
            assert!(
                !result.is_null(),
                "current user should resolve through getpwuid_r"
            );

            return unsafe { std::ffi::CStr::from_ptr((*pwd.as_ptr()).pw_name) }
                .to_string_lossy()
                .into_owned();
        }
    }

    #[test]
    fn custom__exec_worker_user__requires_uid_and_gid_together() {
        let err = resolve(CustomConfigToml {
            exec: CustomExecToml {
                worker_uid: Some(1001),
                worker_gid: None,
                ..Default::default()
            },
            ..Default::default()
        })
        .expect_err("worker_uid without worker_gid should be rejected");

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(
            err.to_string()
                .contains("custom.exec.worker_uid and custom.exec.worker_gid must be set together"),
            "{err}"
        );
    }

    #[test]
    fn custom__exec_worker_user__accepts_uid_gid_pair() {
        let custom = resolve(CustomConfigToml {
            exec: CustomExecToml {
                worker_uid: Some(1001),
                worker_gid: Some(1002),
                ..Default::default()
            },
            ..Default::default()
        })
        .expect("worker uid/gid pair should resolve");

        assert_eq!(
            custom.exec_run_as,
            Some(RunAsUser {
                uid: 1001,
                gid: 1002,
                supplementary_gids: None,
            })
        );
    }

    #[cfg(unix)]
    #[test]
    fn custom__exec_worker_user__resolves_to_current_user_adds_startup_warning() {
        let (_custom, startup_warnings) = resolve_with_warnings(CustomConfigToml {
            exec: CustomExecToml {
                worker_uid: Some(unsafe { libc::geteuid() }),
                worker_gid: Some(unsafe { libc::getegid() }),
                ..Default::default()
            },
            ..Default::default()
        })
        .expect("current uid/gid pair should resolve");

        assert_eq!(
            startup_warnings,
            vec![
                "custom.exec.* resolves to the current user; model-triggered commands will run as the invoker user (same as `!`/UserShell), so privilege separation is not in effect. Configure a different worker user/uid/gid to enable separation."
                    .to_string()
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn custom__exec_worker_user__accepts_matching_user_and_uid_gid() {
        let user = current_user_name();
        let uid = unsafe { libc::geteuid() };
        let gid = unsafe { libc::getegid() };

        let custom = resolve(CustomConfigToml {
            exec: CustomExecToml {
                worker_user: Some(user),
                worker_uid: Some(uid),
                worker_gid: Some(gid),
            },
            ..Default::default()
        })
        .expect("matching worker_user and uid/gid should resolve");

        assert_eq!(
            custom
                .exec_run_as
                .as_ref()
                .map(|run_as| (run_as.uid, run_as.gid)),
            Some((uid, gid))
        );
        assert!(
            custom
                .exec_run_as
                .as_ref()
                .and_then(|run_as| run_as.supplementary_gids.as_ref())
                .is_some()
        );
    }

    #[cfg(unix)]
    #[test]
    fn custom__exec_worker_user__rejects_mismatched_user_and_uid_gid() {
        let user = current_user_name();
        let current_uid = unsafe { libc::geteuid() };
        let mismatched_uid = if current_uid == u32::MAX {
            current_uid - 1
        } else {
            current_uid + 1
        };

        let err = resolve(CustomConfigToml {
            exec: CustomExecToml {
                worker_user: Some(user),
                worker_uid: Some(mismatched_uid),
                worker_gid: Some(unsafe { libc::getegid() }),
            },
            ..Default::default()
        })
        .expect_err("mismatched worker_user and uid/gid should be rejected");

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(
            err.to_string()
                .contains("custom.exec.worker_user resolved to uid/gid")
        );
    }

    #[cfg(unix)]
    #[test]
    fn custom__exec_worker_user__resolves_worker_user_to_ids_and_groups() {
        let custom = resolve(CustomConfigToml {
            exec: CustomExecToml {
                worker_user: Some(current_user_name()),
                ..Default::default()
            },
            ..Default::default()
        })
        .expect("worker_user should resolve through system user database");

        assert_eq!(
            custom
                .exec_run_as
                .as_ref()
                .map(|run_as| (run_as.uid, run_as.gid)),
            Some((unsafe { libc::geteuid() }, unsafe { libc::getegid() }))
        );
        assert!(
            custom
                .exec_run_as
                .as_ref()
                .and_then(|run_as| run_as.supplementary_gids.as_ref())
                .is_some()
        );
    }

    #[test]
    fn custom__exec_worker_user__rejects_inherit_all_shell_environment_policy() {
        let mut startup_warnings = Vec::new();
        let err = resolve_custom_config(
            &CustomConfigToml {
                exec: CustomExecToml {
                    worker_uid: Some(1001),
                    worker_gid: Some(1002),
                    ..Default::default()
                },
                ..Default::default()
            },
            &ShellEnvironmentPolicy {
                inherit: ShellEnvironmentPolicyInherit::All,
                ..Default::default()
            },
            &mut startup_warnings,
        )
        .expect_err("worker user with inherit=all should be rejected");

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(
            err.to_string()
                .contains("shell_environment_policy.inherit is 'all'")
        );
    }
}
