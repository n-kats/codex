#![allow(non_snake_case)]

use super::tests::TestFileSystem;
use crate::CONFIG_TOML_FILE;
use crate::ConfigLayerSource;
use crate::LoaderOverrides;
use crate::NoopThreadConfigLoader;
use crate::loader::load_config_layers_state;
use codex_utils_absolute_path::AbsolutePathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn custom__config_toml_read_control__ignore_project_config_skips_project_layers() {
    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().join("workspace");
    let dot_codex = cwd.join(".codex");

    std::fs::create_dir_all(tmp.path().join(".git")).expect("create git dir");
    std::fs::create_dir_all(&dot_codex).expect("create dot-codex dir");
    std::fs::write(
        dot_codex.join(CONFIG_TOML_FILE),
        r#"
model = "gpt-project"
"#,
    )
    .expect("write project config");
    std::fs::create_dir_all(&cwd).expect("create cwd dir");

    let mut overrides = LoaderOverrides::without_managed_config_for_tests();
    overrides.ignore_project_config = true;

    let stack = load_config_layers_state(
        &TestFileSystem,
        tmp.path(),
        Some(AbsolutePathBuf::from_absolute_path(&cwd).expect("cwd should be absolute")),
        &[],
        overrides,
        &NoopThreadConfigLoader,
    )
    .await
    .expect("config loading should succeed");

    assert!(
        !stack
            .all_layers_low_to_high()
            .any(|layer| matches!(layer.name, ConfigLayerSource::Project { .. })),
        "project layers should be skipped when ignore_project_config is set"
    );
}
