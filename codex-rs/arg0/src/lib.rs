use std::ffi::OsString;
use std::future::Future;
use std::path::Path;
use std::path::PathBuf;

use codex_core::CODEX_APPLY_PATCH_ARG1;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use tempfile::TempDir;

const LINUX_SANDBOX_ARG0: &str = "codex-linux-sandbox";
const APPLY_PATCH_ARG0: &str = "apply_patch";
const MISSPELLED_APPLY_PATCH_ARG0: &str = "applypatch";

pub fn arg0_dispatch() -> Option<TempDir> {
    // Determine if we were invoked via the special alias.
    let mut args = std::env::args_os();
    let argv0 = args.next().unwrap_or_default();
    let exe_name = Path::new(&argv0)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    if exe_name == LINUX_SANDBOX_ARG0 {
        // Safety: [`run_main`] never returns.
        codex_linux_sandbox::run_main();
    } else if exe_name == APPLY_PATCH_ARG0 || exe_name == MISSPELLED_APPLY_PATCH_ARG0 {
        codex_apply_patch::main();
    }

    let argv1 = args.next().unwrap_or_default();
    if argv1 == CODEX_APPLY_PATCH_ARG1 {
        let patch_arg = args.next().and_then(|s| s.to_str().map(str::to_owned));
        let exit_code = match patch_arg {
            Some(patch_arg) => {
                let mut stdout = std::io::stdout();
                let mut stderr = std::io::stderr();
                match codex_apply_patch::apply_patch(&patch_arg, &mut stdout, &mut stderr) {
                    Ok(()) => 0,
                    Err(_) => 1,
                }
            }
            None => {
                eprintln!("Error: {CODEX_APPLY_PATCH_ARG1} requires a UTF-8 PATCH argument.");
                1
            }
        };
        std::process::exit(exit_code);
    }

    // This modifies the environment, which is not thread-safe, so do this
    // before creating any threads/the Tokio runtime.
    apply_codex_home_override_from_args();
    apply_shell_startup_files_override_from_args();
    load_dotenv();

    match prepend_path_entry_for_codex_aliases() {
        Ok(path_entry) => Some(path_entry),
        Err(err) => {
            // It is possible that Codex will proceed successfully even if
            // updating the PATH fails, so warn the user and move on.
            eprintln!("WARNING: proceeding, even though we could not update PATH: {err}");
            None
        }
    }
}

/// While we want to deploy the Codex CLI as a single executable for simplicity,
/// we also want to expose some of its functionality as distinct CLIs, so we use
/// the "arg0 trick" to determine which CLI to dispatch. This effectively allows
/// us to simulate deploying multiple executables as a single binary on Mac and
/// Linux (but not Windows).
///
/// When the current executable is invoked through the hard-link or alias named
/// `codex-linux-sandbox` we *directly* execute
/// [`codex_linux_sandbox::run_main`] (which never returns). Otherwise we:
///
/// 1.  Load `.env` values from `~/.codex/.env` before creating any threads.
/// 2.  Construct a Tokio multi-thread runtime.
/// 3.  Derive the path to the current executable (so children can re-invoke the
///     sandbox) when running on Linux.
/// 4.  Execute the provided async `main_fn` inside that runtime, forwarding any
///     error. Note that `main_fn` receives `codex_linux_sandbox_exe:
///     Option<PathBuf>`, as an argument, which is generally needed as part of
///     constructing [`codex_core::config::Config`].
///
/// This function should be used to wrap any `main()` function in binary crates
/// in this workspace that depends on these helper CLIs.
pub fn arg0_dispatch_or_else<F, Fut>(main_fn: F) -> anyhow::Result<()>
where
    F: FnOnce(Option<PathBuf>) -> Fut,
    Fut: Future<Output = anyhow::Result<()>>,
{
    // Retain the TempDir so it exists for the lifetime of the invocation of
    // this executable. Admittedly, we could invoke `keep()` on it, but it
    // would be nice to avoid leaving temporary directories behind, if possible.
    let _path_entry = arg0_dispatch();

    // Regular invocation – create a Tokio runtime and execute the provided
    // async entry-point.
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async move {
        let codex_linux_sandbox_exe: Option<PathBuf> = if cfg!(target_os = "linux") {
            std::env::current_exe().ok()
        } else {
            None
        };

        main_fn(codex_linux_sandbox_exe).await
    })
}

