mod exec;
mod user_shell;

use crate::types::ShellEnvironmentPolicyToml;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

pub use exec::CustomExecToml;
pub use user_shell::CustomUserShellToml;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct CustomConfigToml {
    #[serde(default)]
    pub exec: CustomExecToml,

    #[serde(default)]
    pub user_shell: CustomUserShellToml,

    pub user_shell_environment_policy: Option<ShellEnvironmentPolicyToml>,

    pub assistant_shell_environment_policy: Option<ShellEnvironmentPolicyToml>,
}
