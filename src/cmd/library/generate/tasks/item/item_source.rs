use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tera::{Context, Tera};

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::cmd::library::manifest::element::Shape;
use crate::cmd::library::manifest::item::Item;
use crate::constants::{SPRITES, SPRITE_LG};
use crate::error::Error;
use crate::result::Result;
use crate::utils::{create_parent_directory, delete_file};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum Element {
    Icon {
        /// The name of the PlantUML procedure.
        procedure_name: String,
        /// The URN of the icon.
        icon_urn: String,
        /// The name of the stereotype.
        stereotype_name: String,
        /// A set of custom properties.
        properties: HashMap<String, Value>,
    },
    IconCard {
        /// The name of the PlantUML procedure.
        procedure_name: String,
        /// The name of the sprite.
        sprite_name: String,
        /// The name of the stereotype.
        stereotype_name: String,
        /// The name of the element family.
        family_name: String,
        /// A set of custom properties.
        properties: HashMap<String, Value>,
    },
    IconGroup {
        /// The name of the PlantUML procedure.
        procedure_name: String,
        /// The name of the sprite.
        sprite_name: String,
        /// The name of the stereotype.
        stereotype_name: String,
        /// The label of the element.
        default_label: String,
        /// A set of custom properties.
        properties: HashMap<String, Value>,
    },
    Group {
        /// The name of the PlantUML procedure.
        procedure_name: String,
        /// The name of the stereotype.
        stereotype_name: String,
        /// The label of the element.
        default_label: String,
        /// A set of custom properties.
        properties: HashMap<String, Value>,
    },
    Custom {
        /// The name of the PlantUML procedure.
        procedure_name: String,
        /// A set of custom properties.
        properties: HashMap<String, Value>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ItemSourceTask {
    /// The URN of the Item.
    item_urn: String,
    /// The paths of the cached sprite values.
    cached_sprite_paths: Vec<String>,
    /// The elements of the Item.
    elements: Vec<Element>,
    /// The path to the output directory.
    output_directory: String,
    /// The name of the Tera template
    template: String,
}

impl ItemSourceTask {
    pub fn create(config: &Config, item: &Item) -> Result<ItemSourceTask> {
        let mut cached_sprite_paths: Vec<String> = vec![];

        if let Some(icon) = &item.icon {
            // if the item defines an icon, then sprites will be generated
            for size in &SPRITES {
                cached_sprite_paths.push(
                    match Path::new(&config.cache_directory)
                        .join(icon.get_sprite_value_path(&item.urn, size))
                        .as_path()
                        .to_str()
                    {
                        None => {
                            return Err(Error::Simple(
                                "unable to get the sprite cached path".to_string(),
                            ));
                        }
                        Some(v) => v.to_string(),
                    },
                );
            }
        }

        Ok(ItemSourceTask {
            item_urn: item.urn.value.clone(),
            cached_sprite_paths,
            elements: item
                .elements
                .iter()
                .map(|element| {
                    let procedure_name = element.shape.get_element_name(&item.urn);
                    let sprite_name = item
                        .icon
                        .clone()
                        .map(|i| i.get_sprite_name(&item.urn, SPRITE_LG))
                        .unwrap_or_default();
                    match element.shape {
                        Shape::Icon {
                            ref stereotype_name,
                            ref properties,
                        } => Element::Icon {
                            procedure_name,
                            icon_urn: item.urn.value.clone(),
                            stereotype_name: stereotype_name.clone(),
                            properties: properties.clone(),
                        },
                        Shape::IconCard {
                            ref stereotype_name,
                            ref properties,
                        } => Element::IconCard {
                            procedure_name,
                            sprite_name,
                            family_name: item.family.clone().unwrap_or_default(),
                            stereotype_name: stereotype_name.clone(),
                            properties: properties.clone(),
                        },
                        Shape::IconGroup {
                            ref stereotype_name,
                            ref properties,
                        } => Element::IconGroup {
                            procedure_name,
                            sprite_name,
                            stereotype_name: stereotype_name.clone(),
                            default_label: item.urn.label.clone(),
                            properties: properties.clone(),
                        },
                        Shape::Group {
                            ref stereotype_name,
                            ref properties,
                        } => Element::Group {
                            procedure_name,
                            stereotype_name: stereotype_name.clone(),
                            default_label: item.urn.label.clone(),
                            properties: properties.clone(),
                        },
                        Shape::Custom { ref properties } => Element::Custom {
                            procedure_name,
                            properties: properties.clone(),
                        },
                    }
                })
                .collect(),
            output_directory: config.output_directory.clone(),
            template: item.templates.source.clone(),
        })
    }
    fn get_relative_source_path(&self) -> Box<Path> {
        Box::from(Path::new(format!("{}.puml", self.item_urn,).as_str()))
    }
    fn get_full_source_path(&self) -> Box<Path> {
        Path::new(&self.output_directory)
            .join(self.get_relative_source_path())
            .into_boxed_path()
    }
}

impl Task for ItemSourceTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!("{} - ItemIconTask - cleanup", &self.item_urn);
        if CleanupScope::ItemSource.is_included_in(_scopes) {
            delete_file(self.get_full_source_path().as_ref())?;
        }
        Ok(())
    }

    fn render_atomic_templates(&self, _tera: &Tera) -> Result<()> {
        log::debug!("{} - ItemIconTask - render templates", &self.item_urn);

        let destination_path = self.get_full_source_path();

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

        // get the sprite value from the cached files
        let mut sprites: Vec<String> = vec![];
        for cached_sprite_path in &self.cached_sprite_paths {
            let cached_sprite_value = read_to_string(cached_sprite_path)
                .map(|c| c.trim().to_string())
                .map_err(|e| {
                    Error::Cause(
                        format!(
                            "unable to read the cached sprite file {}",
                            cached_sprite_path
                        ),
                        Box::from(e),
                    )
                })?;
            sprites.push(cached_sprite_value);
        }

        let mut context = Context::new();
        context.insert("sprites", &sprites);
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
    use crate::constants::{
        get_default_icon_card_element_stereotype, get_default_icon_element_stereotype,
        get_default_icon_group_element_stereotype, get_default_template_item_source,
    };
    use crate::tera::create_tera;

    use super::*;

    #[test]
    fn test_builtin_elements() {
        let generator = ItemSourceTask {
            item_urn: "Package/Module/Family/BuiltInItem".to_string(),
            cached_sprite_paths: vec![
                "test/sprite_value_A.puml".to_string(),
                "test/sprite_value_B.puml".to_string(),
            ],
            elements: vec![
                Element::Icon {
                    procedure_name: "Item".to_string(),
                    icon_urn: "Package/Module/Family/BuiltInItem".to_string(),
                    stereotype_name: get_default_icon_element_stereotype(),
                    properties: HashMap::default(),
                },
                Element::IconCard {
                    procedure_name: "ItemCard".to_string(),
                    sprite_name: "ItemLg".to_string(),
                    family_name: "Family".to_string(),
                    stereotype_name: get_default_icon_card_element_stereotype(),
                    properties: HashMap::default(),
                },
                Element::IconGroup {
                    procedure_name: "ItemGroup".to_string(),
                    sprite_name: "ItemLg".to_string(),
                    stereotype_name: get_default_icon_group_element_stereotype(),
                    default_label: "Item".to_string(),
                    properties: HashMap::default(),
                },
                Element::IconGroup {
                    procedure_name: "ItemBisGroup".to_string(),
                    sprite_name: "ItemBisLg".to_string(),
                    stereotype_name: "ItemBis".to_string(),
                    default_label: "Item Bis".to_string(),
                    properties: HashMap::default(),
                },
                Element::Group {
                    procedure_name: "SimpleGroup".to_string(),
                    stereotype_name: "SimpleGroup".to_string(),
                    default_label: "Simple Group".to_string(),
                    properties: HashMap::default(),
                },
            ],
            output_directory: "target/tests/item_source".to_string(),
            template: get_default_template_item_source(),
        };
        let tera = &create_tera(TEMPLATES.to_vec(), None).unwrap();
        generator.cleanup(&[CleanupScope::All]).unwrap();
        generator.render_atomic_templates(tera).unwrap();
        let content = read_to_string(format!(
            "{}/{}.puml",
            generator.output_directory, generator.item_urn,
        ))
        .unwrap();
        assert!(content.contains("LX_6N8UPcPbT0G"));
        assert!(content.contains(
            r"IconElement($id, 'IconElement', 'Package/Module/Family/BuiltInItem', $name, $tech, $desc)"
        ));
        assert!(content.contains(
            r"IconCardElement($id, 'IconCardElement', '<$ItemLg>', 'Family', $funcName, $content)"
        ));
        assert!(content
            .contains(r"IconGroupElement($id, 'IconGroupElement', '<$ItemLg>', $name, $tech)"));
        assert!(content.contains(r"IconGroupElement($id, 'ItemBis', '<$ItemBisLg>', $name, $tech)"));
        assert!(content.contains(r"GroupElement($id, 'SimpleGroup', $name, $tech)"));
    }

    #[test]
    fn test_custom_elements() {
        let properties: HashMap<String, Value> = serde_yaml::from_str(
            r#"
            keyA: valueA
            keyB: [ itemA, itemB ]
        "#,
        )
        .unwrap();
        let generator = ItemSourceTask {
            item_urn: "Package/Module/Family/CustomItem".to_string(),
            cached_sprite_paths: vec![],
            elements: vec![Element::Custom {
                procedure_name: "CustomItem".to_string(),
                properties,
            }],
            output_directory: "target/tests/item_source".to_string(),
            template: "custom_item_source.tera".to_string(),
        };
        let tera = &create_tera(TEMPLATES.to_vec(), Some("test/tera/**".to_string())).unwrap();
        generator.cleanup(&[CleanupScope::All]).unwrap();
        generator.render_atomic_templates(tera).unwrap();
        let content = read_to_string(format!(
            "{}/{}.puml",
            generator.output_directory, generator.item_urn,
        ))
        .unwrap();
        assert!(content.contains("' valueA"));
        assert!(content.contains("' itemA,itemB"));
        assert!(content.contains("!procedure CustomItem($id)"));
    }
}
