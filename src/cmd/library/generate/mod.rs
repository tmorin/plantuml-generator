use std::fs::read_to_string;
use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use clap::ArgMatches;

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::generator::Generator;
use crate::cmd::library::generate::task::CleanupScope;
use crate::cmd::library::generate::templates::TEMPLATES;
use crate::cmd::library::manifest::library::Library;
use crate::plantuml::create_plantuml;
use crate::tera::create_tera;
use crate::urn::Urn;
use crate::utils::delete_file_or_directory;

mod config;
mod generator;
mod task;
mod tasks;
mod templates;

pub fn execute_library_generate(arg_matches: &ArgMatches) -> Result<()> {
    // create the config
    let config = &Config::default().update_from_args(arg_matches);
    if log::log_enabled!(log::Level::Info) {
        log::info!("output_directory: {}", &config.output_directory);
        log::info!("cache_directory: {}", &config.cache_directory);
        log::info!("tera_discovery_pattern: {}", &config.tera_discovery_pattern);
        log::info!("plantuml_jar: {}", &config.plantuml_jar);
        log::info!("java_binary: {}", &config.java_binary);
        log::info!("inkscape_binary: {}", &config.inkscape_binary);
    }

    // clean the cache directory
    if arg_matches.contains_id("do_clean_cache") {
        let path_to_delete = Path::new(&config.cache_directory);
        log::info!("clean the cache directory: {}", path_to_delete.display());
        delete_file_or_directory(path_to_delete)?
    }

    // clean the targeted output directories
    for urn_as_string in arg_matches
        .get_many::<String>("urns_to_clean")
        .unwrap_or_default()
    {
        let path_to_delete = Path::new(&config.output_directory).join(urn_as_string);
        log::info!(
            "clean the output sub-directory: {}",
            path_to_delete.display()
        );
        delete_file_or_directory(&path_to_delete)?
    }

    // resolve the manifest path
    let manifest_file = arg_matches
        .get_one::<String>("MANIFEST")
        .ok_or_else(|| anyhow::Error::msg("MANIFEST is required".to_string()))?;

    // create the YAML parser
    let yaml = &read_to_string(Path::new(manifest_file))
        .map_err(|e| anyhow::Error::new(e).context(format!("unable to read {}", manifest_file)))?;

    // parse the manifest
    let library: &Library = &serde_yaml::from_str(yaml)
        .map_err(|e| anyhow::Error::new(e).context(format!("unable to parse {}", manifest_file)))?;

    // create side utilities
    let tera = &create_tera(TEMPLATES.to_vec(), library.tera_discovery_pattern.clone())?;
    let plantuml = &create_plantuml(
        &config.java_binary,
        &config.plantuml_jar,
        &config.plantuml_version,
    )?;
    plantuml.download()?;

    let cleanup_scopes: &Vec<CleanupScope> = &arg_matches
        .get_many::<String>("cleanup_scopes")
        .unwrap_or_default()
        .map(|v| CleanupScope::from_str(v))
        .map(|r| r.unwrap())
        .collect();

    // fetch the targeted URNs
    let urns: &Vec<Urn> = &arg_matches
        .get_many::<String>("urns")
        .unwrap_or_default()
        .map(|c| Urn::from(c.as_str()))
        .collect();
    log::info!(
        "targeted urns: {}",
        urns.iter().map(|u| u.value.clone()).collect::<String>()
    );

    // generate the artifacts
    Generator::create(config, library, urns)?.generate(cleanup_scopes, tera, plantuml)?;

    log::info!("the generation is over");

    Ok(())
}

#[cfg(test)]
mod test {
    use std::fs::create_dir_all;

    use crate::cli::build_cli;

    use super::*;

    #[test]
    fn test_urns() {
        delete_file_or_directory("target/tests/cmd/library/generate/urns/distribution".as_ref())
            .unwrap();
        let arg_matches = build_cli().get_matches_from([
            "plantuml-generator",
            "-l=Off",
            "library",
            "generate",
            "test/library-simple.yaml",
            "-u=c4model",
            "-O=target/tests/cmd/library/generate/urns/distribution",
            "-C=target/tests/cmd/library/generate/urns/cache",
            "-P=test/plantuml-1.2022.4.jar",
        ]);
        execute_library_generate(
            arg_matches
                .subcommand_matches("library")
                .unwrap()
                .subcommand_matches("generate")
                .unwrap(),
        )
            .unwrap();
        assert!(Path::new("target/tests/cmd/library/generate/urns/distribution/c4model").exists());
        assert!(
            !Path::new("target/tests/cmd/library/generate/urns/distribution/eventstorming")
                .exists()
        );
    }

    #[test]
    fn test_clean_cache() {
        let path_in_cache =
            Path::new("target/tests/cmd/library/generate/clean_cache/cache/a_package");
        create_dir_all(path_in_cache).unwrap();
        assert!(path_in_cache.exists());
        let arg_matches = build_cli().get_matches_from([
            "plantuml-generator",
            "-l=Off",
            "library",
            "generate",
            "test/library-empty.yaml",
            "--clean-cache",
            "-O=target/tests/cmd/library/generate/clean_cache/distribution",
            "-C=target/tests/cmd/library/generate/clean_cache/cache",
            "-P=test/plantuml-1.2022.4.jar",
        ]);
        execute_library_generate(
            arg_matches
                .subcommand_matches("library")
                .unwrap()
                .subcommand_matches("generate")
                .unwrap(),
        )
            .unwrap();
        assert!(!path_in_cache.exists());
    }

    #[test]
    fn test_clean_urns() {
        let path_in_output = Path::new(
            "target/tests/cmd/library/generate/clean_urns/distribution/a_package/a_module",
        );
        create_dir_all(path_in_output).unwrap();
        assert!(path_in_output.exists());
        let arg_matches = build_cli().get_matches_from([
            "plantuml-generator",
            "-l=Off",
            "library",
            "generate",
            "test/library-empty.yaml",
            "--clean-urn=a_package/a_module",
            "-O=target/tests/cmd/library/generate/clean_urns/distribution",
            "-C=target/tests/cmd/library/generate/clean_urns/cache",
            "-P=test/plantuml-1.2022.4.jar",
        ]);
        execute_library_generate(
            arg_matches
                .subcommand_matches("library")
                .unwrap()
                .subcommand_matches("generate")
                .unwrap(),
        )
            .unwrap();
        assert!(!path_in_output.exists());
        assert!(path_in_output.parent().unwrap().exists());
    }
}
