use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum Artifact {
    #[serde(rename = "github.com/tmorin/plantuml-libs")]
    Builtin {
        /// The version.
        #[serde(default)]
        version: String,
    },
}
