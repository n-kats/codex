//! Read-path helpers for Codex memories.
//!
//! This crate owns memory injection, memory citation parsing, and telemetry
//! classification for read access to the memory folder. It intentionally does
//! not depend on the memory write pipeline.

pub mod citations;
mod metrics;
pub mod usage;

use codex_utils_absolute_path::AbsolutePathBuf;

pub fn memory_root(codex_home: &AbsolutePathBuf) -> AbsolutePathBuf {
    let Some(memory_home) =
        std::env::var_os("CODEX_MEMORIES_HOME").filter(|memory_home| !memory_home.is_empty())
    else {
        return codex_home.join("memories");
    };
    AbsolutePathBuf::relative_to_current_dir(std::path::PathBuf::from(memory_home))
        .unwrap_or_else(|_| codex_home.join("memories"))
}
