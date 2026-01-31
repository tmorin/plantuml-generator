use std::fs::File;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::cmd::library::manifest::library::Library;
use crate::cmd::library::manifest::module::Module;
use crate::utils::{create_parent_directory, delete_file};

type ItemManifest = crate::cmd::library::manifest::item::Item;

#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    /// The URN of the Item.
    item_urn: String,
    /// The family of the Item.
    family: Option<String>,
    /// The relative path to the illustration.
    illustration: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModuleDocumentationTask {
    /// The URN of the module.
    module_urn: String,
    /// The name of the module.
    module_name: String,
    /// The relative path to the library base path.
    path_to_base: String,
    /// The items of the module having a family.
    items_with_family: Vec<Item>,
    /// The items of the module without a family.
    items_without_family: Vec<Item>,
    /// The path to the output directory.
    output_directory: String,
    /// The name of the Tera template
    template: String,
}

pub fn resolve_illustration(library: &Library, item: &ItemManifest) -> String {
    match &item.icon {
        None => item.elements[0]
            .shape
            .get_local_snippet_image_path(&item.urn, &library.customization.icon_format),
        Some(icon) => icon.get_icon_path(&item.urn, &library.customization.icon_format),
    }
}

impl ModuleDocumentationTask {
    pub fn create(
        config: &Config,
        library: &Library,
        module: &Module,
    ) -> Result<ModuleDocumentationTask> {
        Ok(ModuleDocumentationTask {
            module_urn: module.urn.value.clone(),
            module_name: module.urn.name.clone(),
            path_to_base: module.urn.path_to_base.clone(),
            items_with_family: module
                .items
                .iter()
                .filter(|i| i.family.is_some())
                .map(|item| Item {
                    item_urn: item.urn.value.clone(),
                    family: item.family.clone(),
                    illustration: resolve_illustration(library, item),
                })
                .collect(),
            items_without_family: module
                .items
                .iter()
                .filter(|i| i.family.is_none())
                .map(|item| Item {
                    item_urn: item.urn.value.clone(),
                    family: None,
                    illustration: resolve_illustration(library, item),
                })
                .collect(),
            output_directory: config.output_directory.clone(),
            template: module.templates.documentation.clone(),
        })
    }
    fn get_relative_destination_path(&self) -> Box<Path> {
        Box::from(Path::new(
            format!("{}/README.md", self.module_urn,).as_str(),
        ))
    }
    fn get_full_destination_path(&self) -> Box<Path> {
        Path::new(&self.output_directory)
            .join(self.get_relative_destination_path())
            .into_boxed_path()
    }
}

impl Task for ModuleDocumentationTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!("{} - ModuleDocumentationTask - cleanup", self.module_urn);
        delete_file(self.get_full_destination_path().as_ref())?;
        Ok(())
    }

    fn render_atomic_templates(&self, _tera: &Tera) -> Result<()> {
        log::debug!(
            "{} - ModuleDocumentationTask - render templates",
            self.module_urn
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
    use crate::constants::get_default_template_module_documentation;
    use crate::tera::create_tera;
    use crate::urn::Urn;

    use super::*;

    #[test]
    fn test_template() {
        let tera = &create_tera(TEMPLATES.to_vec(), None).unwrap();
        let urn = Urn::from("Package/Module");
        let item_a_urn = Urn::from("Package/Module/FamilyA/itemA");
        let item_b_urn = Urn::from("Package/Module/FamilyB/itemB");
        let item_c_urn = Urn::from("Package/Module/FamilyA/itemC");
        let item_d_urn = Urn::from("Package/Module/itemD");
        let generator = ModuleDocumentationTask {
            module_urn: urn.value,
            module_name: urn.name,
            path_to_base: urn.path_to_base,
            items_with_family: vec![
                Item {
                    item_urn: item_a_urn.value,
                    family: Some("FamilyA".to_string()),
                    illustration: "illustration itemA".to_string(),
                },
                Item {
                    item_urn: item_b_urn.value,
                    family: Some("FamilyB".to_string()),
                    illustration: "illustration itemB".to_string(),
                },
                Item {
                    item_urn: item_c_urn.value,
                    family: Some("FamilyA".to_string()),
                    illustration: "illustration itemC".to_string(),
                },
            ],
            items_without_family: vec![Item {
                item_urn: item_d_urn.value,
                family: None,
                illustration: "illustration itemD".to_string(),
            }],
            output_directory: "target/tests/module_documentation_generator".to_string(),
            template: get_default_template_module_documentation(),
        };
        generator.cleanup(&[CleanupScope::All]).unwrap();
        generator.render_atomic_templates(tera).unwrap();
        let content = read_to_string(format!(
            "{}/Package/Module/README.md",
            generator.output_directory
        ))
        .unwrap();
        assert!(content.contains("The module contains 4 items."));
        assert!(content.contains("[Package/Module/itemD](../../Package/Module/itemD.md)"));
        assert!(content.contains("## FamilyA"));
        assert!(content.contains("## FamilyB"));
    }
}
