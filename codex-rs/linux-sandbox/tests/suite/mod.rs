// Aggregates all former standalone integration tests as modules.
use std::path::PathBuf;

mod landlock;
mod managed_proxy;

fn codex_linux_sandbox_exe() -> PathBuf {
    for key in [
        "CARGO_BIN_EXE_codex-linux-sandbox",
        "CARGO_BIN_EXE_codex_linux_sandbox",
    ] {
        if let Some(value) = std::env::var_os(key) {
            let path = PathBuf::from(value);
            if path.is_absolute() && path.exists() {
                return path;
            }
        }
    }

    if let Ok(current_exe) = std::env::current_exe()
        && let Some(deps_dir) = current_exe.parent()
    {
        if let Some(debug_dir) = deps_dir.parent() {
            let candidate = debug_dir.join("codex-linux-sandbox");
            if candidate.exists() {
                return candidate;
            }
        }

        let candidate = deps_dir.join("codex-linux-sandbox");
        if candidate.exists() {
            return candidate;
        }
    }

    panic!("unable to locate codex-linux-sandbox test binary");
}
