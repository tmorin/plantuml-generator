use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

use anyhow::Result;
use chrono::prelude::*;
use clap::ArgMatches;
use glob::{glob, Paths};

use crate::cmd::diagram::generate::config::Config;
use crate::plantuml::create_plantuml;
use crate::utils::create_parent_directory;

mod config;

fn get_last_modified(path: &Path) -> Result<i64> {
    match path.exists() {
        true => {
            let modified = path
                .metadata()
                .map_err(|e| {
                    anyhow::Error::new(e).context(format!(
                        "unable to get metadata for {}",
                        path.to_str().unwrap()
                    ))
                })?
                .modified()
                .map_err(|e| {
                    anyhow::Error::new(e).context(format!(
                        "unable to get modified value for {}",
                        path.to_str().unwrap()
                    ))
                })?;
            let date_time: DateTime<Local> = DateTime::from(modified);
            Ok(date_time.timestamp_nanos_opt().unwrap())
        }
        false => Ok(0),
    }
}

fn get_last_generation_timestamp(last_gen_path: &Path) -> Result<i64> {
    match last_gen_path.exists() {
        true => {
            let timestamp_as_string = read_to_string(last_gen_path).map_err(|e| {
                anyhow::Error::new(e).context(format!("unable to read {:?}", last_gen_path))
            })?;
            match timestamp_as_string.is_empty() {
                true => Ok(0),
                false => Ok(timestamp_as_string.parse().unwrap_or_default()),
            }
        }
        false => Ok(0),
    }
}

fn save_last_generation_timestamp(last_gen_path: &Path) -> Result<()> {
    let now: DateTime<Local> = DateTime::from(SystemTime::now());
    let value = now.timestamp_nanos_opt().unwrap().to_string();
    log::debug!("save_last_generation_timestamp {}", value);
    let mut last_gen_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .append(false)
        .open(last_gen_path)
        .map_err(|e| {
            anyhow::Error::new(e).context(format!("unable to open {:?}", &last_gen_path))
        })?;
    last_gen_file.write_all(value.as_bytes()).map_err(|e| {
        anyhow::Error::new(e).context(format!("unable to write {:?}", last_gen_file))
    })?;
    Ok(())
}

fn get_puml_paths(config: &Config) -> Result<Paths> {
    let glob_pattern = format!("{}/**/*.puml", config.source_directory);
    glob(&glob_pattern).map_err(|e| {
        anyhow::Error::new(e).context(format!(
            "unable to parse the glob pattern ({})",
            &glob_pattern
        ))
    })
}