const ILLEGAL_ENV_VAR_PREFIX: &str = "CODEX_";
const CODEX_HOME_CLI_FLAG: &str = "--codex-home";
const SHELL_STARTUP_FILES_CLI_FLAG: &str = "--shell-startup-files";
const CODEX_SHELL_STARTUP_FILES_ENV_VAR: &str = "CODEX_SHELL_STARTUP_FILES";

/// Load env vars from ~/.codex/.env.
///
/// Security: Do not allow `.env` files to create or modify any variables
/// with names starting with `CODEX_`.
fn load_dotenv() {
    if let Ok(codex_home) = codex_core::config::find_codex_home()
        && let Ok(iter) = dotenvy::from_path_iter(codex_home.join(".env"))
    {
        set_filtered(iter);
    }
}

fn apply_codex_home_override_from_args() {
    let Some(codex_home) = parse_codex_home_flag(std::env::args_os()) else {
        return;
    };

    // It is safe to call set_var() because our process is single-threaded at this point in its
    // execution (before the Tokio runtime is created).
    unsafe { std::env::set_var("CODEX_HOME", &codex_home) };

    if let Err(err) = std::fs::create_dir_all(&codex_home) {
        eprintln!(
            "WARNING: proceeding, even though we could not create CODEX_HOME directory {}: {err}",
            codex_home.display()
        );
    }
}

fn parse_codex_home_flag<I>(mut args: I) -> Option<PathBuf>
where
    I: Iterator<Item = OsString>,
{
    // Skip argv0.
    let _ = args.next();

    while let Some(arg) = args.next() {
        let Some(arg) = arg.to_str() else {
            continue;
        };

        if let Some((flag, value)) = arg.split_once('=') {
            if flag == CODEX_HOME_CLI_FLAG && !value.is_empty() {
                return Some(PathBuf::from(value));
            }
            continue;
        }

        if arg == CODEX_HOME_CLI_FLAG {
            return args.next().and_then(|s| s.to_str().map(PathBuf::from));
        }
    }

    None
}

fn apply_shell_startup_files_override_from_args() {
    let Some(mode) = parse_shell_startup_files_flag(std::env::args_os()) else {
        return;
    };

    // It is safe to call set_var() because our process is single-threaded at this point in its
    // execution (before the Tokio runtime is created).
    unsafe { std::env::set_var(CODEX_SHELL_STARTUP_FILES_ENV_VAR, mode) };
}

fn parse_shell_startup_files_flag<I>(mut args: I) -> Option<String>
where
    I: Iterator<Item = OsString>,
{
    // Skip argv0.
    let _ = args.next();

    while let Some(arg) = args.next() {
        let Some(arg) = arg.to_str() else {
            continue;
        };

        if let Some((flag, value)) = arg.split_once('=') {
            if flag == SHELL_STARTUP_FILES_CLI_FLAG && !value.is_empty() {
                return Some(value.to_string());
            }
            continue;
        }

        if arg == SHELL_STARTUP_FILES_CLI_FLAG {
            return args.next().and_then(|s| s.to_str().map(str::to_string));
        }
    }

    None
}

/// Helper to set vars from a dotenvy iterator while filtering out `CODEX_` keys.
fn set_filtered<I>(iter: I)
where
    I: IntoIterator<Item = Result<(String, String), dotenvy::Error>>,
{
    for (key, value) in iter.into_iter().flatten() {
        if !key.to_ascii_uppercase().starts_with(ILLEGAL_ENV_VAR_PREFIX) {
            // It is safe to call set_var() because our process is
            // single-threaded at this point in its execution.
            unsafe { std::env::set_var(&key, &value) };
        }
    }
}

