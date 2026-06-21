//! Shared command-line flags used by both interactive and non-interactive Codex entry points.

use crate::SandboxModeCliArg;
use clap::Args;
use codex_protocol::config_types::ProfileV2Name;
use std::path::PathBuf;

#[derive(Args, Clone, Debug, Default)]
pub struct SharedCliOptions {
    /// Override the Codex home directory used for config, logs, and caches.
    #[arg(long = "codex-home", value_name = "DIR", global = true)]
    pub codex_home: Option<PathBuf>,

    /// Override the Codex memories directory used for writable memory roots.
    #[arg(long = "codex-memory", value_name = "DIR", global = true)]
    pub codex_memory: Option<PathBuf>,

    /// Read user config from this file instead of `$CODEX_HOME/config.toml`.
    #[arg(
        long = "config",
        alias = "config-toml-file",
        value_name = "FILE",
        global = true,
        conflicts_with = "no_config"
    )]
    pub config_toml_file: Option<PathBuf>,

    /// Skip user and project config layers for this invocation.
    #[arg(long = "no-config", global = true, default_value_t = false)]
    pub no_config: bool,

    /// Optional image(s) to attach to the initial prompt.
    #[arg(
        long = "image",
        short = 'i',
        value_name = "FILE",
        value_delimiter = ',',
        num_args = 1..
    )]
    pub images: Vec<PathBuf>,

    /// Model the agent should use.
    #[arg(long, short = 'm')]
    pub model: Option<String>,

    /// Use open-source provider.
    #[arg(long = "oss", default_value_t = false)]
    pub oss: bool,

    /// Specify which local provider to use (lmstudio or ollama).
    /// If not specified with --oss, will use config default or show selection.
    #[arg(long = "local-provider")]
    pub oss_provider: Option<String>,

    /// Layer $CODEX_HOME/<name>.config.toml on top of the base user config.
    #[arg(long = "profile", short = 'p')]
    pub config_profile_v2: Option<ProfileV2Name>,

    /// Select the sandbox policy to use when executing model-generated shell
    /// commands.
    #[arg(long = "sandbox", short = 's')]
    pub sandbox_mode: Option<SandboxModeCliArg>,

    /// Skip all confirmation prompts and execute commands without sandboxing.
    /// EXTREMELY DANGEROUS. Intended solely for running in environments that are externally sandboxed.
    #[arg(
        long = "dangerously-bypass-approvals-and-sandbox",
        alias = "yolo",
        default_value_t = false
    )]
    pub dangerously_bypass_approvals_and_sandbox: bool,

    /// Run enabled hooks without requiring persisted hook trust for this invocation.
    /// DANGEROUS. Intended only for automation that already vets hook sources.
    #[arg(long = "dangerously-bypass-hook-trust", default_value_t = false)]
    pub bypass_hook_trust: bool,

    /// Tell the agent to use the specified directory as its working root.
    #[clap(long = "cd", short = 'C', value_name = "DIR")]
    pub cwd: Option<PathBuf>,

    /// Additional directories that should be writable alongside the primary workspace.
    #[arg(long = "add-dir", value_name = "DIR", value_hint = clap::ValueHint::DirPath)]
    pub add_dir: Vec<PathBuf>,
}

impl SharedCliOptions {
    pub fn inherit_exec_root_options(&mut self, root: &Self) {
        let self_selected_sandbox_mode =
            self.sandbox_mode.is_some() || self.dangerously_bypass_approvals_and_sandbox;
        let Self {
            codex_home,
            codex_memory,
            config_toml_file,
            no_config,
            images,
            model,
            oss,
            oss_provider,
            config_profile_v2,
            sandbox_mode,
            dangerously_bypass_approvals_and_sandbox,
            bypass_hook_trust,
            cwd,
            add_dir,
        } = self;
        let Self {
            codex_home: root_codex_home,
            codex_memory: root_codex_memory,
            config_toml_file: root_config_toml_file,
            no_config: root_no_config,
            images: root_images,
            model: root_model,
            oss: root_oss,
            oss_provider: root_oss_provider,
            config_profile_v2: root_config_profile_v2,
            sandbox_mode: root_sandbox_mode,
            dangerously_bypass_approvals_and_sandbox: root_dangerously_bypass_approvals_and_sandbox,
            bypass_hook_trust: root_bypass_hook_trust,
            cwd: root_cwd,
            add_dir: root_add_dir,
        } = root;

        if codex_home.is_none() {
            codex_home.clone_from(root_codex_home);
        }
        if codex_memory.is_none() {
            codex_memory.clone_from(root_codex_memory);
        }
        if config_toml_file.is_none() {
            config_toml_file.clone_from(root_config_toml_file);
        }
        if !*no_config {
            *no_config = *root_no_config;
        }
        if model.is_none() {
            model.clone_from(root_model);
        }
        if *root_oss {
            *oss = true;
        }
        if oss_provider.is_none() {
            oss_provider.clone_from(root_oss_provider);
        }
        if config_profile_v2.is_none() {
            config_profile_v2.clone_from(root_config_profile_v2);
        }
        if sandbox_mode.is_none() {
            *sandbox_mode = *root_sandbox_mode;
        }
        if !self_selected_sandbox_mode {
            *dangerously_bypass_approvals_and_sandbox =
                *root_dangerously_bypass_approvals_and_sandbox;
        }
        if !*bypass_hook_trust {
            *bypass_hook_trust = *root_bypass_hook_trust;
        }
        if cwd.is_none() {
            cwd.clone_from(root_cwd);
        }
        if !root_images.is_empty() {
            let mut merged_images = root_images.clone();
            merged_images.append(images);
            *images = merged_images;
        }
        if !root_add_dir.is_empty() {
            let mut merged_add_dir = root_add_dir.clone();
            merged_add_dir.append(add_dir);
            *add_dir = merged_add_dir;
        }
    }

    pub fn apply_subcommand_overrides(&mut self, subcommand: Self) {
        let subcommand_selected_sandbox_mode = subcommand.sandbox_mode.is_some()
            || subcommand.dangerously_bypass_approvals_and_sandbox;
        let Self {
            codex_home,
            codex_memory,
            config_toml_file,
            no_config,
            images,
            model,
            oss,
            oss_provider,
            config_profile_v2,
            sandbox_mode,
            dangerously_bypass_approvals_and_sandbox,
            bypass_hook_trust,
            cwd,
            add_dir,
        } = subcommand;

        if let Some(codex_home) = codex_home {
            self.codex_home = Some(codex_home);
        }
        if let Some(codex_memory) = codex_memory {
            self.codex_memory = Some(codex_memory);
        }
        if let Some(config_toml_file) = config_toml_file {
            self.config_toml_file = Some(config_toml_file);
        }
        if no_config {
            self.no_config = true;
        }
        if let Some(model) = model {
            self.model = Some(model);
        }
        if oss {
            self.oss = true;
        }
        if let Some(oss_provider) = oss_provider {
            self.oss_provider = Some(oss_provider);
        }
        if let Some(config_profile_v2) = config_profile_v2 {
            self.config_profile_v2 = Some(config_profile_v2);
        }
        if subcommand_selected_sandbox_mode {
            self.sandbox_mode = sandbox_mode;
            self.dangerously_bypass_approvals_and_sandbox =
                dangerously_bypass_approvals_and_sandbox;
        }
        if bypass_hook_trust {
            self.bypass_hook_trust = true;
        }
        if let Some(cwd) = cwd {
            self.cwd = Some(cwd);
        }
        if !images.is_empty() {
            self.images = images;
        }
        if !add_dir.is_empty() {
            self.add_dir.extend(add_dir);
        }
    }
}
