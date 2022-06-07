use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use image::imageops::FilterType;
use image::io::Reader;
use image::GenericImageView;

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::error::Error;
use crate::manifest::icon::Icon;
use crate::manifest::item::Item;
use crate::manifest::library::Library;
use crate::result::Result;
use crate::utils::{create_parent_directory, delete_file};

pub struct ItemIconTask {
    /// The URN of the Item.
    item_urn: String,
    /// The path of the source image file.
    full_source_image: String,
    /// The path of the destination image file.
    pub full_destination_image: String,
    /// The height of the destination icon.
    destination_icon_height: u32,
    /// The command/path of the inkscape binary.
    inkscape_binary: String,
}

impl ItemIconTask {
    pub fn create(
        config: &Config,
        library: &Library,
        item: &Item,
        icon: &Icon,
        full_source_image: &str,
    ) -> Result<ItemIconTask> {
        let full_destination_image = match Path::new(&config.output_directory)
            .join(icon.get_icon_path(&item.urn, &library.customization.icon_format))
            .as_path()
            .to_str()
        {
            None => return Err(Error::Simple("unable to get destination path".to_string())),
            Some(v) => v.to_string(),
        };
        Ok(ItemIconTask {
            item_urn: item.urn.value.clone(),
            full_source_image: full_source_image.to_string(),
            full_destination_image,
            destination_icon_height: library.customization.icon_height,
            inkscape_binary: config.inkscape_binary.clone(),
        })
    }
    fn generate_icon_with_inkscape(&self) -> Result<()> {
        log::debug!(
            "generate the icon {} to {} with inkscape",
            &self.full_source_image,
            &self.full_destination_image
        );

        // generate the icon
        let output = Command::new(&self.inkscape_binary)
            .arg(&self.full_source_image)
            .arg(format!(
                "--export-filename={}",
                &self.full_destination_image
            ))
            .arg(format!("--export-height={}", &self.destination_icon_height))
            .output()
            .map_err(|e| {
                Error::Cause(
                    format!("unable to generate {}", &self.full_destination_image),
                    Box::from(e),
                )
            })?;

        // check generation worked
        match output.status.success() {
            true => Ok(()),
            false => {
                io::stdout().write_all(&output.stdout).map_err(|e| {
                    Error::Cause("unable to write stdout".to_string(), Box::from(e))
                })?;
                io::stderr().write_all(&output.stderr).map_err(|e| {
                    Error::Cause("unable to write stderr".to_string(), Box::from(e))
                })?;
                Err(Error::Simple("failed to create the icon".to_string()))
            }
        }
    }
    fn generate_icon_with_builtin_library(&self) -> Result<()> {
        log::debug!(
            "generate the icon {} to {} with built library",
            &self.full_source_image,
            &self.full_destination_image
        );

        // get an handler on the source icon
        let image = Reader::open(&self.full_source_image)
            .map_err(|e| {
                Error::Cause(
                    format!("unable to open {}", &self.full_source_image),
                    Box::from(e),
                )
            })?
            .decode()
            .map_err(|e| {
                Error::Cause(
                    format!("unable to decode {}", &self.full_source_image),
                    Box::from(e),
                )
            })?;

        // compute the width of the sprite icon
        let (width, height) = image.dimensions();
        let destination_icon_width = self.destination_icon_height * width / height;

        // generate the sprite icon
        image
            .resize(
                destination_icon_width,
                self.destination_icon_height,
                FilterType::Triangle,
            )
            .save(&self.full_destination_image)
            .map_err(|e| {
                Error::Cause(
                    format!("unable to save {}", &self.full_destination_image),
                    Box::from(e),
                )
            })?;

        Ok(())
    }
}

impl Task for ItemIconTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!("{} - ItemIconTask - cleanup", &self.item_urn);
        if CleanupScope::ItemIcon.is_included_in(_scopes) {
            delete_file(Path::new(self.full_destination_image.as_str()))?;
        }
        Ok(())
    }

    fn create_resources(&self) -> Result<()> {
        log::debug!("{} - ItemIconTask - create resources", &self.item_urn);

        let icon_destination_path = Path::new(&self.full_destination_image);

        // skip early when generation not required
        if icon_destination_path.exists() {
            return Ok(());
        }

        // create the parent directory
        create_parent_directory(icon_destination_path)?;

        // resolve the icon source extension
        let icon_source_extension = match Path::new(&self.full_source_image).extension() {
            None => Err(Error::Simple(
                "unable to get the extension of the icon source".to_string(),
            )),
            Some(s) => Ok(s.to_str().unwrap_or_default().to_string()),
        }?;

        // generate the icon
        if icon_source_extension.eq("svg") {
            // generate with inkscape when the source is an SVG
            self.generate_icon_with_inkscape()?;
        } else {
            // generate with built-in library when the source is an SVG
            self.generate_icon_with_builtin_library()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::cmd::library::generate::config::Config;

    use super::*;

    #[test]
    fn test_create_resources_with_inkscape() {
        let config = Config::default();
        let generator = ItemIconTask {
            item_urn: "PackageA/ModuleB/FamilyC/ItemD".to_string(),
            full_source_image: "test/raw/eip/MessageConstruction__MessageExpiration.svg"
                .to_string(),
            full_destination_image: "target/tests/item_icon/output.png".to_string(),
            destination_icon_height: 50,
            inkscape_binary: config.inkscape_binary,
        };
        generator.cleanup(&vec![CleanupScope::All]).unwrap();
        generator.create_resources().unwrap();
        assert!(Path::new("target/tests/item_icon/output.png").exists());
        generator.cleanup(&vec![CleanupScope::All]).unwrap();
        assert!(!Path::new("target/tests/item_icon/output.png").exists());
        generator.create_resources().unwrap();
        assert!(Path::new("target/tests/item_icon/output.png").exists());
    }

    #[test]
    fn test_create_resources_with_builtin_library() {
        let config = Config::default();
        let generator = ItemIconTask {
            item_urn: "PackageA/ModuleB/FamilyC/ItemD".to_string(),
            full_source_image: "test/original_icon.png".to_string(),
            full_destination_image: "target/tests/item_icon/output_with_builtin.png".to_string(),
            destination_icon_height: 50,
            inkscape_binary: config.inkscape_binary,
        };
        generator.cleanup(&vec![CleanupScope::All]).unwrap();
        generator.create_resources().unwrap();
        assert!(Path::new("target/tests/item_icon/output_with_builtin.png").exists());
        generator.cleanup(&vec![CleanupScope::All]).unwrap();
        assert!(!Path::new("target/tests/item_icon/output_with_builtin.png").exists());
        generator.create_resources().unwrap();
        assert!(Path::new("target/tests/item_icon/output_with_builtin.png").exists());
    }
}
