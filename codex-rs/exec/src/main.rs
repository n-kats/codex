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
use codex_arg0::arg0_dispatch_or_else;
use codex_exec::Cli;
use codex_exec::run_main;

#[derive(Parser, Debug, Default)]
struct ExecCliConfigOverrides {
    /// Override a configuration value that would otherwise be loaded from
    /// `~/.codex/config.toml`. Use a dotted path (`foo.bar.baz`) to override
    /// nested values. The `value` portion is parsed as TOML. If it fails to
    /// parse as TOML, the raw string is used as a literal.
    ///
    /// Examples:
    ///   - `--config model="o3"`
    ///   - `--config 'sandbox_permissions=["disk-full-read-access"]'`
    ///   - `--config shell_environment_policy.inherit=all`
    #[arg(
        short = 'c',
        long = "config",
        value_name = "key=value",
        action = clap::ArgAction::Append,
        global = true,
    )]
    raw_overrides: Vec<String>,
}

#[derive(Parser, Debug)]
struct TopCli {
    #[clap(flatten)]
    config_overrides: ExecCliConfigOverrides,

    #[clap(flatten)]
    inner: Cli,
}

fn main() -> anyhow::Result<()> {
    arg0_dispatch_or_else(|paths| async move {
        let top_cli = TopCli::parse();
        // Merge root-level overrides into inner CLI struct so downstream logic remains unchanged.
        let mut inner = top_cli.inner;
        inner
            .config_overrides
            .raw_overrides
            .splice(0..0, top_cli.config_overrides.raw_overrides);

        run_main(inner, paths).await?;
        Ok(())
    })
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;
