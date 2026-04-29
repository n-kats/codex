//! Memory root helpers used by core code.

#[allow(dead_code, unused_imports)]
mod control;

#[allow(unused_imports)]
pub use control::clear_memory_roots_contents;

use codex_utils_absolute_path::AbsolutePathBuf;
use std::path::PathBuf;

#[allow(dead_code)]
pub fn memory_root(codex_home: &AbsolutePathBuf) -> AbsolutePathBuf {
    if let Some(memory_root) = resolve_memory_root_env() {
        return memory_root;
    }
    codex_home.join("memories")
}

#[allow(dead_code)]
fn resolve_memory_root_env() -> Option<AbsolutePathBuf> {
    let raw = std::env::var_os("CODEX_MEMORIES_HOME")?;
    if raw.is_empty() {
        return None;
    }
    AbsolutePathBuf::relative_to_current_dir(PathBuf::from(raw)).ok()
}
