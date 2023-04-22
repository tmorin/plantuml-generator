use std::path::Path;

use anyhow::Result;
use clap::ArgMatches;

use crate::cmd::workspace::init::config::Config;
use crate::cmd::workspace::manifest::workspace::Workspace;
use crate::utils::{create_directory, create_parent_directory};

mod config;

pub fn execute_workspace_init(arg_matches: &ArgMatches) -> Result<()> {
    // resolve the config
    let config = &Config::default().update_from_args(arg_matches);
    if log::log_enabled!(log::Level::Info) {
        log::info!("cache_directory: {}", &config.cache_directory);
        log::info!("source_directory: {}", &config.source_directory);
        log::info!("workspace_manifest: {}", &config.workspace_manifest);
    }
    let cache_path = Path::new(config.cache_directory.as_str());
    let source_path = Path::new(config.source_directory.as_str());
    let manifest_path = source_path.join(&config.workspace_manifest);
    // create cache directory
    create_directory(cache_path)?;
    // create source directory
    create_parent_directory(manifest_path.as_path())?;
    // create the Workspace manifest
    let manifest = Workspace {
        cache_directory: cache_path.to_str().unwrap().to_string(),
        artifacts: vec![],
    };
    // save the Workspace manifest
    let f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(manifest_path)
        .expect("Couldn't open file");
    serde_yaml::to_writer(f, &manifest).unwrap();
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::cli::build_cli;
    use crate::utils::delete_file_or_directory;

    use super::*;

    #[test]
    fn test_init() {
        delete_file_or_directory("target/tests/cmd/workspace/init".as_ref()).unwrap();
        let arg_matches = build_cli().get_matches_from([
            "plantuml-generator",
            "-l=Debug",
            "workspace",
            "init",
            "-s=target/tests/cmd/workspace/init/source",
            "-C=target/tests/cmd/workspace/init/cache",
        ]);
        execute_workspace_init(
            arg_matches
                .subcommand_matches("workspace")
                .unwrap()
                .subcommand_matches("init")
                .unwrap(),
        )
            .unwrap();
        let path_workspace_manifest =
            Path::new("target/tests/cmd/workspace/init/source/.pgen-workspace.yaml");
        assert!(path_workspace_manifest.exists());
        let path_cache_directory = Path::new("target/tests/cmd/workspace/init/cache");
        assert!(path_cache_directory.exists());
    }
}
