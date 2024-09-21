use std::path::Path;

use clap::ArgMatches;
use serde::{Deserialize, Serialize};

use crate::constants::get_default_java_binary;
use crate::constants::get_default_plantuml_jar;
use crate::constants::get_default_plantuml_version;
use crate::constants::get_default_source_directory;
use crate::constants::{get_default_cache_directory, get_default_source_patterns};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The path to the output directory.
    #[serde(default = "get_default_source_directory")]
    pub source_directory: String,
    /// The patterns to discover the source files separated by comas.
    #[serde(default = "get_default_source_patterns")]
    pub source_patterns: String,
    /// The path to the cache directory.
    #[serde(default = "get_default_cache_directory")]
    pub cache_directory: String,
    /// The PlantUML version.
    #[serde(default = "get_default_plantuml_version")]
    pub plantuml_version: String,
    /// The path to the PlantUML jar.
    #[serde(default = "get_default_plantuml_jar")]
    pub plantuml_jar: String,
    /// The path to the java binary.
    #[serde(default = "get_default_java_binary")]
    pub java_binary: String,
}

impl Config {
    pub fn update_from_args(&self, args: &ArgMatches) -> Config {
        let source_directory = args
            .get_one::<String>("source_directory")
            .map(|v| v.to_string())
            .unwrap_or_else(|| self.source_directory.clone());

        let source_patterns = args
            .get_one::<String>("source_patterns")
            .map(|v| v.to_string())
            .unwrap_or_else(|| self.source_patterns.clone());

        let cache_directory = args
            .get_one::<String>("cache_directory")
            .map(|v| v.to_string())
            .unwrap_or_else(|| self.cache_directory.clone());

        let plantuml_version = match args
            .get_one::<String>("plantuml_version")
            .map(|v| v.to_string())
        {
            None => self.plantuml_version.clone(),
            Some(v) => v,
        };

        let plantuml_jar = match args.get_one::<String>("plantuml_jar") {
            None => match Path::new(&cache_directory)
                .join(format!("plantuml-{}.jar", plantuml_version))
                .as_path()
                .to_str()
            {
                None => self.plantuml_jar.clone(),
                Some(v) => String::from(v),
            },
            Some(plantuml_jar) => plantuml_jar.to_string(),
        };

        Config {
            source_directory,
            source_patterns,
            cache_directory,
            plantuml_version,
            plantuml_jar,
            java_binary: args
                .get_one::<String>("java_binary")
                .map(|v| v.to_string())
                .unwrap_or_else(|| self.java_binary.clone()),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            source_directory: std::env::var("PLANTUML_GENERATOR_SOURCE_DIRECTORY")
                .unwrap_or_else(|_| get_default_source_directory()),
            source_patterns: std::env::var("PLANTUML_GENERATOR_SOURCE_PATTERNS")
                .unwrap_or_else(|_| get_default_source_patterns()),
            cache_directory: std::env::var("PLANTUML_GENERATOR_CACHE_DIRECTORY")
                .unwrap_or_else(|_| get_default_cache_directory()),
            plantuml_version: std::env::var("PLANTUML_GENERATOR_PLANTUML_VERSION")
                .unwrap_or_else(|_| get_default_plantuml_version()),
            plantuml_jar: std::env::var("PLANTUML_GENERATOR_PLANTUML_JAR")
                .unwrap_or_else(|_| get_default_plantuml_jar()),
            java_binary: std::env::var("PLANTUML_GENERATOR_JAVA_BINARY").unwrap_or_else(|_| {
                match std::env::var("JAVA_HOME") {
                    Ok(v) => format!("{}/bin/java", v),
                    Err(_) => get_default_java_binary(),
                }
            }),
        }
    }
}
