use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::utils::create_parent_directory;

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
    pub fn render(&self, source_path: &Path) -> Result<()> {
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
        // generate the file
        let output = Command::new(&self.java_binary)
            .arg("-jar")
            .arg(&self.plantuml_jar)
            .arg(source)
            .output()
            .map_err(|e| anyhow::Error::new(e).context(format!("unable to render {}", source)))?;
        io::stdout().write_all(&output.stdout).unwrap();
        // .map_err(|e| anyhow::Error::new(e).context(format!("unable to write stdout"), Box::from(e)))?;
        io::stderr().write_all(&output.stderr).unwrap();
        // check the generation
        if !output.status.success() {
            // .map_err(|e| anyhow::Error::new(e).context(format!("unable to write stderr"), Box::from(e)))?;
            return Err(anyhow::Error::msg(format!("failed to render {}", source)));
        }

        Ok(())
    }
    pub fn download(&self) -> Result<()> {
        // https://netcologne.dl.sourceforge.net/project/plantuml/1.2022.4/plantuml.1.2022.4.jar
        // https://downloads.sourceforge.net/project/plantuml/1.2022.4/plantuml.1.2022.4.jar
        let url = format!(
            "https://downloads.sourceforge.net/project/plantuml/{}/plantuml.{}.jar",
            self.plantuml_version, self.plantuml_version,
        );

        let destination_path = Path::new(&self.plantuml_jar);
        if destination_path.exists() {
            log::info!("the PlantUML jar is already there");
            return Ok(());
        }

        log::info!("download the PlantUML jar from {}", url);

        create_parent_directory(destination_path)?;

        let mut destination_file = File::create(destination_path).map_err(|e| {
            anyhow::Error::new(e).context(format!("unable to open {}", &self.plantuml_jar))
        })?;

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
}
