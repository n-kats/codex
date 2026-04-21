use super::discover_project_doc_paths;
use super::get_user_instructions;
use crate::config::ConfigBuilder;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

#[tokio::test]
async fn project_doc_paths_prefer_explicit_over_autodiscovery() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::write(root.path().join("AGENTS.md"), "auto").unwrap();
    std::fs::write(root.path().join("custom.md"), "custom").unwrap();

    let codex_home = TempDir::new().unwrap();
    let mut cfg = ConfigBuilder::default()
        .codex_home(codex_home.path().to_path_buf())
        .build()
        .await
        .expect("defaults for test should always succeed");
    cfg.cwd = root.path().to_path_buf();
    cfg.project_doc_max_bytes = 4096;
    cfg.project_doc_paths = vec![root.path().join("custom.md")];

    let discovery = discover_project_doc_paths(&cfg).expect("discover paths");
    assert_eq!(discovery, vec![root.path().join("custom.md")]);

    let res = get_user_instructions(&cfg)
        .await
        .expect("explicit doc expected");
    assert_eq!(res, "custom");
}

#[tokio::test]
async fn project_doc_paths_include_ancestor_agents_docs() {
    let root = tempfile::tempdir().expect("tempdir");
    let nested = root.path().join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(root.path().join("AGENTS.md"), "root").unwrap();
    std::fs::write(nested.join("AGENTS.md"), "nested").unwrap();

    let codex_home = TempDir::new().unwrap();
    let mut cfg = ConfigBuilder::default()
        .codex_home(codex_home.path().to_path_buf())
        .build()
        .await
        .expect("defaults for test should always succeed");
    cfg.cwd = nested;
    cfg.project_doc_max_bytes = 4096;

    let res = get_user_instructions(&cfg).await.expect("docs expected");
    assert_eq!(res, "root\n\nnested");
}
