use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::cmd::workspace::manifest::artifact::Artifact;
use crate::constants::get_default_cache_directory;

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Workspace {
    #[serde(default = "get_default_cache_directory")]
    pub cache_directory: String,
    #[serde(default)]
    pub artifacts: Vec<Artifact>,
}
