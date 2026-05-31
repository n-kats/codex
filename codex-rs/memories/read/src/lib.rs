//! Read-path helpers for Codex memories.
//!
//! This crate owns memory injection, memory citation parsing, and telemetry
//! classification for read access to the memory folder. It intentionally does
//! not depend on the memory write pipeline.

pub mod citations;
mod metrics;
pub mod usage;

use codex_utils_absolute_path::AbsolutePathBuf;
use std::fs;

pub fn memory_root(codex_home: &AbsolutePathBuf) -> AbsolutePathBuf {
    codex_home.join("memories")
}

pub async fn build_memory_tool_developer_instructions(
    codex_home: &AbsolutePathBuf,
) -> Option<String> {
    let memory_summary_path = memory_root(codex_home).join("memory_summary.md");
    let memory_summary = fs::read_to_string(&memory_summary_path)
        .ok()?
        .trim()
        .to_string();
    if memory_summary.is_empty() {
        return None;
    }

    Some(format!(
        "Memory summary from {}:\n\n{}",
        memory_root(codex_home).display(),
        memory_summary
    ))
}
