use crate::config::types::ShellEnvironmentPolicyToml;
use crate::spawn::RunAsUser;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Base config deserialized from ~/.codex/config.toml.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct CustomExecToml {
    /// Run model-triggered command execution as a dedicated OS user.
    ///
    /// See `_docs/custom_notes/command_exec_worker_user/README.md`.
    pub worker_user: Option<String>,
    /// Optional numeric override for worker UID.
    pub worker_uid: Option<u32>,
    /// Optional numeric override for worker GID.
    pub worker_gid: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct CustomThemeDiffToml {
    /// Enable diff color styling. When false, all diff-specific coloring is disabled.
    pub enabled: Option<bool>,
    /// Enable add/delete line background tint.
    pub line_bg: Option<bool>,
    /// Enable line-number gutter styling.
    pub gutter: Option<bool>,
    /// Enable `+` / `-` sign coloring.
    pub sign: Option<bool>,
    /// Enable non-syntax diff content styling.
    pub content: Option<bool>,
    /// Hex color (`#RRGGBB`) for added diff line backgrounds in the TUI.
    pub add_line_bg: Option<String>,
    /// Hex color (`#RRGGBB`) for deleted diff line backgrounds in the TUI.
    pub del_line_bg: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct CustomThemeToml {
    #[serde(default)]
    pub diff: CustomThemeDiffToml,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct CustomUserShellToml {
    /// When `true`, `!` (UserShell) command output is not injected into the
    /// model context and is not persisted to the local session history.
    pub no_inject: Option<bool>,
}

/// Fork-specific custom settings under `[custom]`.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct CustomConfigToml {
    #[serde(default)]
    pub exec: CustomExecToml,

    #[serde(default)]
    pub user_shell: CustomUserShellToml,

    /// Override the shell environment policy used by model-triggered command execution.
    ///
    /// When unset, falls back to `[shell_environment_policy]`.
    pub assistant_shell_environment_policy: Option<ShellEnvironmentPolicyToml>,

    /// Override the shell environment policy used by `!` (UserShell).
    ///
    /// When unset, falls back to the assistant-resolved policy.
    pub user_shell_environment_policy: Option<ShellEnvironmentPolicyToml>,

    /// Custom TUI theme overrides.
    #[serde(default)]
    pub theme: Option<CustomThemeToml>,
}

pub(crate) fn resolve_exec_run_as(
    custom: Option<&CustomConfigToml>,
) -> std::io::Result<Option<RunAsUser>> {
    let Some(custom) = custom else {
        return Ok(None);
    };

    let worker_user = custom
        .exec
        .worker_user
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let worker_uid = custom.exec.worker_uid;
    let worker_gid = custom.exec.worker_gid;

    if worker_uid.is_some() != worker_gid.is_some() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "custom.exec.worker_uid and custom.exec.worker_gid must be set together",
        ));
    }

    #[cfg(not(unix))]
    {
        if worker_user.is_some() || worker_uid.is_some() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "custom.exec is only supported on Unix targets",
            ));
        }
        return Ok(None);
    }

    #[cfg(unix)]
    {
        let resolved_from_user = worker_user.map(lookup_run_as_user_by_name).transpose()?;

        match (resolved_from_user, worker_uid, worker_gid) {
            (None, None, None) => Ok(None),
            (None, Some(uid), Some(gid)) => Ok(Some(RunAsUser {
                uid,
                gid,
                supplementary_gids: None,
            })),
            (Some(resolved), None, None) => Ok(Some(resolved)),
            (Some(resolved), Some(uid), Some(gid)) => {
                if resolved.uid != uid || resolved.gid != gid {
                    let resolved_uid = resolved.uid;
                    let resolved_gid = resolved.gid;
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "custom.exec.worker_user resolves to uid/gid {resolved_uid}/{resolved_gid} but custom.exec.worker_uid/gid is {uid}/{gid}"
                        ),
                    ));
                }
                Ok(Some(resolved))
            }
            (None, Some(_), None)
            | (None, None, Some(_))
            | (Some(_), Some(_), None)
            | (Some(_), None, Some(_)) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "custom.exec.worker_uid and custom.exec.worker_gid must be set together",
            )),
        }
    }
}

#[cfg(unix)]
fn lookup_run_as_user_by_name(user: &str) -> std::io::Result<RunAsUser> {
    use std::ffi::CString;

    let c_user = CString::new(user).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("custom.exec.worker_user is not a valid C string: {err}"),
        )
    })?;

    let pw = unsafe { libc::getpwnam(c_user.as_ptr()) };
    if pw.is_null() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("custom.exec.worker_user `{user}` not found"),
        ));
    }
    let uid = unsafe { (*pw).pw_uid as u32 };
    let gid = unsafe { (*pw).pw_gid as u32 };

    let supplementary_gids = Some(lookup_supplementary_gids(user, gid)?);
    Ok(RunAsUser {
        uid,
        gid,
        supplementary_gids,
    })
}

#[cfg(unix)]
fn lookup_supplementary_gids(user: &str, gid: u32) -> std::io::Result<Vec<u32>> {
    use std::ffi::CString;

    let c_user = CString::new(user).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("custom.exec.worker_user is not a valid C string: {err}"),
        )
    })?;

    let mut groups: Vec<libc::gid_t> = vec![0; 16];
    let mut ngroups: libc::c_int = groups.len() as libc::c_int;
    let mut rv = unsafe {
        libc::getgrouplist(
            c_user.as_ptr(),
            gid as libc::gid_t,
            groups.as_mut_ptr(),
            &mut ngroups,
        )
    };

    if rv == -1 {
        let needed = ngroups.max(0) as usize;
        groups.resize(needed.max(16), 0);
        ngroups = groups.len() as libc::c_int;
        rv = unsafe {
            libc::getgrouplist(
                c_user.as_ptr(),
                gid as libc::gid_t,
                groups.as_mut_ptr(),
                &mut ngroups,
            )
        };
    }

    if rv == -1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("failed to resolve supplementary groups for custom.exec.worker_user `{user}`"),
        ));
    }

    let count = (ngroups.max(0) as usize).min(groups.len());
    Ok(groups.into_iter().take(count).map(|g| g as u32).collect())
}

pub(crate) fn parse_hex_rgb(value: &str, field_name: &str) -> std::io::Result<(u8, u8, u8)> {
    let trimmed = value.trim();
    let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);
    if hex.len() != 6 || !hex.as_bytes().iter().all(|b| b.is_ascii_hexdigit()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{field_name} must be a hex color like #RRGGBB"),
        ));
    }

    let parse_channel = |range: std::ops::Range<usize>| -> std::io::Result<u8> {
        u8::from_str_radix(&hex[range], 16).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{field_name} contains invalid hex digits: {err}"),
            )
        })
    };

    Ok((
        parse_channel(0..2)?,
        parse_channel(2..4)?,
        parse_channel(4..6)?,
    ))
}
