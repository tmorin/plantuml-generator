use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::error::Error;
use crate::manifest::example::Example;
use crate::manifest::library::Library;
use crate::manifest::package::Package;
use crate::plantuml::PlantUML;
use crate::result::Result;
use crate::utils::{create_parent_directory, delete_file};

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageExampleTask {
    /// The URN of the package.
    package_urn: String,
    /// The name of the Tera template
    template: String,
    /// The relative path to the library base path.
    path_to_base: String,
    /// The name of the Tera template
    full_source_path: String,
    /// The name of the Tera template
    full_image_path: String,
}

impl PackageExampleTask {
    pub fn create(
        config: &Config,
        library: &Library,
        package: &Package,
        example: &Example,
    ) -> Result<PackageExampleTask> {
        let full_source_path = Path::new(&config.output_directory)
            .join(example.get_source_path(&package.urn))
            .as_path()
            .to_str()
            .map(|v| v.to_string())
            .ok_or_else(|| Error::Simple("unable to get full_source_path".to_string()))?;
        let full_image_path = Path::new(&config.output_directory)
            .join(example.get_destination_path(&package.urn, &library.customization.icon_format))
            .as_path()
            .to_str()
            .map(|v| v.to_string())
            .ok_or_else(|| Error::Simple("unable to get full_image_path".to_string()))?;
        Ok(PackageExampleTask {
            package_urn: package.urn.value.clone(),
            template: example.template.clone(),
            path_to_base: package.urn.path_to_base.clone(),
            full_source_path,
            full_image_path,
        })
    }
}

impl Task for PackageExampleTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!("{} - PackageExampleTask - cleanup", self.template);
        if CleanupScope::Example.is_included_in(_scopes) {
            delete_file(Path::new(&self.full_source_path))?;
            delete_file(Path::new(&self.full_image_path))?;
        }
        Ok(())
    }

    fn render_atomic_templates(&self, _tera: &Tera) -> Result<()> {
        log::debug!("{} - PackageExampleTask - render templates", self.template);

        let destination_path = Path::new(&self.full_source_path);

        // skip early when generation not required
        if destination_path.exists() {
            return Ok(());
        }

        // create the destination directory
        create_parent_directory(destination_path)?;

        // create the destination file
        let destination_file = File::create(&destination_path).map_err(|e| {
            Error::Cause(
                "unable to create the destination file".to_string(),
                Box::from(e),
            )
        })?;

        let mut context = Context::new();
        context.insert("data", &self);
        _tera
            .render_to(&self.template, &context, destination_file)
            .map_err(|e| Error::Cause(format!("unable to render {}", &self.template), Box::from(e)))
    }

    fn render_sources(&self, plantuml: &PlantUML) -> Result<()> {
        log::debug!("{} - PackageExampleTask - render sources", self.template);

        let destination_path = Path::new(&self.full_image_path);

        // skip early when generation not required
        if destination_path.exists() {
            return Ok(());
        }

        // render the snippet
        let source_path = Path::new(&self.full_source_path);
        plantuml.render(source_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use crate::cmd::library::generate::templates::TEMPLATES;
    use crate::tera::create_tera;

    use super::*;

    #[test]
    fn test_template() {
        let tera = &create_tera(TEMPLATES.to_vec(), Some("test/tera/*".to_string())).unwrap();
        let generator = PackageExampleTask {
            package_urn: "test".to_string(),
            template: "package_example_test.tera".to_string(),
            path_to_base: "".to_string(),
            full_source_path: "target/tests/package_examples/source.puml".to_string(),
            full_image_path: "target/tests/package_examples/source.png".to_string(),
        };
        generator.cleanup(&vec![CleanupScope::All]).unwrap();
        generator.render_atomic_templates(tera).unwrap();
        let content = read_to_string("target/tests/package_examples/source.puml").unwrap();
        assert!(content.trim().contains("the content of the example"));
    }
}
