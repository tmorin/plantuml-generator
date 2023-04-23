use clap::ArgMatches;
use serde::{Deserialize, Serialize};

use crate::constants::get_default_source_directory;
use crate::constants::get_default_workspace_manifest;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The path to the workspace manifest.
    #[serde(default = "get_default_workspace_manifest")]
    pub workspace_manifest: String,
    /// The path to the source directory.
    #[serde(default = "get_default_source_directory")]
    pub source_directory: String,
}

impl Config {
    pub fn update_from_args(&self, args: &ArgMatches) -> Config {
        let workspace_manifest = args
            .get_one::<String>("workspace_manifest")
            .map(|v| v.to_string())
            .unwrap_or_else(|| self.workspace_manifest.clone());

        let source_directory = args
            .get_one::<String>("source_directory")
            .map(|v| v.to_string())
            .unwrap_or_else(|| self.source_directory.clone());

        Config {
            workspace_manifest,
            source_directory,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            workspace_manifest: std::env::var("PLANTUML_GENERATOR_WORKSPACE_MANIFEST")
                .unwrap_or_else(|_| get_default_workspace_manifest()),
            source_directory: std::env::var("PLANTUML_GENERATOR_SOURCE_DIRECTORY")
                .unwrap_or_else(|_| get_default_source_directory()),
        }
    }
}
