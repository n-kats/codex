#![allow(non_snake_case)]

use super::*;
use crate::agents_md_manager::AgentsMdManager;
use crate::agents_md_manager::SessionInstructions;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn custom__agents_md__project_doc_override_reloads_cached_instructions() {
    let tmp = tempfile::tempdir().expect("tempdir");
    fs::write(tmp.path().join("AGENTS.md"), "automatic instructions").unwrap();
    let custom_path = tmp.path().join("CUSTOM.md");
    fs::write(&custom_path, "custom instructions").unwrap();

    let mut cfg = make_config(&tmp, /*limit*/ 4096, /*instructions*/ None).await;
    let environments = resolved_local_environments([("local", cfg.config.cwd.clone())]);
    let manager = AgentsMdManager::new(SessionInstructions::default());

    let (result, warnings) = manager.refresh(&cfg.config, &environments).await;
    assert!(warnings.is_empty());
    result
        .expect("automatic instructions should load")
        .expect("automatic instructions");
    assert_eq!(
        manager
            .get_loaded()
            .await
            .expect("automatic instructions")
            .text(),
        "automatic instructions"
    );

    cfg.config.project_doc_paths = vec![custom_path.abs()];
    let (result, warnings) = manager.refresh(&cfg.config, &environments).await;
    assert!(warnings.is_empty());
    result
        .expect("custom instructions should load")
        .expect("custom instructions");
    assert_eq!(
        manager
            .get_loaded()
            .await
            .expect("custom instructions")
            .text(),
        "custom instructions"
    );
}
