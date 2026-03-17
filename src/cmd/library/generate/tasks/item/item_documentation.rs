use std::fs::File;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::cmd::library::manifest::item::Item;
use crate::cmd::library::manifest::library::Library;
use crate::utils::{create_parent_directory, delete_file};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Object {
    Icon {
        /// The name of the icon.
        name: String,
        /// The relative path to the illustration from the Item directory.
        illustration_path: String,
    },
    Element {
        /// The name of the element.
        name: String,
        /// The relative path to the illustration from the Item directory.
        illustration_path: String,
        /// The path to the local snippet.
        full_snippet_local_path: String,
        /// The path to the remote snippet.
        full_snippet_remote_path: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ItemDocumentationTask {
    /// The URN of the Item.
    item_urn: String,
    /// The name of the Item.
    item_name: String,
    /// The elements of the Item.
    objects: Vec<Object>,
    /// The relative path to the library base path.
    path_to_base: String,
    /// The path to the output directory.
    output_directory: String,
    /// The name of the Tera template
    template: String,
}

impl ItemDocumentationTask {
    pub fn create(
        config: &Config,
        library: &Library,
        item: &Item,
    ) -> Result<ItemDocumentationTask> {
        let mut objects: Vec<Object> = vec![];

        if let Some(icon) = &item.icon {
            objects.push(Object::Icon {
                name: "Illustration".to_string(),
                illustration_path: icon
                    .get_icon_path(&item.urn, &library.customization.icon_format),
            })
        }

        for element in &item.elements {
            objects.push(Object::Element {
                name: element.shape.get_element_name(&item.urn),
                illustration_path: element
                    .shape
                    .get_local_snippet_image_path(&item.urn, &library.customization.icon_format),
                full_snippet_local_path: Path::new(&config.output_directory)
                    .join(element.shape.get_local_snippet_puml_path(&item.urn))
                    .as_path()
                    .to_str()
                    .map(|v| v.to_string())
                    .ok_or_else(|| {
                        anyhow::Error::msg("unable to get full_snippet_local_path".to_string())
                    })?,
                full_snippet_remote_path: Path::new(&config.output_directory)
                    .join(element.shape.get_remote_snippet_puml_path(&item.urn))
                    .as_path()
                    .to_str()
                    .map(|v| v.to_string())
                    .ok_or_else(|| {
                        anyhow::Error::msg("unable to get full_snippet_remote_path".to_string())
                    })?,
            });
        }

        Ok(ItemDocumentationTask {
            item_urn: item.urn.value.clone(),
            item_name: item.urn.name.clone(),
            objects,
            path_to_base: item.urn.get_parent().path_to_base,
            output_directory: config.output_directory.clone(),
            template: item.templates.documentation.clone(),
        })
    }
    pub fn get_relative_documentation_path(&self) -> Box<Path> {
        Box::from(Path::new(format!("{}.md", self.item_urn,).as_str()))
    }
    fn get_full_documentation_path(&self) -> Box<Path> {
        Path::new(&self.output_directory)
            .join(self.get_relative_documentation_path())
            .into_boxed_path()
    }
}

impl Task for ItemDocumentationTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!("{} - ItemDocumentationTask - cleanup", &self.item_urn);
        delete_file(self.get_full_documentation_path().as_ref())?;
        Ok(())
    }

    fn render_atomic_templates(&self, _tera: &Tera) -> Result<()> {
        log::debug!(
            "{} - ItemDocumentationTask - render templates",
            &self.item_urn
        );

        let destination_path = self.get_full_documentation_path();

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
    use crate::constants::get_default_template_item_documentation;
    use crate::tera::create_tera;
    use crate::urn::Urn;

    use super::*;

    #[test]
    fn test_template() {
        let tera = create_tera(TEMPLATES.to_vec(), None).unwrap();
        let urn = Urn::from("Package/Module/Family/Item");
        let item_name = String::from(&urn.label);
        let generator = ItemDocumentationTask {
            item_urn: urn.value.clone(),
            item_name,
            objects: vec![
                Object::Icon {
                    name: "Illustration".to_string(),
                    illustration_path: "./Icon.png".to_string(),
                },
                Object::Element {
                    name: "Icon".to_string(),
                    illustration_path: "./Item.png".to_string(),
                    full_snippet_local_path: "test/full_snippet_local_path.puml".to_string(),
                    full_snippet_remote_path: "test/full_snippet_remote_path.puml".to_string(),
                },
                Object::Element {
                    name: "Card".to_string(),
                    illustration_path: "./ItemCard.png".to_string(),
                    full_snippet_local_path: "test/full_snippet_local_path.puml".to_string(),
                    full_snippet_remote_path: "test/full_snippet_remote_path.puml".to_string(),
                },
                Object::Element {
                    name: "Group".to_string(),
                    illustration_path: "./ItemGroup.png".to_string(),
                    full_snippet_local_path: "test/full_snippet_local_path.puml".to_string(),
                    full_snippet_remote_path: "test/full_snippet_remote_path.puml".to_string(),
                },
            ],
            path_to_base: urn.get_parent().path_to_base,
            output_directory: "target/tests/item_documentation".to_string(),
            template: get_default_template_item_documentation(),
        };
        generator.cleanup(&[CleanupScope::All]).unwrap();
        generator.render_atomic_templates(&tera).unwrap();
        let content = read_to_string(format!(
            "{}/{}.md",
            generator.output_directory, generator.item_urn,
        ))
        .unwrap();
        assert!(content.contains(r"# Item"));
        assert!(content.contains(r"| Illustration | Icon | Card | Group |"));
        assert!(content.contains(r"| ![illustration for Illustration](../../.././Icon.png) | ![illustration for Icon](../../.././Item.png) | ![illustration for Card](../../.././ItemCard.png) | ![illustration for Group](../../.././ItemGroup.png) |"));
        assert!(content.contains(r"## Icon"));
        assert!(content.contains(r"## Card"));
        assert!(content.contains(r"## Group"));
    }
}
