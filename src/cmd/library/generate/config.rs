use std::path::Path;

use clap::ArgMatches;
use serde::{Deserialize, Serialize};

use crate::constants::get_default_cache_directory;
use crate::constants::get_default_inkscape_binary;
use crate::constants::get_default_java_binary;
use crate::constants::get_default_output_directory;
use crate::constants::get_default_plantuml_jar;
use crate::constants::get_default_plantuml_version;
use crate::constants::get_default_tera_discovery_pattern;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The path to the output directory.
    #[serde(default = "get_default_output_directory")]
    pub output_directory: String,
    /// The path to the cache directory.
    #[serde(default = "get_default_cache_directory")]
    pub cache_directory: String,
    /// The path to the primary Tera directory.
    #[serde(default = "get_default_tera_discovery_pattern")]
    pub tera_discovery_pattern: String,
    /// The PlantUML version.
    #[serde(default = "get_default_plantuml_version")]
    pub plantuml_version: String,
    /// The path to the PlantUML jar.
    #[serde(default = "get_default_plantuml_jar")]
    pub plantuml_jar: String,
    /// The path to the java binary.
    #[serde(default = "get_default_java_binary")]
    pub java_binary: String,
    /// The inkscape to the java binary.
    #[serde(default = "get_default_inkscape_binary")]
    pub inkscape_binary: String,
}

#[cfg(test)]
impl Config {
    pub fn rebase_directories(&self, root_directory: String) -> Config {
        Config {
            output_directory: match Path::new(&root_directory)
                .join(&self.output_directory)
                .as_path()
                .to_str()
            {
                None => self.output_directory.clone(),
                Some(v) => String::from(v),
            },
            cache_directory: match Path::new(&root_directory)
                .join(&self.cache_directory)
                .as_path()
                .to_str()
            {
                None => self.cache_directory.clone(),
                Some(v) => String::from(v),
            },
            tera_discovery_pattern: self.tera_discovery_pattern.clone(),
            plantuml_version: self.plantuml_version.clone(),
            plantuml_jar: self.plantuml_jar.clone(),
            java_binary: self.java_binary.clone(),
            inkscape_binary: self.inkscape_binary.clone(),
        }
    }
    pub fn update_plantuml_jar(&self, plantuml_jar: String) -> Config {
        Config {
            output_directory: self.output_directory.clone(),
            cache_directory: self.cache_directory.clone(),
            tera_discovery_pattern: self.tera_discovery_pattern.clone(),
            plantuml_version: self.plantuml_version.clone(),
            plantuml_jar: plantuml_jar.clone(),
            java_binary: self.java_binary.clone(),
            inkscape_binary: self.inkscape_binary.clone(),
        }
    }
}

impl Config {
    pub fn update_from_args(&self, args: &ArgMatches) -> Config {
        let cache_directory = args
            .value_of("cache_directory")
            .map(|v| v.to_string())
            .unwrap_or_else(|| self.cache_directory.clone());

        let plantuml_version = match args.value_of("plantuml_version").map(|v| v.to_string()) {
            None => self.plantuml_version.clone(),
            Some(v) => v,
        };

        let plantuml_jar = match args.value_of("plantuml_jar") {
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
            output_directory: args
                .value_of("output_directory")
                .map(|v| v.to_string())
                .unwrap_or_else(|| self.output_directory.clone()),
            cache_directory,
            tera_discovery_pattern: self.tera_discovery_pattern.clone(),
            plantuml_version,
            plantuml_jar,
            java_binary: args
                .value_of("java_binary")
                .map(|v| v.to_string())
                .unwrap_or_else(|| self.java_binary.clone()),
            inkscape_binary: args
                .value_of("inkscape_binary")
                .map(|v| v.to_string())
                .unwrap_or_else(|| self.inkscape_binary.clone()),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            output_directory: std::env::var("PLANTUML_GENERATOR_OUTPUT_DIRECTORY")
                .unwrap_or_else(|_| get_default_output_directory()),
            cache_directory: std::env::var("PLANTUML_GENERATOR_CACHE_DIRECTORY")
                .unwrap_or_else(|_| get_default_cache_directory()),
            tera_discovery_pattern: std::env::var("PLANTUML_GENERATOR_DISCOVERY_PATTERN")
                .unwrap_or_else(|_| get_default_tera_discovery_pattern()),
            plantuml_version: std::env::var("PLANTUML_GENERATOR_PLANTUML_VERSION")
                .unwrap_or_else(|_| get_default_plantuml_version()),
            plantuml_jar: std::env::var("PLANTUML_GENERATOR_PLANTUML_JAR")
                .unwrap_or_else(|_| get_default_plantuml_jar()),
            java_binary: match std::env::var("PLANTUML_GENERATOR_JAVA_BINARY") {
                Ok(v) => v,
                Err(_) => match std::env::var("JAVA_HOME") {
                    Ok(v) => format!("{}/bin/java", v),
                    Err(_) => get_default_java_binary(),
                },
            },
            inkscape_binary: std::env::var("PLANTUML_GENERATOR_INKSCAPE_BINARY")
                .unwrap_or_else(|_| get_default_inkscape_binary()),
        }
    }
}
