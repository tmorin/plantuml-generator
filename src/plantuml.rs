use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::utils::{create_parent_directory, should_add_smetana_layout};
use anyhow::Result;

#[derive(Debug)]
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
    fn prepare_args(&self, p_args_as_strings: Option<Vec<String>>) -> Vec<String> {
        let mut final_args = p_args_as_strings.unwrap_or_default();
        
        // Check if we should automatically add smetana layout
        if should_add_smetana_layout(&final_args) {
            log::info!("GraphViz dot not available and GRAPHVIZ_DOT not set, using -Playout=smetana");
            final_args.insert(0, "-Playout=smetana".to_string());
        }
        
        final_args
    }
    
    pub fn render(&self, source_path: &Path, p_args_as_strings: Option<Vec<String>>) -> Result<()> {
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
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;
        // check the generation
        if !output.status.success() {
            return Err(anyhow::Error::msg(format!("failed to render {}", source)));
        }

        Ok(())
    }
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
        let args = Some(vec!["-png".to_string()]);
        let result = plantuml.prepare_args(args);
        
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
        let args = Some(vec!["-Playout=elk".to_string(), "-png".to_string()]);
        let result = plantuml.prepare_args(args);
        
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
        
        let args = Some(vec!["-png".to_string()]);
        let result = plantuml.prepare_args(args);
        
        // Should NOT have smetana added
        assert_eq!(result[0], "-png");
        assert_eq!(result.len(), 1);
        
        // Clean up
        std::env::remove_var("GRAPHVIZ_DOT");
    }
}
