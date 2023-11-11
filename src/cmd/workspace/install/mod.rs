use std::fs::{read_to_string, File};
use std::path::Path;

use clap::ArgMatches;

use crate::cmd::workspace::install::config::Config;
use crate::cmd::workspace::manifest::artifact::Artifact;
use crate::cmd::workspace::manifest::workspace::Workspace;
use crate::utils::{create_directory, create_parent_directory, delete_file_or_directory};

mod config;

pub fn execute_workspace_install(arg_matches: &ArgMatches) -> anyhow::Result<()> {
    // resolve the config
    let config = &Config::default().update_from_args(arg_matches);
    let do_force_install = arg_matches.get_flag("do_force_install");
    if log::log_enabled!(log::Level::Info) {
        log::info!("source_directory: {}", &config.source_directory);
        log::info!("workspace_manifest: {}", &config.workspace_manifest);
    }
    let source_path = Path::new(config.source_directory.as_str());
    let manifest_path = source_path.join(config.workspace_manifest.as_str());

    // stop if manifest doesn't exists
    if !manifest_path.exists() {
        Err(anyhow::Error::msg(format!(
            "the manifest {} doesn't exists",
            manifest_path.to_str().unwrap(),
        )))?;
    }

    // create the YAML parser
    let yaml = &read_to_string(&manifest_path).map_err(|e| {
        anyhow::Error::new(e).context(format!(
            "unable to read {}",
            manifest_path.to_str().unwrap()
        ))
    })?;

    // parse the manifest
    let manifest: &Workspace = &serde_yaml::from_str(yaml).map_err(|e| {
        anyhow::Error::new(e).context(format!(
            "unable to parse {}",
            manifest_path.to_str().unwrap()
        ))
    })?;
    log::debug!("manifest {:?}", manifest);

    // process the artifact
    for artifact in &manifest.artifacts {
        log::debug!("process artifact {:?}", artifact);
        match artifact {
            Artifact::Builtin { version } => {
                // resolve the path
                let cache_path = Path::new(&manifest.cache_directory);
                let archive_cache_path = &cache_path.join("tmorin_plantuml-libs");
                let archive_path = &archive_cache_path.join(format!("archive-{}.zip", version));
                let artifact_path = &archive_cache_path.join(version);

                // cleanup if expected
                if do_force_install {
                    delete_file_or_directory(archive_path)?;
                    delete_file_or_directory(artifact_path)?;
                }

                // download the archive
                if !archive_path.exists() {
                    // create the cache folder
                    create_parent_directory(archive_path)?;
                    let url = format!(
                        "https://github.com/tmorin/plantuml-libs/releases/download/v{}/tmorin-plantuml-libs.zip",
                        version,
                    );
                    log::info!("download {}", url);
                    match reqwest::blocking::get(&url)
                        .map_err(anyhow::Error::new)
                        .and_then(|r| r.error_for_status().map_err(anyhow::Error::new))
                        .and_then(|mut r| {
                            File::create(archive_path)
                                .map_err(anyhow::Error::new)
                                .and_then(|mut archive_file| {
                                    r.copy_to(&mut archive_file).map_err(anyhow::Error::new)
                                })
                        }) {
                        Ok(_) => {
                            log::info!("download completed for {}", url)
                        }
                        Err(e) => {
                            log::warn!("{:?}", e)
                        }
                    }
                }

                // unzip the archive
                if archive_path.exists() && !artifact_path.exists() {
                    // create the destination folder
                    create_directory(artifact_path)?;
                    log::info!("unzip {:?} to {:?}", archive_path, artifact_path);
                    match File::open(archive_path)
                        .map_err(|e| {
                            anyhow::Error::new(e).context(format!(
                                "unable to open {}",
                                archive_path.to_str().unwrap()
                            ))
                        })
                        .and_then(|archive_file| {
                            zip_extract::extract(archive_file, artifact_path, false).map_err(|e| {
                                anyhow::Error::new(e).context(format!(
                                    "unable to unzip {}",
                                    artifact_path.to_str().unwrap()
                                ))
                            })
                        }) {
                        Ok(_) => {
                            log::info!("unzip completed for {:?}", archive_path)
                        }
                        Err(e) => {
                            log::warn!("{:?}", e)
                        }
                    };
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::fs::copy;

    use crate::cli::build_cli;
    use crate::constants::WORKSPACE_MANIFEST;
    use crate::utils::{create_parent_directory, delete_file_or_directory};

    use super::*;

    #[test]
    fn test_install() {
        let test_path = Path::new("target/tests/cmd/workspace/install");
        let manifest_path = &test_path.join("source").join(WORKSPACE_MANIFEST);

        delete_file_or_directory(test_path).unwrap();
        create_parent_directory(manifest_path).unwrap();
        copy(Path::new("test/workspace-simple.yaml"), manifest_path).unwrap();

        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(manifest_path)
            .expect("Couldn't open file");

        let arg_matches = build_cli().get_matches_from([
            "plantuml-generator",
            "-l=Debug",
            "workspace",
            "install",
            "-s=target/tests/cmd/workspace/install/source",
        ]);

        execute_workspace_install(
            arg_matches
                .subcommand_matches("workspace")
                .unwrap()
                .subcommand_matches("install")
                .unwrap(),
        )
        .unwrap();

        execute_workspace_install(
            arg_matches
                .subcommand_matches("workspace")
                .unwrap()
                .subcommand_matches("install")
                .unwrap(),
        )
        .unwrap();
    }
}
