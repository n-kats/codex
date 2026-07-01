use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct CustomThemeToml {
    #[serde(default)]
    pub diff: Option<CustomThemeDiffToml>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct CustomThemeDiffToml {
    pub enabled: Option<bool>,
    pub line_bg: Option<bool>,
    pub gutter: Option<bool>,
    pub sign: Option<bool>,
    pub content: Option<bool>,
    pub add_line_bg: Option<String>,
    pub del_line_bg: Option<String>,
}
