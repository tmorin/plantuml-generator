use std::fs::File;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::cmd::library::manifest::library::Library;
use crate::utils::{create_parent_directory, delete_file};

#[derive(Debug, Deserialize, Serialize)]
pub struct LibraryBootstrapTask {
    /// The name of the library.
    library_name: String,
    /// The URL to fetch the library remotely.
    remote_url: String,
    /// The format of the items' icons.
    icon_format: String,
    /// The width for text.
    text_width_max: u32,
    /// The width for message.
    msg_width_max: u32,
    /// The font size for XS.
    font_size_xs: u32,
    /// The font size for SM.
    font_size_sm: u32,
    /// The font size for MD.
    font_size_md: u32,
    /// The font size for LG.
    font_size_lg: u32,
    /// The default font color.
    font_color: String,
    /// The default font light color.
    font_color_light: String,
    /// The path to the output directory.
    output_directory: String,
    /// The name of the Tera template
    template: String,
}

impl LibraryBootstrapTask {
    pub fn create(config: &Config, library: &Library) -> Result<LibraryBootstrapTask> {
        Ok(LibraryBootstrapTask {
            library_name: library.name.clone(),
            remote_url: library.remote_url.clone(),
            icon_format: library.customization.icon_format.clone(),
            text_width_max: library.customization.text_width_max,
            msg_width_max: library.customization.msg_width_max,
            font_size_xs: library.customization.font_size_xs,
            font_size_sm: library.customization.font_size_sm,
            font_size_md: library.customization.font_size_md,
            font_size_lg: library.customization.font_size_lg,
            font_color: library.customization.font_color.clone(),
            font_color_light: library.customization.font_color_light.clone(),
            output_directory: config.output_directory.clone(),
            template: library.templates.bootstrap.clone(),
        })
    }
    fn get_relative_source_path(&self) -> Box<Path> {
        Box::from(Path::new("bootstrap.puml"))
    }
    fn get_full_source_path(&self) -> Box<Path> {
        Path::new(&self.output_directory)
            .join(self.get_relative_source_path())
            .into_boxed_path()
    }
}

impl Task for LibraryBootstrapTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!("{} - LibraryBootstrapTask - cleanup", self.library_name);
        delete_file(self.get_full_source_path().as_ref())?;
        Ok(())
    }

    fn render_atomic_templates(&self, _tera: &Tera) -> Result<()> {
        log::debug!(
            "{} - LibraryBootstrapTask - render templates",
            self.library_name
        );

        let destination_path = self.get_full_source_path();

        // skip early when generation not required
        if destination_path.exists() {
            return Ok(());
        }

        // create the destination directory
        create_parent_directory(&destination_path)?;

        // create the destination file
        let destination_file = File::create(&destination_path).map_err(|e| {
            anyhow::Error::new(e).context("unable to create the destination file".to_string())
        })?;

        let mut context = Context::new();
        context.insert("data", &self);
        _tera
            .render_to(&self.template, &context, destination_file)
            .map_err(|e| {
                anyhow::Error::new(e).context(format!("unable to render {}", &self.template))
            })
    }
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use crate::cmd::library::generate::templates::TEMPLATES;
    use crate::constants::get_default_template_library_bootstrap;
    use crate::tera::create_tera;

    use super::*;

    #[test]
    fn test_template() {
        let tera = &create_tera(TEMPLATES.to_vec(), None).unwrap();
        let generator = LibraryBootstrapTask {
            library_name: "a library".to_string(),
            remote_url: "a remote url".to_string(),
            icon_format: "png".to_string(),
            text_width_max: 300,
            msg_width_max: 400,
            font_size_xs: 2,
            font_size_sm: 4,
            font_size_md: 6,
            font_size_lg: 8,
            font_color: "black".to_string(),
            font_color_light: "grey".to_string(),
            output_directory: "target/tests/library_bootstrap_generator".to_string(),
            template: get_default_template_library_bootstrap(),
        };
        generator.cleanup(&[CleanupScope::All]).unwrap();
        generator.render_atomic_templates(tera).unwrap();
        let content =
            read_to_string(format!("{}/bootstrap.puml", generator.output_directory)).unwrap();
        assert!(content.contains(r##"!global $LIB_BASE_LOCATION="a remote url""##));
        assert!(content.contains(r##"!global $ICON_FORMAT="png""##));
        assert!(content.contains(r##"!global $FONT_SIZE_XS=2"##));
        assert!(content.contains(r##"!global $FONT_COLOR="black""##));
    }
}
