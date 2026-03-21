#![allow(non_snake_case)]

use codex_utils_cargo_bin::find_resource;
use core_test_support::test_codex_exec::test_codex_exec;
use serde_json::Value;
use tempfile::TempDir;
use uuid::Uuid;
use walkdir::WalkDir;

fn exec_fixture() -> anyhow::Result<std::path::PathBuf> {
    Ok(find_resource!("tests/fixtures/cli_responses_fixture.sse")?)
}

fn find_session_file_containing_marker(
    sessions_dir: &std::path::Path,
    marker: &str,
) -> Option<std::path::PathBuf> {
    for entry in WalkDir::new(sessions_dir) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        if !entry.file_name().to_string_lossy().ends_with(".jsonl") {
            continue;
        }
        let path = entry.path();
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let mut lines = content.lines();
        if lines.next().is_none() {
            continue;
        }
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            let Ok(item): Result<Value, _> = serde_json::from_str(line) else {
                continue;
            };
            if item.get("type").and_then(|t| t.as_str()) == Some("response_item")
                && let Some(payload) = item.get("payload")
                && payload.get("type").and_then(|t| t.as_str()) == Some("message")
                && payload
                    .get("content")
                    .map(std::string::ToString::to_string)
                    .unwrap_or_default()
                    .contains(marker)
            {
                return Some(path.to_path_buf());
            }
        }
    }
    None
}

