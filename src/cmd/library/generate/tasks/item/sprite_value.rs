use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::error::Error;
use crate::manifest::icon::Icon;
use crate::manifest::item::Item;
use crate::result::Result;
use crate::utils::{create_parent_directory, delete_file};

#[derive(Debug, Deserialize, Serialize)]
pub struct SpriteValueTask {
    /// The URN of the Item.
    item_urn: String,
    /// The path of the source icon file.
    full_source_icon: String,
    /// The path of the destination text file.
    full_destination_text: String,
    /// The command/path of the java binary.
    java_binary: String,
    /// The path of the PlantUML jar.
    plantuml_jar: String,
}

impl SpriteValueTask {
    pub fn create(
        config: &Config,
        item: &Item,
        icon: &Icon,
        full_source_icon: &str,
        sprite_size_name: &str,
    ) -> Result<SpriteValueTask> {
        // resolve the path to host the input sprite image
        let full_destination_text = match Path::new(&config.cache_directory)
            .join(icon.get_sprite_value_path(&item.urn, sprite_size_name))
            .as_path()
            .to_str()
        {
            None => {
                return Err(Error::Simple(format!(
                    "unable to get full_destination_text for {}/{}",
                    item.urn, sprite_size_name
                )));
            }
            Some(v) => v.to_string(),
        };

        Ok(SpriteValueTask {
            item_urn: item.urn.value.clone(),
            full_source_icon: full_source_icon.to_string(),
            full_destination_text,
            java_binary: config.java_binary.clone(),
            plantuml_jar: config.plantuml_jar.clone(),
        })
    }
}

impl Task for SpriteValueTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!(
            "{} - SpriteValueTask - cleanup {}",
            &self.item_urn,
            &self.full_destination_text
        );
        if CleanupScope::SpriteValue.is_included_in(_scopes) {
            delete_file(Path::new(&self.full_destination_text))?;
        }
        Ok(())
    }

    fn create_resources(&self) -> Result<()> {
        log::debug!(
            "{} - SpriteValueTask - create resource {}",
            &self.item_urn,
            &self.full_destination_text
        );

        let destination_text_path = Path::new(&self.full_destination_text);

        // skip early when generation not required
        if destination_text_path.exists() {
            return Ok(());
        }

        // create the destination directory
        create_parent_directory(destination_text_path)?;

        // generate the sprite
        let output = Command::new(&self.java_binary)
            .arg("-jar")
            .arg(&self.plantuml_jar)
            .arg("-encodesprite")
            .arg("16z")
            .arg(&self.full_source_icon)
            .output()
            .map_err(|e| Error::Cause("unable to generate the sprite".to_string(), Box::from(e)))?;

        // check the generation
        if !output.status.success() {
            io::stdout()
                .write_all(&output.stdout)
                .map_err(|e| Error::Cause("unable to write stdout".to_string(), Box::from(e)))?;
            io::stderr()
                .write_all(&output.stderr)
                .map_err(|e| Error::Cause("unable to write stderr".to_string(), Box::from(e)))?;
            return Err(Error::Simple(String::from("failed to create the sprite")));
        }

        // write the sprite value
        let mut writer = fs::File::create(&self.full_destination_text).map_err(|e| {
            Error::Cause(
                format!("unable to create {}", &self.full_destination_text),
                Box::from(e),
            )
        })?;
        writer.write_all(&output.stdout).map_err(|e| {
            Error::Cause(
                format!("unable to write {}", &self.full_destination_text),
                Box::from(e),
            )
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::cmd::library::generate::config::Config;

    use super::*;

    #[test]
    fn test_create_resources() {
        let config = Config::default();
        let generator = SpriteValueTask {
            item_urn: "a/urn".to_string(),
            full_source_icon: "test/original_icon.png".to_string(),
            full_destination_text: "target/tests/sprite_value/test_generate.text".to_string(),
            java_binary: config.java_binary,
            plantuml_jar: "test/plantuml-1.2021.3.jar".to_string(),
        };
        generator.cleanup(&vec![CleanupScope::All]).unwrap();
        generator.create_resources().unwrap();
        assert!(Path::new(&generator.full_destination_text).exists());
        generator.cleanup(&vec![CleanupScope::All]).unwrap();
        assert!(!Path::new(&generator.full_destination_text).exists());
        generator.create_resources().unwrap();
        assert!(Path::new(&generator.full_destination_text).exists());
    }
}
