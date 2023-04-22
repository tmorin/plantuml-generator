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
pub struct Package {
    /// The URN of the package.
    package_urn: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LibraryDocumentationTask {
    /// The name of the library.
    library_name: String,
    /// The URL to fetch the library remotely.
    remote_url: String,
    /// The packages of the library.
    packages: Vec<Package>,
    /// The path to the output directory.
    output_directory: String,
    /// The name of the Tera template
    template: String,
}

impl LibraryDocumentationTask {
    pub fn create(config: &Config, library: &Library) -> Result<LibraryDocumentationTask> {
        Ok(LibraryDocumentationTask {
            library_name: library.name.clone(),
            remote_url: library.remote_url.clone(),
            packages: library
                .packages
                .iter()
                .map(|p| Package {
                    package_urn: p.urn.value.clone(),
                })
                .collect(),
            output_directory: config.output_directory.clone(),
            template: library.templates.documentation.clone(),
        })
    }
    fn get_relative_destination_path(&self) -> Box<Path> {
        Box::from(Path::new("README.md"))
    }
    fn get_full_destination_path(&self) -> Box<Path> {
        Path::new(&self.output_directory)
            .join(self.get_relative_destination_path())
            .into_boxed_path()
    }
}

impl Task for LibraryDocumentationTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!("{} - LibraryDocumentationTask - cleanup", self.library_name);
        delete_file(self.get_full_destination_path().as_ref())?;
        Ok(())
    }

    fn render_atomic_templates(&self, _tera: &Tera) -> Result<()> {
        log::debug!(
            "{} - LibraryDocumentationTask - render templates",
            self.library_name
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
    use crate::constants::get_default_template_library_documentation;
    use crate::tera::create_tera;
    use crate::urn::Urn;

    use super::*;

    #[test]
    fn test_template() {
        let tera = &create_tera(TEMPLATES.to_vec(), None).unwrap();
        let generator = LibraryDocumentationTask {
            library_name: "a library".to_string(),
            remote_url: "a remote url".to_string(),
            packages: vec![
                Package {
                    package_urn: Urn::from("PackageA").value,
                },
                Package {
                    package_urn: Urn::from("PackageB").value,
                },
                Package {
                    package_urn: Urn::from("PackageC").value,
                },
            ],
            output_directory: "target/tests/library_documentation_generator".to_string(),
            template: get_default_template_library_documentation(),
        };
        generator.cleanup(&[CleanupScope::All]).unwrap();
        generator.render_atomic_templates(tera).unwrap();
        let content = read_to_string(format!("{}/README.md", generator.output_directory)).unwrap();
        assert!(content.contains(r##"The library provides 3 packages."##));
        assert!(content.contains(r##"- [PackageA](PackageA/README.md)"##));
        assert!(content.contains(r##"- [PackageB](PackageB/README.md)"##));
        assert!(content.contains(r##"- [PackageC](PackageC/README.md)"##));
    }
}
