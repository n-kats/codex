mod theme;
mod user_shell;

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

pub use theme::CustomThemeDiffToml;
pub use theme::CustomThemeToml;
pub use user_shell::CustomUserShellToml;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct CustomConfigToml {
    #[serde(default)]
    pub user_shell: CustomUserShellToml,

    #[serde(default)]
    pub theme: CustomThemeToml,
}
