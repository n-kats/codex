use crate::config::find_codex_home;
use crate::shell::ShellType;
use std::collections::HashMap;
use std::path::Path;

const CODEX_SHELL_STARTUP_FILES_ENV_VAR: &str = "CODEX_SHELL_STARTUP_FILES";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellStartupFiles {
    Default,
    Clean,
}

impl ShellStartupFiles {
    pub fn from_env() -> Self {
        parse_shell_startup_files(
            std::env::var(CODEX_SHELL_STARTUP_FILES_ENV_VAR)
                .ok()
                .as_deref(),
        )
    }
}

fn parse_shell_startup_files(raw: Option<&str>) -> ShellStartupFiles {
    let Some(raw) = raw else {
        return ShellStartupFiles::Default;
    };
    match raw.trim().to_ascii_lowercase().as_str() {
        "clean" => ShellStartupFiles::Clean,
        _ => ShellStartupFiles::Default,
    }
}

pub fn apply_shell_startup_files_env(env: &mut HashMap<String, String>, shell_type: ShellType) {
    let ShellStartupFiles::Clean = ShellStartupFiles::from_env() else {
        return;
    };
    let Ok(codex_home) = find_codex_home() else {
        return;
    };
    apply_shell_startup_files_env_with_home(env, shell_type, &codex_home);
}

fn apply_shell_startup_files_env_with_home(
    env: &mut HashMap<String, String>,
    shell_type: ShellType,
    codex_home: &Path,
) {
    if shell_type != ShellType::Zsh {
        return;
    }

    let zdotdir = codex_home.join("shell_dotfiles").join("empty_zdotdir");
    if let Err(err) = std::fs::create_dir_all(&zdotdir) {
        tracing::warn!(
            "failed to create ZDOTDIR isolation directory {}: {err}",
            zdotdir.display()
        );
        return;
    }

    env.insert("ZDOTDIR".to_string(), zdotdir.display().to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn parses_shell_startup_files_default() {
        assert_eq!(parse_shell_startup_files(None), ShellStartupFiles::Default);
        assert_eq!(
            parse_shell_startup_files(Some("default")),
            ShellStartupFiles::Default
        );
        assert_eq!(
            parse_shell_startup_files(Some("unknown")),
            ShellStartupFiles::Default
        );
    }

    #[test]
    fn parses_shell_startup_files_clean() {
        assert_eq!(
            parse_shell_startup_files(Some("clean")),
            ShellStartupFiles::Clean
        );
        assert_eq!(
            parse_shell_startup_files(Some(" CLEAN ")),
            ShellStartupFiles::Clean
        );
    }

    #[test]
    fn clean_isolates_zsh_only() {
        let tmp = tempdir().expect("create TempDir");
        let codex_home = tmp.path();

        let mut env = HashMap::new();
        apply_shell_startup_files_env_with_home(&mut env, ShellType::Bash, codex_home);
        assert_eq!(env.get("ZDOTDIR"), None);

        let mut env = HashMap::new();
        apply_shell_startup_files_env_with_home(&mut env, ShellType::Zsh, codex_home);
        let zdotdir = env.get("ZDOTDIR").expect("ZDOTDIR set");
        assert_eq!(
            PathBuf::from(zdotdir),
            codex_home.join("shell_dotfiles").join("empty_zdotdir")
        );
    }
}
