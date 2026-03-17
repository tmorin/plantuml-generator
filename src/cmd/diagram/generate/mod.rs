use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use chrono::prelude::*;
use clap::ArgMatches;
use glob::glob;
use rayon::prelude::*;

use crate::cmd::diagram::generate::config::Config;
use crate::plantuml::{create_plantuml, PlantUML};
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

fn get_puml_paths(config: &Config) -> Vec<PathBuf> {
    config
        .source_patterns
        .split(",")
        .map(str::trim)
        .map(|pattern| format!("{}/{}", config.source_directory, pattern))
        .flat_map(|glob_pattern| {
            glob(&glob_pattern)
                .map(|paths| paths.flatten())
                .map_err(|e| {
                    anyhow::Error::new(e).context(format!(
                        "unable to parse the glob pattern ({})",
                        &glob_pattern
                    ))
                })
                .map(|paths| paths.collect::<Vec<PathBuf>>())
                .unwrap()
        })
        .collect::<Vec<PathBuf>>()
}

/// Renders diagrams sequentially (used for benchmarking comparison).
#[cfg(test)]
fn render_sequential(
    puml_paths: &[PathBuf],
    plantuml: &PlantUML,
    plantuml_args: &[String],
    force_generation: bool,
    last_generation_timestamp: i64,
) -> Result<()> {
    for source_path in puml_paths {
        let last_modification_timestamp = get_last_modified(source_path)?;
        log::debug!(
            "{} > {} = {}",
            last_modification_timestamp,
            last_generation_timestamp,
            last_modification_timestamp > last_generation_timestamp,
        );
        if force_generation || last_modification_timestamp > last_generation_timestamp {
            log::info!("generate {:?}", source_path);
            plantuml.render(source_path, Some(plantuml_args.to_vec()))?;
        }
    }
    Ok(())
}

