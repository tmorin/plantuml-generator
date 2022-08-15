use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::error::Error;
use crate::manifest::package::Package;
use crate::result::Result;
use crate::utils::{create_parent_directory, delete_file};

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageBootstrapTask {
    /// The URN of the package.
    package_urn: String,
    /// The path to the output directory.
    output_directory: String,
    /// The name of the Tera template
    template: String,
}

impl PackageBootstrapTask {
    pub fn create(config: &Config, package: &Package) -> Result<PackageBootstrapTask> {
        Ok(PackageBootstrapTask {
            package_urn: package.urn.value.clone(),
            output_directory: config.output_directory.clone(),
            template: package.templates.bootstrap.clone(),
        })
    }
    fn get_relative_destination_path(&self) -> Box<Path> {
        Box::from(Path::new(
            format!("{}/bootstrap.puml", self.package_urn).as_str(),
        ))
    }
    fn get_full_destination_path(&self) -> Box<Path> {
        Path::new(&self.output_directory)
            .join(self.get_relative_destination_path())
            .into_boxed_path()
    }
}

impl Task for PackageBootstrapTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!("{} - PackageBootstrapTask - cleanup", self.package_urn);
        delete_file(self.get_full_destination_path().as_ref())?;
        Ok(())
    }

    fn render_atomic_templates(&self, _tera: &Tera) -> Result<()> {
        log::debug!(
            "{} - PackageBootstrapTask - render templates",
            self.package_urn
        );

        let destination_path = self.get_full_destination_path();

        // skip early when generation not required
        if destination_path.exists() {
            return Ok(());
        }

        // create the destination directory
        create_parent_directory(&destination_path)?;

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
        let generator = PackageBootstrapTask {
            package_urn: "Package".to_string(),
            output_directory: "target/tests/package_bootstrap_generator".to_string(),
            template: "package_bootstrap_bis.tera".to_string(),
        };
        generator.cleanup(&vec![CleanupScope::All]).unwrap();
        generator.render_atomic_templates(tera).unwrap();
        let content = read_to_string(format!(
            "{}/Package/bootstrap.puml",
            generator.output_directory
        ))
            .unwrap();
        assert!(content.trim().contains("header"));
        assert!(content.trim().contains("content"));
        assert!(content.trim().contains("footer"));
    }
}
