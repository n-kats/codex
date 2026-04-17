#![allow(non_snake_case)]

use codex_core::config::ConfigBuilder;
use codex_core::config::ConfigOverrides;
use codex_core::config_loader::CloudRequirementsLoader;
use codex_core::config_loader::LoaderOverrides;
use serial_test::serial;
use std::ffi::OsString;

struct EnvVarGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &std::path::Path) -> Self {
        let original = std::env::var_os(key);
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, original }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.original {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }
}

#[tokio::test]
#[serial]
async fn custom__config_toml_read_control__loader_overrides_user_config_path_is_honored_by_tui()
-> std::io::Result<()> {
    let codex_home = tempfile::TempDir::new()?;
    let _guard = EnvVarGuard::set("CODEX_HOME", codex_home.path());
    let cwd = tempfile::TempDir::new()?;

    std::fs::write(
        codex_home.path().join("config.toml"),
        r#"
[shell_environment_policy]
inherit = "core"

[custom.user_shell]
no_inject = false
"#,
    )?;

    let alt_config = codex_home.path().join("alt.toml");
    std::fs::write(
        &alt_config,
        r#"
[shell_environment_policy]
inherit = "core"

[custom.user_shell]
no_inject = true

[custom.exec]
worker_uid = 1234
worker_gid = 5678
"#,
    )?;

    let loader_overrides = LoaderOverrides {
        user_config_path: Some(alt_config),
        disable_project_config: true,
        ..LoaderOverrides::default()
    };
    let overrides = ConfigOverrides {
        cwd: Some(cwd.path().to_path_buf()),
        ..Default::default()
    };
    let config = ConfigBuilder::default()
        .cli_overrides(Vec::new())
        .harness_overrides(overrides)
        .loader_overrides(loader_overrides)
        .cloud_requirements(CloudRequirementsLoader::default())
        .build()
        .await?;

    assert!(config.user_shell_no_inject);
    let run_as = config
        .exec_run_as
        .expect("expected custom.exec.* to resolve");
    assert_eq!(run_as.uid, 1234);
    assert_eq!(run_as.gid, 5678);
    Ok(())
}

#[tokio::test]
#[serial]
async fn custom__config_toml_read_control__disable_user_config_ignores_user_layer_for_tui()
-> std::io::Result<()> {
    let codex_home = tempfile::TempDir::new()?;
    let _guard = EnvVarGuard::set("CODEX_HOME", codex_home.path());
    let cwd = tempfile::TempDir::new()?;

    let alt_config = codex_home.path().join("alt.toml");
    std::fs::write(
        &alt_config,
        r#"
[shell_environment_policy]
inherit = "core"

[custom.user_shell]
no_inject = true

[custom.exec]
worker_uid = 1234
worker_gid = 5678
"#,
    )?;

    let loader_overrides = LoaderOverrides {
        user_config_path: Some(alt_config),
        disable_user_config: true,
        disable_project_config: true,
        ..LoaderOverrides::default()
    };
    let overrides = ConfigOverrides {
        cwd: Some(cwd.path().to_path_buf()),
        ..Default::default()
    };
    let config = ConfigBuilder::default()
        .cli_overrides(Vec::new())
        .harness_overrides(overrides)
        .loader_overrides(loader_overrides)
        .cloud_requirements(CloudRequirementsLoader::default())
        .build()
        .await?;

    assert!(!config.user_shell_no_inject);
    assert!(config.exec_run_as.is_none());
    Ok(())
}

#[tokio::test]
#[serial]
async fn custom__config_toml_read_control__load_config_or_exit_with_loader_overrides_honors_user_config_path()
-> std::io::Result<()> {
    let codex_home = tempfile::TempDir::new()?;
    let _guard = EnvVarGuard::set("CODEX_HOME", codex_home.path());
    let cwd = tempfile::TempDir::new()?;

    std::fs::write(
        codex_home.path().join("config.toml"),
        r#"
[shell_environment_policy]
inherit = "core"

[custom.user_shell]
no_inject = false
"#,
    )?;

    let override_config = codex_home.path().join("override.toml");
    std::fs::write(
        &override_config,
        r#"
[shell_environment_policy]
inherit = "core"

[custom.user_shell]
no_inject = true
"#,
    )?;

    let loader_overrides = LoaderOverrides {
        user_config_path: Some(override_config),
        disable_project_config: true,
        ..LoaderOverrides::default()
    };
    let overrides = ConfigOverrides {
        cwd: Some(cwd.path().to_path_buf()),
        ..Default::default()
    };
    let config = crate::load_config_or_exit_with_loader_overrides(
        Vec::new(),
        overrides,
        loader_overrides,
        CloudRequirementsLoader::default(),
        None,
    )
    .await;

    assert!(config.user_shell_no_inject);
    Ok(())
}