/// Creates a temporary directory with either:
///
/// - UNIX: `apply_patch` symlink to the current executable
/// - WINDOWS: `apply_patch.bat` batch script to invoke the current executable
///   with the "secret" --codex-run-as-apply-patch flag.
///
/// This temporary directory is prepended to the PATH environment variable so
/// that `apply_patch` can be on the PATH without requiring the user to
/// install a separate `apply_patch` executable, simplifying the deployment of
/// Codex CLI.
/// Note: In debug builds the temp-dir guard is disabled to ease local testing.
///
/// IMPORTANT: This function modifies the PATH environment variable, so it MUST
/// be called before multiple threads are spawned.
pub fn prepend_path_entry_for_codex_aliases() -> std::io::Result<TempDir> {
    let codex_home = codex_core::config::find_codex_home()?;
    #[cfg(not(debug_assertions))]
    {
        // Guard against placing helpers in system temp directories outside debug builds.
        let temp_root = std::env::temp_dir();
        if codex_home.starts_with(&temp_root) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Refusing to create helper binaries under temporary dir {temp_root:?} (codex_home: {codex_home:?})"
                ),
            ));
        }
    }

    std::fs::create_dir_all(&codex_home)?;
    // Use a CODEX_HOME-scoped temp root to avoid cluttering the top-level directory.
    let temp_root = codex_home.join("tmp").join("path");
    std::fs::create_dir_all(&temp_root)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Ensure only the current user can access the temp directory.
        std::fs::set_permissions(&temp_root, std::fs::Permissions::from_mode(0o700))?;
    }

    let temp_dir = tempfile::Builder::new()
        .prefix("codex-arg0")
        .tempdir_in(&temp_root)?;
    let path = temp_dir.path();

    for filename in &[
        APPLY_PATCH_ARG0,
        MISSPELLED_APPLY_PATCH_ARG0,
        #[cfg(target_os = "linux")]
        LINUX_SANDBOX_ARG0,
    ] {
        let exe = std::env::current_exe()?;

        #[cfg(unix)]
        {
            let link = path.join(filename);
            symlink(&exe, &link)?;
        }

        #[cfg(windows)]
        {
            let batch_script = path.join(format!("{filename}.bat"));
            std::fs::write(
                &batch_script,
                format!(
                    r#"@echo off
"{}" {CODEX_APPLY_PATCH_ARG1} %*
"#,
                    exe.display()
                ),
            )?;
        }
    }

    #[cfg(unix)]
    const PATH_SEPARATOR: &str = ":";

    #[cfg(windows)]
    const PATH_SEPARATOR: &str = ";";

    let path_element = path.display();
    let updated_path_env_var = match std::env::var("PATH") {
        Ok(existing_path) => {
            format!("{path_element}{PATH_SEPARATOR}{existing_path}")
        }
        Err(_) => {
            format!("{path_element}")
        }
    };

    unsafe {
        std::env::set_var("PATH", updated_path_env_var);
    }

    Ok(temp_dir)
}

#[cfg(test)]
mod tests {
    use super::parse_codex_home_flag;
    use pretty_assertions::assert_eq;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn parses_codex_home_with_equals_syntax() {
        let args = vec![
            OsString::from("codex"),
            OsString::from("--codex-home=/tmp/codex-home"),
        ];
        assert_eq!(
            Some(PathBuf::from("/tmp/codex-home")),
            parse_codex_home_flag(args.into_iter())
        );
    }

    #[test]
    fn parses_codex_home_with_separate_value() {
        let args = vec![
            OsString::from("codex"),
            OsString::from("--codex-home"),
            OsString::from("/tmp/codex-home"),
        ];
        assert_eq!(
            Some(PathBuf::from("/tmp/codex-home")),
            parse_codex_home_flag(args.into_iter())
        );
    }

    #[test]
    fn ignores_missing_codex_home_flag() {
        let args = vec![OsString::from("codex"), OsString::from("--help")];
        assert_eq!(None, parse_codex_home_flag(args.into_iter()));
    }
}
