use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

use crate::utils::{create_parent_directory, should_add_smetana_layout};
use anyhow::Result;

/// Global mutex to serialise writes to stdout/stderr across rayon threads so
/// that per-diagram output from concurrent `render` calls is not interleaved.
static OUTPUT_MUTEX: Mutex<()> = Mutex::new(());

/// Encapsulates the configuration and execution of PlantUML commands.
///
/// `PlantUML` wraps the Java-based PlantUML tool, handling both rendering
/// of `.puml` source files and downloading of the PlantUML JAR from GitHub.
#[derive(Debug, Clone)]
pub struct PlantUML {
    /// The command/path of the java binary.
    java_binary: String,
    /// The path of the PlantUML jar.
    plantuml_jar: String,
    /// The path of the PlantUML jar.
    plantuml_version: String,
}

impl PlantUML {
    /// Prepare the final arguments list, adding smetana layout if needed
    fn prepare_args(&self, p_args_as_strings: Option<&[String]>) -> Vec<String> {
        let mut final_args = p_args_as_strings.unwrap_or_default().to_vec();

        // Check if we should automatically add smetana layout
        if should_add_smetana_layout(&final_args) {
            log::info!(
                "GraphViz dot not available and GRAPHVIZ_DOT not set, using -Playout=smetana"
            );
            final_args.insert(0, "-Playout=smetana".to_string());
        }

        final_args
    }

    /// Renders a PlantUML source file using the configured Java binary and JAR.
    ///
    /// Invokes `java -jar <plantuml_jar> <source_path> [args...]` and streams
    /// stdout/stderr to the process output. Returns an error if the process
    /// exits with a non-zero status.
    ///
    /// When GraphViz is not configured for use by PlantUML (that is, `dot` is
    /// unavailable and `GRAPHVIZ_DOT` is not set) and the user has not already
    /// specified a layout engine via `args`, `-Playout=smetana` is prepended
    /// automatically so that diagrams can still be generated.
    ///
    /// # Arguments
    ///
    /// * `source_path` - Path to the `.puml` source file to render.
    /// * `p_args_as_strings` - Optional extra arguments forwarded to PlantUML
    ///   (e.g. `-png`, `-svg`).
    ///
    /// # Errors
    ///
    /// Returns an error if the source path cannot be converted to a string,
    /// if the Java process fails to start, or if PlantUML exits with a
    /// non-zero status code.
    pub fn render(&self, source_path: &Path, p_args_as_strings: Option<&[String]>) -> Result<()> {
        //get the source
        let source = match source_path.to_str() {
            None => {
                return Err(anyhow::Error::msg(format!(
                    "unable to get the string value of {:?}",
                    source_path
                )));
            }
            Some(s) => s,
        };

        // Build the final arguments list
        let final_args = self.prepare_args(p_args_as_strings);

        // generate the file
        let p_args = if final_args.is_empty() {
            None
        } else {
            Some(
                final_args
                    .iter()
                    .map(OsString::from)
                    .collect::<Vec<OsString>>(),
            )
        };

        let output = Command::new(&self.java_binary)
            .arg("-jar")
            .arg(&self.plantuml_jar)
            .arg(source)
            .args(p_args.unwrap_or_default())
            .output()
            .map_err(|e| anyhow::Error::new(e).context(format!("unable to render {}", source)))?;
        {
            // Hold the lock only while writing to stdout/stderr so that output
            // from concurrent rayon threads is not interleaved.
            // Recover from a poisoned mutex rather than panicking: the lock is
            // used purely for I/O serialisation and holds no invariants.
            let _guard = OUTPUT_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;
        }
        // check the generation
        if !output.status.success() {
            return Err(anyhow::Error::msg(format!("failed to render {}", source)));
        }

        Ok(())
    }

    /// Downloads the PlantUML JAR from the official GitHub releases page.
    ///
    /// The JAR is placed at the path configured via `plantuml_jar`. If the
    /// file already exists, the download is skipped.
    ///
    /// # Errors
    ///
    /// Returns an error if the parent directory cannot be created, if the
    /// destination file cannot be opened, or if the HTTP download fails.
    pub fn download(&self) -> Result<()> {
        // https://github.com/plantuml/plantuml/releases/download/v1.2024.7/plantuml-1.2024.7.jar
        let url = format!(
            "https://github.com/plantuml/plantuml/releases/download/v{}/plantuml-{}.jar",
            self.plantuml_version, self.plantuml_version,
        );

        let destination_path = Path::new(&self.plantuml_jar);
        if destination_path.exists() {
            log::info!("the PlantUML jar is already there");
            return Ok(());
        }

        create_parent_directory(destination_path)?;

        let mut destination_file = File::create(destination_path).map_err(|e| {
            anyhow::Error::new(e).context(format!("unable to open {}", &self.plantuml_jar))
        })?;

        log::info!("download the PlantUML jar from {}", url);
        reqwest::blocking::get(&url)
            .map_err(|e| anyhow::Error::new(e).context(format!("unable to download {}", &url)))?
            .copy_to(&mut destination_file)
            .map_err(|e| {
                anyhow::Error::new(e).context(format!("unable to write {}", &self.plantuml_jar))
            })?;

        Ok(())
    }
}