fn extract_conversation_id(path: &std::path::Path) -> String {
    let content = std::fs::read_to_string(path).unwrap();
    let mut lines = content.lines();
    let meta_line = lines.next().expect("missing meta line");
    let meta: Value = serde_json::from_str(meta_line).expect("invalid meta json");
    meta.get("payload")
        .and_then(|p| p.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

fn write_cwd_config(
    cwd: &std::path::Path,
    model: &str,
    approval_policy: &str,
    sandbox_mode: &str,
    reasoning_effort: &str,
    reasoning_summary: &str,
) -> anyhow::Result<()> {
    std::fs::create_dir_all(cwd.join(".codex"))?;
    std::fs::write(
        cwd.join(".codex").join("config.toml"),
        format!(
            "model = \"{model}\"\napproval_policy = \"{approval_policy}\"\nsandbox_mode = \"{sandbox_mode}\"\nmodel_reasoning_effort = \"{reasoning_effort}\"\nmodel_reasoning_summary = \"{reasoning_summary}\"\n",
        ),
    )?;
    Ok(())
}

fn write_project_trust(
    codex_home: &std::path::Path,
    trusted_projects: &[&std::path::Path],
) -> anyhow::Result<()> {
    let mut contents = String::new();
    for trusted_project in trusted_projects {
        let trusted_project = trusted_project.to_string_lossy().replace('\\', "\\\\");
        contents.push_str(&format!(
            r#"[projects."{trusted_project}"]
trust_level = "trusted"
"#
        ));
    }
    std::fs::write(codex_home.join("config.toml"), contents)?;
    Ok(())
}

#[test]
fn custom__resume__resume_by_id_はsession_cwdのconfigでsummaryを出す() -> anyhow::Result<()> {
    let test = test_codex_exec();
    let fixture = exec_fixture()?;

    let dir_a = TempDir::new()?;
    let dir_b = TempDir::new()?;

    write_project_trust(test.home_path(), &[dir_a.path(), dir_b.path()])?;

    std::fs::create_dir_all(dir_a.path().join(".git"))?;
    std::fs::create_dir_all(dir_b.path().join(".git"))?;
    write_cwd_config(
        dir_a.path(),
        "gpt-5.1",
        "never",
        "read-only",
        "high",
        "detailed",
    )?;
    write_cwd_config(
        dir_b.path(),
        "gpt-5.2-codex",
        "on-request",
        "workspace-write",
        "low",
        "concise",
    )?;

    let marker = format!("custom-resume-reload-{}", Uuid::new_v4());
    let prompt = format!("echo {marker}");

    test.cmd()
        .env("CODEX_RS_SSE_FIXTURE", &fixture)
        .env("OPENAI_BASE_URL", "http://unused.local")
        .arg("--skip-git-repo-check")
        .arg("-C")
        .arg(dir_a.path())
        .arg(&prompt)
        .assert()
        .success();

    let sessions_dir = test.home_path().join("sessions");
    let path = find_session_file_containing_marker(&sessions_dir, &marker)
        .expect("no session file found after first run");
    let session_id = extract_conversation_id(&path);
    assert!(
        !session_id.is_empty(),
        "missing conversation id in meta line"
    );

    let marker2 = format!("custom-resume-reload-2-{}", Uuid::new_v4());
    let prompt2 = format!("echo {marker2}");

    let output = test
        .cmd()
        .env("CODEX_RS_SSE_FIXTURE", &fixture)
        .env("OPENAI_BASE_URL", "http://unused.local")
        .arg("--skip-git-repo-check")
        .arg("-C")
        .arg(dir_b.path())
        .arg("resume")
        .arg(&session_id)
        .arg(&prompt2)
        .output()?;

    assert!(output.status.success(), "resume run failed: {output:?}");

    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        stderr.contains(&format!("workdir: {}", dir_b.path().display())),
        "stderr missing invocation cwd: {stderr}"
    );
    assert!(
        stderr.contains("model: gpt-5.1"),
        "stderr missing resumed session model from session cwd: {stderr}"
    );
    assert!(
        !stderr.contains("model: gpt-5.2-codex"),
        "stderr still reflects current cwd model instead of session cwd: {stderr}"
    );
    assert!(
        stderr.contains("sandbox: read-only"),
        "stderr missing resumed session sandbox from session cwd: {stderr}"
    );
    assert!(
        stderr.contains("reasoning effort: high"),
        "stderr missing resumed session reasoning effort from session cwd: {stderr}"
    );
    assert!(
        stderr.contains("reasoning summaries: detailed"),
        "stderr missing resumed session reasoning summary from session cwd: {stderr}"
    );

    Ok(())
}

#[test]
fn custom__start__startはcurrent_cwdのconfigでsummaryを出す() -> anyhow::Result<()> {
    let test = test_codex_exec();
    let fixture = exec_fixture()?;

    let dir_a = TempDir::new()?;
    let dir_b = TempDir::new()?;

    write_project_trust(test.home_path(), &[dir_a.path(), dir_b.path()])?;

    std::fs::create_dir_all(dir_a.path().join(".git"))?;
    std::fs::create_dir_all(dir_b.path().join(".git"))?;
    write_cwd_config(
        dir_a.path(),
        "gpt-5.1",
        "never",
        "read-only",
        "high",
        "detailed",
    )?;
    write_cwd_config(
        dir_b.path(),
        "gpt-5.2-codex",
        "on-request",
        "workspace-write",
        "low",
        "concise",
    )?;

    let marker = format!("custom-start-model-switch-{}", Uuid::new_v4());
    let prompt = format!("echo {marker}");

    let output = test
        .cmd()
        .env("CODEX_RS_SSE_FIXTURE", &fixture)
        .env("OPENAI_BASE_URL", "http://unused.local")
        .arg("--skip-git-repo-check")
        .arg("-C")
        .arg(dir_b.path())
        .arg(&prompt)
        .output()?;

    assert!(output.status.success(), "start run failed: {output:?}");

    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        stderr.contains(&format!("workdir: {}", dir_b.path().display())),
        "stderr missing current cwd workdir: {stderr}"
    );
    assert!(
        stderr.contains("model: gpt-5.2-codex"),
        "stderr missing current cwd model: {stderr}"
    );
    assert!(
        !stderr.contains("model: gpt-5.1"),
        "stderr unexpectedly shows session cwd model on start: {stderr}"
    );
    assert!(
        stderr.contains("sandbox: workspace-write"),
        "stderr missing current cwd sandbox: {stderr}"
    );
    assert!(
        stderr.contains("reasoning effort: low"),
        "stderr missing current cwd reasoning effort: {stderr}"
    );
    assert!(
        stderr.contains("reasoning summaries: concise"),
        "stderr missing current cwd reasoning summary: {stderr}"
    );

    Ok(())
}
