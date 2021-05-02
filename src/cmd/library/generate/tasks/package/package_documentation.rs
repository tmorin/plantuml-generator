use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::error::Error;
use crate::manifest::library::Library;
use crate::manifest::package::Package;
use crate::result::Result;
use crate::utils::{create_parent_directory, delete_file};

#[derive(Debug, Deserialize, Serialize)]
pub struct Module {
    /// The URN of the module.
    module_urn: String,
    /// The name of the module.
    module_name: String,
    /// The number of items in the module.
    nbr_items: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Example {
    /// The name of the example.
    name: String,
    /// The path to the image.
    destination: String,
    /// The path to the source.
    source: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageDocumentationTask {
    /// The URN of the package.
    package_urn: String,
    /// The name of the package.
    package_name: String,
    /// The relative path to the library base path.
    path_to_base: String,
    /// The modules of the package.
    modules: Vec<Module>,
    /// The examples of the package.
    examples: Vec<Example>,
    /// The path to the output directory.
    output_directory: String,
    /// The name of the Tera template
    template: String,
}

impl PackageDocumentationTask {
    pub fn create(
        config: &Config,
        library: &Library,
        package: &Package,
    ) -> Result<PackageDocumentationTask> {
        Ok(PackageDocumentationTask {
            package_urn: package.urn.value.clone(),
            package_name: package.urn.name.clone(),
            path_to_base: package.urn.path_to_base.clone(),
            modules: package
                .modules
                .iter()
                .map(|module| Module {
                    module_urn: module.urn.value.clone(),
                    module_name: module.urn.name.clone(),
                    nbr_items: module.items.len() as u32,
                })
                .collect(),
            examples: package
                .examples
                .iter()
                .map(|example| Example {
                    name: example.name.clone(),
                    destination: example
                        .get_destination_path(&package.urn, &library.customization.icon_format),
                    source: example.get_source_path(&package.urn),
                })
                .collect(),
            output_directory: config.output_directory.clone(),
            template: package.templates.documentation.clone(),
        })
    }
    fn get_relative_destination_path(&self) -> Box<Path> {
        Box::from(Path::new(
            format!("{}/README.md", self.package_urn).as_str(),
        ))
    }
    fn get_full_destination_path(&self) -> Box<Path> {
        Path::new(&self.output_directory)
            .join(self.get_relative_destination_path())
            .into_boxed_path()
    }
}

impl Task for PackageDocumentationTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!("{} - PackageDocumentationTask - cleanup", self.package_urn);
        delete_file(self.get_full_destination_path().as_ref())?;
        Ok(())
    }

    fn render_templates(&self, _tera: &Tera) -> Result<()> {
        log::debug!(
            "{} - PackageDocumentationTask - render templates",
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

    use crate::constants::get_default_template_package_documentation;
    use crate::tera::create_tera;
    use crate::urn::Urn;

    use super::*;
    use crate::cmd::library::generate::templates::TEMPLATES;

    #[test]
    fn test_template() {
        let tera = &create_tera(TEMPLATES.to_vec(), None).unwrap();
        let package_urn = Urn::from("Package");
        let module_a_urn = Urn::from("Package/ModuleA");
        let module_b_urn = Urn::from("Package/ModuleB");
        let generator = PackageDocumentationTask {
            package_urn: package_urn.value,
            package_name: package_urn.name,
            path_to_base: package_urn.path_to_base,
            modules: vec![
                Module {
                    module_urn: module_a_urn.value,
                    module_name: module_a_urn.name,
                    nbr_items: 2,
                },
                Module {
                    module_urn: module_b_urn.value,
                    module_name: module_b_urn.name,
                    nbr_items: 3,
                },
            ],
            examples: vec![
                Example {
                    name: "example A name".to_string(),
                    destination: "example A destination".to_string(),
                    source: "example A source".to_string(),
                },
                Example {
                    name: "example B name".to_string(),
                    destination: "example B destination".to_string(),
                    source: "example B source".to_string(),
                },
            ],
            output_directory: "target/tests/package_documentation_generator".to_string(),
            template: get_default_template_package_documentation(),
        };
        generator.cleanup(&vec![CleanupScope::All]).unwrap();
        generator.render_templates(tera).unwrap();
        let content =
            read_to_string(format!("{}/Package/README.md", generator.output_directory)).unwrap();
        assert!(content.contains("# Package"));
        assert!(content.contains("include('Package/bootstrap')"));
        assert!(content.contains("The package provides 2 modules."));
        assert!(content.contains("[Package/ModuleA]"));
        assert!(content.contains("[Package/ModuleB]"));
        assert!(content.contains("The package provides 2 examples."));
        assert!(content.contains("## example A name"));
        assert!(content.contains("## example B name"));
    }
}