/// Renders diagrams in parallel using rayon for improved throughput.
///
/// All source paths are processed concurrently. If one or more renders fail,
/// their errors are collected, sorted by source path for deterministic output,
/// and combined into a single error (one failure per line) so that no failure
/// is silently discarded.
fn render_parallel(
    puml_paths: &[PathBuf],
    plantuml: &PlantUML,
    plantuml_args: &[String],
    force_generation: bool,
    last_generation_timestamp: i64,
) -> Result<()> {
    let mut errors: Vec<(PathBuf, String)> = puml_paths
        .par_iter()
        .filter_map(|source_path| {
            let result: Result<()> = (|| {
                let last_modification_timestamp = get_last_modified(source_path)?;
                log::debug!(
                    "{} > {} = {}",
                    last_modification_timestamp,
                    last_generation_timestamp,
                    last_modification_timestamp > last_generation_timestamp,
                );
                if force_generation || last_modification_timestamp > last_generation_timestamp {
                    log::info!("generate {:?}", source_path);
                    plantuml.render(source_path, Some(plantuml_args.to_vec()))?;
                }
                Ok(())
            })();
            result.err().map(|e| (source_path.clone(), e.to_string()))
        })
        .collect();

    if errors.is_empty() {
        return Ok(());
    }
    // Sort by path so the combined message is deterministic across runs.
    errors.sort_by(|(a, _), (b, _)| a.cmp(b));
    let message = errors
        .into_iter()
        .map(|(path, msg)| format!("{}: {}", path.display(), msg))
        .collect::<Vec<_>>()
        .join("\n");
    Err(anyhow::anyhow!("{}", message))
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
    // discover source files
    let puml_paths = get_puml_paths(config);
    // collect plantuml args once outside the loop
    let plantuml_args: Vec<String> = arg_matches
        .get_many::<String>("plantuml_args")
        .unwrap_or_default()
        .map(|v| v.to_string())
        .collect();
    // generate source files in parallel
    render_parallel(
        &puml_paths,
        &plantuml,
        &plantuml_args,
        force_generation,
        last_generation_timestamp,
    )?;
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
        for source_file in &[
            "diagrams_a.puml",
            "diagrams_c.plantuml",
            "folder_a/diagrams_b.puml",
        ] {
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
        let path_diagram_b_0_png =
            Path::new("target/tests/cmd/diagram/generate/source/diagram_c_0.png");
        assert!(path_diagram_b_0_png.exists());
        let path_diagram_b_1_png =
            Path::new("target/tests/cmd/diagram/generate/source/diagram_c_1.png");
        assert!(path_diagram_b_1_png.exists());
        // get path_diagram_a_0_src modified
        let path_diagram_a_0_png_modified_before =
            path_diagram_a_0_png.metadata().unwrap().modified().unwrap();
        // mutate path_diagram_a_0_src
        let path_diagram_a_0_src =
            Path::new("target/tests/cmd/diagram/generate/source/diagrams_a.puml");
        let mut file_diagram_a_0_src = OpenOptions::new()
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

    /// Benchmark test that measures sequential vs parallel diagram generation speedup.
    ///
    /// Creates 6 PlantUML source files, renders them both sequentially and in parallel,
    /// measures wall-clock time for each approach, and asserts speedup ≥ 1.3x.
    ///
    /// Run with: `cargo test test_parallel_speedup -- --nocapture --ignored`
    #[test]
    #[ignore]
    fn test_parallel_speedup() {
        use crate::plantuml::create_plantuml;
        use std::time::Instant;
        use tempfile::TempDir;

        let dir = TempDir::new().expect("failed to create temp dir");
        let plantuml_jar = "test/plantuml-1.2022.4.jar";
        let plantuml = create_plantuml("java", plantuml_jar, "")
            .expect("failed to create plantuml");

        // Create 6 simple PlantUML source files for a meaningful parallel workload.
        let diagram_count = 6;
        let puml_paths: Vec<PathBuf> = (0..diagram_count)
            .map(|i| {
                let path = dir.path().join(format!("bench_diagram_{i}.puml"));
                std::fs::write(
                    &path,
                    format!(
                        "@startuml bench_{i}\nobject A_{i}\nobject B_{i}\nA_{i} -> B_{i}\n@enduml\n"
                    ),
                )
                .expect("failed to write puml file");
                path
            })
            .collect();

        // Warm-up: one sequential pass to populate the JVM class-data caches
        // so that neither the sequential nor the parallel timed run is penalised
        // by cold JVM start-up overhead.
        render_sequential(&puml_paths, &plantuml, &[], true, 0)
            .expect("warm-up render failed");

        // Measure sequential rendering time.
        let seq_start = Instant::now();
        render_sequential(&puml_paths, &plantuml, &[], true, 0)
            .expect("sequential render failed");
        let seq_duration = seq_start.elapsed();

        // Measure parallel rendering time.
        let par_start = Instant::now();
        render_parallel(&puml_paths, &plantuml, &[], true, 0)
            .expect("parallel render failed");
        let par_duration = par_start.elapsed();

        let speedup = seq_duration.as_secs_f64() / par_duration.as_secs_f64();

        println!();
        println!("=== Diagram Generation Performance Benchmark ===");
        println!("  Diagram count       : {}", diagram_count);
        println!("  CPU threads (rayon) : {}", rayon::current_num_threads());
        println!("  Sequential time     : {:.3}s", seq_duration.as_secs_f64());
        println!("  Parallel time       : {:.3}s", par_duration.as_secs_f64());
        println!("  Speedup             : {:.2}x", speedup);
        println!("  Target speedup      : ≥ 1.30x");
        println!(
            "  Result              : {}",
            if speedup >= 1.3 { "PASS ✓" } else { "FAIL ✗" }
        );
        println!("================================================");

        assert!(
            speedup >= 1.3,
            "Expected speedup ≥ 1.3x but got {:.2}x \
             (sequential={:.3}s, parallel={:.3}s). \
             Ensure multiple CPU cores are available.",
            speedup,
            seq_duration.as_secs_f64(),
            par_duration.as_secs_f64(),
        );
    }

    /// Verifies that `render_parallel` collects ALL failures and that the
    /// combined error message includes every failing path, sorted alphabetically.
    ///
    /// Two `.puml` files are created and rendered against a non-existent JAR so
    /// that both renders fail with a non-zero Java exit code.  The test asserts
    /// that neither error is silently discarded and that the paths appear in
    /// sorted order (deterministic output).
    #[test]
    fn test_parallel_error_aggregation() {
        use crate::plantuml::create_plantuml;
        use tempfile::TempDir;

        let dir = TempDir::new().expect("failed to create temp dir");
        // Construct a path within the temp dir that is never created, so that
        // java exits with a non-zero status for every render attempt.
        let fake_jar = dir.path().join("nonexistent_plantuml.jar");
        let plantuml = create_plantuml("java", fake_jar.to_str().unwrap(), "")
            .expect("failed to create plantuml");

        let path_a = dir.path().join("aaa_diagram.puml");
        let path_b = dir.path().join("zzz_diagram.puml");
        std::fs::write(&path_a, "@startuml\nobject A\n@enduml\n").unwrap();
        std::fs::write(&path_b, "@startuml\nobject B\n@enduml\n").unwrap();

        let puml_paths = vec![path_a.clone(), path_b.clone()];

        let result = render_parallel(&puml_paths, &plantuml, &[], true, 0);
        assert!(result.is_err(), "expected render_parallel to fail");

        let err_msg = result.unwrap_err().to_string();

        // Both failing paths must appear in the combined error.
        assert!(
            err_msg.contains(path_a.to_str().unwrap()),
            "expected error to mention {}, got: {}",
            path_a.display(),
            err_msg
        );
        assert!(
            err_msg.contains(path_b.to_str().unwrap()),
            "expected error to mention {}, got: {}",
            path_b.display(),
            err_msg
        );

        // aaa_diagram should appear before zzz_diagram (sorted by path).
        let pos_a = err_msg.find(path_a.to_str().unwrap()).unwrap();
        let pos_b = err_msg.find(path_b.to_str().unwrap()).unwrap();
        assert!(
            pos_a < pos_b,
            "expected errors sorted by path (aaa before zzz), got: {}",
            err_msg
        );
    }
}