/// Creates a new [`PlantUML`] instance with the given configuration.
///
/// # Arguments
///
/// * `java_binary` - Path or name of the Java binary (e.g. `"java"`).
/// * `plantuml_jar` - Path to the PlantUML JAR file.
/// * `plantuml_version` - Version string used to download the JAR if needed
///   (e.g. `"1.2024.7"`).
///
/// # Errors
///
/// Currently infallible; returns `Ok` for forward-compatibility.
pub fn create_plantuml(
    java_binary: &str,
    plantuml_jar: &str,
    plantuml_version: &str,
) -> Result<PlantUML> {
    Ok(PlantUML {
        java_binary: java_binary.to_string(),
        plantuml_jar: plantuml_jar.to_string(),
        plantuml_version: plantuml_version.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use crate::constants::{JAVA_BINARY, PLANTUML_VERSION};
    use crate::utils::delete_file;
    use serial_test::serial;

    use super::*;

    #[test]
    fn test_download() {
        let plantuml = PlantUML {
            java_binary: JAVA_BINARY.to_string(),
            plantuml_jar: "target/plantuml.jar".to_string(),
            plantuml_version: PLANTUML_VERSION.to_string(),
        };
        delete_file(Path::new(&plantuml.plantuml_jar)).unwrap_or_default();
        plantuml.download().expect("the download fails");
    }

    #[test]
    #[serial]
    fn test_prepare_args_adds_smetana_when_no_graphviz() {
        // Simulate environment with no GraphViz by setting PLANTUML_IGNORE_DOT
        std::env::set_var("PLANTUML_IGNORE_DOT", "1");
        // Clear GRAPHVIZ_DOT to ensure fallback behavior
        std::env::remove_var("GRAPHVIZ_DOT");

        let plantuml = PlantUML {
            java_binary: "java".to_string(),
            plantuml_jar: "plantuml.jar".to_string(),
            plantuml_version: "1.0.0".to_string(),
        };

        // Test with no args - should not panic and should return some args
        let result_no_args = plantuml.prepare_args(None);
        assert!(
            !result_no_args.is_empty(),
            "prepare_args should return at least one argument when called with None"
        );

        // Test with args but no layout - original args must be preserved regardless of dot availability
        let args = vec!["-png".to_string()];
        let result = plantuml.prepare_args(Some(&args));

        // The original argument should still be present and remain the last argument
        assert!(
            !result.is_empty(),
            "prepare_args should return at least one argument when called with user args"
        );
        assert_eq!(
            result.last().unwrap(),
            "-png",
            "Original args should be preserved at the end of the argument list"
        );

        // Clean up
        std::env::remove_var("GRAPHVIZ_DOT");
        std::env::remove_var("PLANTUML_IGNORE_DOT");
    }

    #[test]
    fn test_prepare_args_respects_user_layout() {
        let plantuml = PlantUML {
            java_binary: "java".to_string(),
            plantuml_jar: "plantuml.jar".to_string(),
            plantuml_version: "1.0.0".to_string(),
        };

        // Test with user-specified layout - should NOT add smetana
        let args = vec!["-Playout=elk".to_string(), "-png".to_string()];
        let result = plantuml.prepare_args(Some(&args));

        // User's layout should be preserved
        assert_eq!(result[0], "-Playout=elk");
        assert_eq!(result[1], "-png");
        assert_eq!(result.len(), 2);
    }

    #[test]
    #[serial]
    fn test_prepare_args_with_graphviz_dot_set() {
        // When GRAPHVIZ_DOT is set, should not add smetana
        std::env::set_var("GRAPHVIZ_DOT", "/usr/bin/dot");

        let plantuml = PlantUML {
            java_binary: "java".to_string(),
            plantuml_jar: "plantuml.jar".to_string(),
            plantuml_version: "1.0.0".to_string(),
        };

        let args = vec!["-png".to_string()];
        let result = plantuml.prepare_args(Some(&args));

        // Should NOT have smetana added
        assert_eq!(result[0], "-png");
        assert_eq!(result.len(), 1);

        // Clean up
        std::env::remove_var("GRAPHVIZ_DOT");
    }
}
