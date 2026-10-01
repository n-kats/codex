#![allow(non_snake_case)]

use super::*;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

#[test]
fn custom__agents_md_restore__resume_preserves_root_agents_md_and_config_flags() {
    let interactive = finalize_resume_from_args(
        [
            "codex",
            "--agents-md",
            "root/AGENTS.md",
            "--config-file",
            "root.toml",
            "resume",
            "sid",
        ]
        .as_ref(),
    );

    let shared = interactive.shared.clone().into_inner();
    assert_eq!(shared.agents_md, vec![PathBuf::from("root/AGENTS.md")]);
    assert_eq!(shared.config_toml_file, Some(PathBuf::from("root.toml")));
    assert_eq!(interactive.resume_session_id.as_deref(), Some("sid"));
}

#[test]
fn custom__agents_md_restore__resume_subcommand_overrides_root_flags() {
    let interactive = finalize_resume_from_args(
        [
            "codex",
            "--agents-md",
            "root/AGENTS.md",
            "--config-file",
            "root.toml",
            "resume",
            "sid",
            "--agents-md",
            "resume/AGENTS.md",
            "--config-file",
            "resume.toml",
        ]
        .as_ref(),
    );

    let shared = interactive.shared.clone().into_inner();
    assert_eq!(shared.agents_md, vec![PathBuf::from("resume/AGENTS.md")]);
    assert_eq!(shared.config_toml_file, Some(PathBuf::from("resume.toml")));
    assert_eq!(interactive.resume_session_id.as_deref(), Some("sid"));
}

#[test]
fn custom__agents_md_restore__fork_preserves_root_agents_md_and_config_flags() {
    let interactive = finalize_fork_from_args(
        [
            "codex",
            "--agents-md",
            "root/AGENTS.md",
            "--config-file",
            "root.toml",
            "fork",
            "sid",
        ]
        .as_ref(),
    );

    let shared = interactive.shared.clone().into_inner();
    assert_eq!(shared.agents_md, vec![PathBuf::from("root/AGENTS.md")]);
    assert_eq!(shared.config_toml_file, Some(PathBuf::from("root.toml")));
    assert_eq!(interactive.fork_session_id.as_deref(), Some("sid"));
}
