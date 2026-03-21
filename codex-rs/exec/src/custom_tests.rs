#![allow(non_snake_case)]

use super::*;
use pretty_assertions::assert_eq;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn write_config(path: &Path, model: &str, approval_policy: &str, compact_prompt: &str) {
    let config_dir = path.join(".codex");
    fs::create_dir_all(&config_dir).expect("create .codex dir");
    fs::write(
        config_dir.join("config.toml"),
        format!(
            "model = \"{model}\"\napproval_policy = \"{approval_policy}\"\ncompact_prompt = \"{compact_prompt}\"\n"
        ),
    )
    .expect("write config.toml");
}

fn mark_as_git_repo(path: &Path) {
    fs::create_dir_all(path.join(".git")).expect("create git marker");
}

fn write_project_trust(codex_home: &Path, trusted_projects: &[&Path]) {
    let mut contents = String::new();
    for trusted_project in trusted_projects {
        let trusted_project = trusted_project.to_string_lossy().replace('\\', "\\\\");
        contents.push_str(&format!(
            r#"[projects."{trusted_project}"]
trust_level = "trusted"
"#
        ));
    }
    fs::write(codex_home.join("config.toml"), contents).expect("write trust config");
}

fn write_session_meta_rollout(path: &Path, session_id: &str, cwd: &Path) {
    use codex_protocol::protocol::RolloutItem;
    use codex_protocol::protocol::RolloutLine;
    use codex_protocol::protocol::SessionMeta;
    use codex_protocol::protocol::SessionMetaLine;
    use codex_protocol::protocol::SessionSource;

    let session_meta = SessionMetaLine {
        meta: SessionMeta {
            id: codex_protocol::ThreadId::from_string(session_id).expect("session id"),
            forked_from_id: None,
            timestamp: "2025-03-20T00:00:00Z".to_string(),
            cwd: cwd.to_path_buf(),
            originator: "codex-exec".to_string(),
            cli_version: "0.0.0".to_string(),
            source: SessionSource::Cli,
            agent_nickname: None,
            agent_role: None,
            model_provider: Some("openai".to_string()),
            base_instructions: None,
            dynamic_tools: None,
            memory_mode: None,
        },
        git: None,
    };
    let rollout = RolloutLine {
        timestamp: "2025-03-20T00:00:00Z".to_string(),
        item: RolloutItem::SessionMeta(session_meta),
    };
    fs::write(
        path,
        format!(
            "{}\n",
            serde_json::to_string(&rollout).expect("serialize session meta")
        ),
    )
    .expect("write rollout");
}

#[tokio::test]
async fn custom__resume_config__session_cwdの設定を読みつつruntime_cwdは維持する() {
    let codex_home = tempdir().expect("codex home");
    let session_cwd = tempdir().expect("session cwd");
    let current_cwd = tempdir().expect("current cwd");

    mark_as_git_repo(session_cwd.path());
    mark_as_git_repo(current_cwd.path());
    write_project_trust(codex_home.path(), &[session_cwd.path(), current_cwd.path()]);
    write_config(
        session_cwd.path(),
        "gpt-5.1",
        "never",
        "session resume compact prompt",
    );
    write_config(
        current_cwd.path(),
        "gpt-5.2-codex",
        "on-request",
        "current resume compact prompt",
    );

    let rollout_path = session_cwd.path().join("resume.jsonl");
    let session_id = "00000000-0000-0000-0000-000000000001";
    write_session_meta_rollout(&rollout_path, session_id, session_cwd.path());

    let current_config = ConfigBuilder::default()
        .codex_home(codex_home.path().to_path_buf())
        .fallback_cwd(Some(current_cwd.path().to_path_buf()))
        .build()
        .await
        .expect("build current config");
    assert_eq!(
        current_config.compact_prompt.as_deref(),
        Some("current resume compact prompt")
    );

    let cli_overrides: Vec<(String, toml::Value)> = Vec::new();
    let loader_overrides = LoaderOverrides::default();
    let cloud_requirements = CloudRequirementsLoader::default();
    let resumed = resume_config_for_path(
        &current_config,
        &ConfigOverrides::default(),
        &cli_overrides,
        &loader_overrides,
        &cloud_requirements,
        rollout_path.as_path(),
    )
    .await
    .expect("resume config");

    assert_eq!(resumed.cwd, current_cwd.path().to_path_buf());
    assert_eq!(resumed.model.as_deref(), Some("gpt-5.1"));
    assert_eq!(
        resumed.compact_prompt.as_deref(),
        Some("session resume compact prompt")
    );
    assert_ne!(
        resumed.compact_prompt, current_config.compact_prompt,
        "resume should rebuild compact prompt from the session cwd"
    );
    assert_eq!(
        resumed.permissions.approval_policy.value(),
        AskForApproval::Never
    );
    assert_eq!(resumed.codex_home, current_config.codex_home);
}
