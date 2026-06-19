use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct CustomExecToml {
    pub worker_user: Option<String>,
    pub worker_uid: Option<u32>,
    pub worker_gid: Option<u32>,
}
