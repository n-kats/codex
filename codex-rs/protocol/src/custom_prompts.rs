use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
use ts_rs::TS;

pub const PROMPTS_CMD_PREFIX: &str = "prompts";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
pub struct CustomPrompt {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
    pub description: Option<String>,
    pub argument_hint: Option<String>,
}
