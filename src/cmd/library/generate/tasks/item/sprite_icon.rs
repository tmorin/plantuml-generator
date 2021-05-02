use std::path::Path;

use image::imageops::FilterType;
use image::io::Reader;
use image::GenericImageView;
use serde::{Deserialize, Serialize};

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::error::Error;
use crate::manifest::icon::Icon;
use crate::manifest::item::Item;
use crate::result::Result;
use crate::utils::{create_parent_directory, delete_file};

#[derive(Debug, Deserialize, Serialize)]
pub struct SpriteIconTask {
    /// The URN of the Item.
    item_urn: String,
    /// The path of the source icon file.
    full_source_icon: String,
    /// The path of the destination icon file.
    pub full_destination_icon: String,
    /// The height of the destination icon.
    destination_icon_height: u32,
}

impl SpriteIconTask {
    pub fn create(
        config: &Config,
        item: &Item,
        icon: &Icon,
        full_source_icon: &str,
        (sprite_size_name, sprite_size_value): (&str, u32),
    ) -> Result<SpriteIconTask> {
        // resolve the path to host the input sprite image
        let full_destination_icon = match Path::new(&config.cache_directory)
            .join(icon.get_sprite_image_path(&item.urn, sprite_size_name))
            .as_path()
            .to_str()
        {
            None => {
                return Err(Error::Simple(format!(
                    "unable to get full_destination_icon for {}/{}",
                    item.urn, sprite_size_name
                )));
            }
            Some(v) => v.to_string(),
        };

        Ok(SpriteIconTask {
            item_urn: item.urn.value.clone(),
            full_source_icon: full_source_icon.to_string(),
            full_destination_icon,
            destination_icon_height: sprite_size_value,
        })
    }
}

impl Task for SpriteIconTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!(
            "{} - SpriteIconTask - cleanup {}",
            &self.item_urn,
            &self.full_destination_icon
        );
        if CleanupScope::SpriteIcon.is_included_in(_scopes) {
            delete_file(Path::new(&self.full_destination_icon))?;
        }
        Ok(())
    }

    fn create_resources(&self) -> Result<()> {
        log::debug!(
            "{} - SpriteIconTask - create resource {}",
            &self.item_urn,
            &self.full_destination_icon
        );

        let destination_icon_path = Path::new(&self.full_destination_icon);

        // skip early when generation not required
        if destination_icon_path.exists() {
            return Ok(());
        }

        // create the destination directory
        create_parent_directory(destination_icon_path)?;

        // get an handler on the source icon
        let image = Reader::open(&self.full_source_icon)
            .map_err(|e| {
                Error::Cause(
                    format!("unable to open {}", &self.full_source_icon),
                    Box::from(e),
                )
            })?
            .decode()
            .map_err(|e| {
                Error::Cause(
                    format!("unable to decode {}", &self.full_source_icon),
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
            .save(&self.full_destination_icon)
            .map_err(|e| {
                Error::Cause(
                    format!("unable to save {}", &self.full_destination_icon),
                    Box::from(e),
                )
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_resources() {
        let generator = SpriteIconTask {
            item_urn: "a/urn".to_string(),
            full_source_icon: "test/original_icon.png".to_string(),
            full_destination_icon: "target/tests/sprite_icon/test_generate.png".to_string(),
            destination_icon_height: 16,
        };
        generator.cleanup(&vec![CleanupScope::All]).unwrap();
        generator.create_resources().unwrap();
        assert!(Path::new(&generator.full_destination_icon).exists());
        generator.cleanup(&vec![CleanupScope::All]).unwrap();
        assert!(!Path::new(&generator.full_destination_icon).exists());
        generator.create_resources().unwrap();
        assert!(Path::new(&generator.full_destination_icon).exists());
    }
}
