//! Entry-point for the `codex-exec` binary.
//!
//! When this CLI is invoked normally, it parses the standard `codex-exec` CLI
//! options and launches the non-interactive Codex agent. However, if it is
//! invoked with arg0 as `codex-linux-sandbox`, we instead treat the invocation
//! as a request to run the logic for the standalone `codex-linux-sandbox`
//! executable (i.e., parse any -s args and then run a *sandboxed* command under
//! Landlock + seccomp.
//!
//! This allows us to ship a completely separate set of functionality as part
//! of the `codex-exec` binary.
use clap::Parser;
use codex_arg0::Arg0DispatchPaths;
use codex_arg0::arg0_dispatch_or_else;
use codex_core::config_loader::LoaderOverrides;
use codex_exec::Cli;
use codex_exec::run_main;
use codex_utils_cli::CliConfigOverrides;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct TopCli {
    #[clap(flatten)]
    config_overrides: CliConfigOverrides,

    #[clap(
        long = "config",
        alias = "config-toml-file",
        value_name = "FILE",
        global = true,
        conflicts_with = "no_config"
    )]
    config_toml_file: Option<PathBuf>,

    #[clap(long = "no-config", global = true, default_value_t = false)]
    no_config: bool,

    #[clap(flatten)]
    inner: Cli,
}

fn main() -> anyhow::Result<()> {
    arg0_dispatch_or_else(|arg0_paths: Arg0DispatchPaths| async move {
        let top_cli = TopCli::parse();
        // Merge root-level overrides into inner CLI struct so downstream logic remains unchanged.
        let mut inner = top_cli.inner;
        inner
            .config_overrides
            .raw_overrides
            .splice(0..0, top_cli.config_overrides.raw_overrides);

        let loader_overrides = {
            let mut overrides = LoaderOverrides::default();
            if top_cli.no_config {
                overrides.ignore_user_config = true;
                overrides.ignore_user_and_project_exec_policy_rules = true;
            } else if let Some(path) = top_cli.config_toml_file {
                overrides.user_config_path = Some(resolve_path_from_cwd(path));
            }
            overrides
        };

        run_main(inner, arg0_paths, loader_overrides).await?;
        Ok(())
    })
}

fn resolve_path_from_cwd(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(path)
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "custom_tests.rs"]
mod custom_tests;