pub fn execute_diagram_generate(arg_matches: &ArgMatches) -> Result<()> {
    // resolve the config
    let config = &Config::default().update_from_args(arg_matches);
    let force_generation = arg_matches.get_flag("do_force_generation");
    if log::log_enabled!(log::Level::Info) {
        log::info!("source_directory: {}", &config.source_directory);
        log::info!("cache_directory: {}", &config.cache_directory);
        log::info!("plantuml_jar: {}", &config.plantuml_jar);
        log::info!("java_binary: {}", &config.java_binary);
        log::info!("force_generation: {}", force_generation);
    }
    // resolve the LAST_GENERATION file
    let last_gen_path_buff = Path::new(config.cache_directory.as_str()).join("LAST_GENERATION");
    let last_gen_path = last_gen_path_buff.as_path();
    create_parent_directory(last_gen_path)?;
    // create PlantUML
    let plantuml = create_plantuml(
        &config.java_binary,
        &config.plantuml_jar,
        &config.plantuml_version,
    )?;
    plantuml.download()?;
    // get latest generation
    let last_generation_timestamp = get_last_generation_timestamp(last_gen_path)?;
    // discover .puml files
    let puml_paths = get_puml_paths(config)?.flatten();
    // generate .puml file
    for source_path in puml_paths {
        let last_modification_timestamp = get_last_modified(&source_path)?;
        log::debug!(
            "{} > {} = {}",
            last_modification_timestamp,
            last_generation_timestamp,
            last_modification_timestamp > last_generation_timestamp,
        );
        if force_generation || last_modification_timestamp > last_generation_timestamp {
            log::info!("generate {:?}", source_path);
            let plantuml_args = arg_matches
                .get_many::<String>("plantuml_args")
                .unwrap_or_default()
                .map(|v| v.to_string())
                .collect::<Vec<_>>();
            plantuml.render(&source_path, Some(plantuml_args))?;
        }
    }
    save_last_generation_timestamp(last_gen_path)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::cli::build_cli;
    use crate::utils::{create_parent_directory, delete_file, delete_file_or_directory};

    use super::*;

    #[test]
    fn test_diagram_generation() {
        delete_file_or_directory("target/tests/cmd/diagram/generate".as_ref()).unwrap();
        for source_file in &["diagrams_a.puml", "folder_a/diagrams_b.puml"] {
            let from_prefix = "test/source";
            let from_path = Path::new(from_prefix).join(source_file);
            let to_prefix = "target/tests/cmd/diagram/generate/source";
            let to_path = Path::new(to_prefix).join(source_file);
            create_parent_directory(&to_path).unwrap();
            std::fs::copy(&from_path, &to_path).unwrap();
        }
        let arg_matches = build_cli().get_matches_from([
            "plantuml-generator",
            "-l=Debug",
            "diagram",
            "generate",
            "-s=target/tests/cmd/diagram/generate/source",
            "-C=target/tests/cmd/diagram/generate/cache",
            "-P=test/plantuml-1.2022.4.jar",
            "-a=-png -v",
        ]);
        execute_diagram_generate(
            arg_matches
                .subcommand_matches("diagram")
                .unwrap()
                .subcommand_matches("generate")
                .unwrap(),
        )
        .unwrap();
        let path_diagram_a_0_png =
            Path::new("target/tests/cmd/diagram/generate/source/diagram_a_0.png");
        assert!(path_diagram_a_0_png.exists());
        let path_diagram_a_1_png =
            Path::new("target/tests/cmd/diagram/generate/source/diagram_a_1.png");
        assert!(Path::new(path_diagram_a_1_png).exists());
        let path_diagram_b_0_png =
            Path::new("target/tests/cmd/diagram/generate/source/folder_a/diagram_b_0.png");
        assert!(path_diagram_b_0_png.exists());
        let path_diagram_b_1_png =
            Path::new("target/tests/cmd/diagram/generate/source/folder_a/diagram_b_1.png");
        assert!(path_diagram_b_1_png.exists());
        // get path_diagram_a_0_src modified
        let path_diagram_a_0_png_modified_before =
            path_diagram_a_0_png.metadata().unwrap().modified().unwrap();
        // mutate path_diagram_a_0_src
        let path_diagram_a_0_src =
            Path::new("target/tests/cmd/diagram/generate/source/diagrams_a.puml");
        let mut file_diagram_a_0_src = OpenOptions::new()
            .write(true)
            .append(true)
            .open(path_diagram_a_0_src)
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        file_diagram_a_0_src.write_all("' test".as_bytes()).unwrap();
        // delete path_diagram_b_0_png
        delete_file(path_diagram_b_0_png).unwrap();
        assert!(!path_diagram_b_0_png.exists());
        // execute generate diagrams
        execute_diagram_generate(
            arg_matches
                .subcommand_matches("diagram")
                .unwrap()
                .subcommand_matches("generate")
                .unwrap(),
        )
        .unwrap();
        // check path_diagram_a_0_png has been generated again
        let path_diagram_a_0_png_modified_after =
            path_diagram_a_0_png.metadata().unwrap().modified().unwrap();
        assert!(path_diagram_a_0_png_modified_before < path_diagram_a_0_png_modified_after);
        // check diagram_b_0 hasn't been generated again
        assert!(!path_diagram_b_0_png.exists());
    }
}
