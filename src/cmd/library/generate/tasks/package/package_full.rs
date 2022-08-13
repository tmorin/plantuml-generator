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
pub struct Item {
    /// The URN of the package.
    item_urn: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageFullTask {
    /// The URN of the package.
    package_urn: String,
    /// The items of the library.
    items: Vec<Item>,
    /// The path to the output directory.
    output_directory: String,
    /// The name of the Tera template
    template: String,
}

impl PackageFullTask {
    pub fn create(config: &Config, package: &Package) -> Result<PackageFullTask> {
        let items = package
            .modules
            .iter()
            .fold(vec![], |mut module_urns, module| {
                module
                    .items
                    .iter()
                    .map(|item| item.urn.clone())
                    .for_each(|urn| module_urns.push(urn));
                module_urns
            })
            .into_iter()
            .map(|item_urn| Item {
                item_urn: item_urn.value,
            })
            .collect();
        Ok(PackageFullTask {
            package_urn: package.urn.value.clone(),
            items,
            output_directory: config.output_directory.clone(),
            template: package.templates.full.clone(),
        })
    }
    fn get_relative_destination_path(&self) -> Box<Path> {
        Box::from(Path::new(
            format!("{}/full.puml", self.package_urn).as_str(),
        ))
    }
    fn get_full_destination_path(&self) -> Box<Path> {
        Path::new(&self.output_directory)
            .join(self.get_relative_destination_path())
            .into_boxed_path()
    }
}

impl Task for PackageFullTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!("{} - PackageFullTask - cleanup", self.package_urn);
        delete_file(self.get_full_destination_path().as_ref())?;
        Ok(())
    }

    fn render_templates(&self, _tera: &Tera) -> Result<()> {
        log::debug!("{} - PackageFullTask - render templates", self.package_urn);

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
    use crate::constants::get_default_template_package_full;
    use crate::tera::create_tera;

    use super::*;

    #[test]
    fn test_template() {
        let tera = &create_tera(TEMPLATES.to_vec(), Some("test/tera/*".to_string())).unwrap();
        let generator = PackageFullTask {
            package_urn: "Package".to_string(),
            items: vec![
                Item {
                    item_urn: "urn_a".to_string(),
                },
                Item {
                    item_urn: "urn_b".to_string(),
                },
                Item {
                    item_urn: "urn_c".to_string(),
                },
            ],
            output_directory: "target/tests/package_full_generator".to_string(),
            template: get_default_template_package_full(),
        };
        generator.cleanup(&vec![CleanupScope::All]).unwrap();
        generator.render_templates(tera).unwrap();
        let content =
            read_to_string(format!("{}/Package/full.puml", generator.output_directory)).unwrap();
        assert!(content.trim().contains("@startuml"));
        assert!(content.trim().contains("include('Package/bootstrap')"));
        assert!(content.trim().contains("include('urn_a')"));
        assert!(content.trim().contains("include('urn_b')"));
        assert!(content.trim().contains("include('urn_c')"));
        assert!(content.trim().contains("@enduml"));
    }
}
