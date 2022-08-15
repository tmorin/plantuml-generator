use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::path::Path;

use heck::{ToTitleCase, ToUpperCamelCase};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use tera::{Context, Tera};

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::error::Error;
use crate::manifest::element::{Element, Shape};
use crate::manifest::item::Item;
use crate::manifest::library::Library;
use crate::manifest::package::Package;
use crate::plantuml::PlantUML;
use crate::result::Result;
use crate::utils::{create_parent_directory, delete_file};

#[derive(Debug, Clone, Eq, Deserialize, Serialize)]
pub enum SnippetMode {
    Local,
    Remote,
}

impl PartialEq for SnippetMode {
    fn eq(&self, other: &Self) -> bool {
        self.to_string().eq(&other.to_string())
    }
}

impl fmt::Display for SnippetMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SnippetMode::Local => "Local",
            SnippetMode::Remote => "Remote",
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ElementSnippetTask {
    /// The URL to fetch the library remotely.
    remote_url: String,
    /// The URN of the package
    package_urn: String,
    /// The URN of the Item.
    item_urn: String,
    /// The relative path to the library base path.
    path_to_base: String,
    /// The shape of the element (Icon, Card or Group).
    element_shape: String,
    /// The mode of the snippet (Local or Remote).
    snippet_mode: SnippetMode,
    /// The name of the PlantUML procedure.
    procedure_name: String,
    /// The name of the variable.
    variable_name: String,
    /// The label of the element.
    primary_label: String,
    /// The technical label of the element.
    technical_label: Option<String>,
    /// The description label of the element.
    description_label: Option<String>,
    /// The name of the Tera template
    template: String,
    /// The path of the snippet source.
    full_destination_source_path: String,
    /// The path of the snippet image.
    full_destination_image_path: String,
    /// A set of custom properties.
    properties: HashMap<String, Value>,
}

impl ElementSnippetTask {
    pub fn create(
        config: &Config,
        library: &Library,
        package: &Package,
        item: &Item,
        element: &Element,
        snippet_mode: SnippetMode,
    ) -> Result<ElementSnippetTask> {
        let procedure_name = element.shape.get_element_name(&item.urn);
        let variable_name = procedure_name.to_upper_camel_case();
        let primary_label = procedure_name.to_title_case();

        let full_destination_source_path = match snippet_mode {
            SnippetMode::Local => match Path::new(&config.output_directory)
                .join(element.shape.get_local_snippet_puml_path(&item.urn))
                .as_path()
                .to_str()
            {
                None => {
                    return Err(Error::Simple(
                        "unable to get the full path of get_local_snippet_puml_path".to_string(),
                    ));
                }
                Some(v) => v.to_string(),
            },
            SnippetMode::Remote => match Path::new(&config.output_directory)
                .join(element.shape.get_remote_snippet_puml_path(&item.urn))
                .as_path()
                .to_str()
            {
                None => {
                    return Err(Error::Simple(
                        "unable to get the full path of get_remote_snippet_puml_path".to_string(),
                    ));
                }
                Some(v) => v.to_string(),
            },
        };

        let full_destination_image_path =
            match snippet_mode {
                SnippetMode::Local => match Path::new(&config.output_directory)
                    .join(element.shape.get_local_snippet_image_path(
                        &item.urn,
                        &library.customization.icon_format,
                    ))
                    .as_path()
                    .to_str()
                {
                    None => {
                        return Err(Error::Simple(
                            "unable to get the full path of get_local_snippet_puml_path"
                                .to_string(),
                        ));
                    }
                    Some(v) => v.to_string(),
                },
                SnippetMode::Remote => match Path::new(&config.output_directory)
                    .join(element.shape.get_remote_snippet_image_path(
                        &item.urn,
                        &library.customization.icon_format,
                    ))
                    .as_path()
                    .to_str()
                {
                    None => {
                        return Err(Error::Simple(
                            "unable to get the full path of get_remote_snippet_puml_path"
                                .to_string(),
                        ));
                    }
                    Some(v) => v.to_string(),
                },
            };

        let properties = match &element.shape {
            Shape::Custom { properties } => properties.clone(),
            _ => HashMap::default(),
        };

        Ok(ElementSnippetTask {
            remote_url: library.remote_url.clone(),
            package_urn: package.urn.value.clone(),
            item_urn: item.urn.value.clone(),
            path_to_base: item.urn.get_parent().path_to_base,
            element_shape: element.shape.get_name(),
            snippet_mode,
            procedure_name,
            variable_name,
            primary_label,
            technical_label: None,
            description_label: None,
            template: item.templates.snippet.clone(),
            full_destination_source_path,
            full_destination_image_path,
            properties,
        })
    }
}

impl Task for ElementSnippetTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!(
            "{}/{}/{} - ElementSnippetTask - cleanup",
            &self.item_urn,
            &self.element_shape,
            &self.snippet_mode,
        );
        if CleanupScope::SnippetSource.is_included_in(_scopes) {
            delete_file(Path::new(&self.full_destination_source_path))?;
        }
        if CleanupScope::SnippetImage.is_included_in(_scopes) {
            delete_file(Path::new(&self.full_destination_image_path))?;
        }
        Ok(())
    }

    fn render_atomic_templates(&self, _tera: &Tera) -> Result<()> {
        log::debug!(
            "{}/{}/{} - ElementSnippetTask - render templates",
            &self.item_urn,
            &self.element_shape,
            &self.snippet_mode,
        );

        let destination_path = Path::new(&self.full_destination_source_path);

        // skip early when generation not required
        if destination_path.exists() {
            return Ok(());
        }

        // create the destination directory
        create_parent_directory(destination_path)?;

        // create the destination file
        let destination_file = File::create(destination_path).map_err(|e| {
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
        if self.snippet_mode.eq(&SnippetMode::Remote) {
            return Ok(());
        }

        log::debug!(
            "{}/{}/{} - ElementSnippetTask - render sources",
            &self.item_urn,
            &self.element_shape,
            &self.snippet_mode,
        );

        let destination_path = Path::new(&self.full_destination_image_path);

        // skip early when generation not required
        if destination_path.exists() {
            return Ok(());
        }

        // render the snippet
        let source_path = Path::new(&self.full_destination_source_path);
        plantuml.render(source_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use crate::cmd::library::generate::tasks::item::element_snippet::SnippetMode::{Local, Remote};
    use crate::cmd::library::generate::templates::TEMPLATES;
    use crate::constants::get_default_template_item_snippet;
    use crate::tera::create_tera;
    use crate::urn::Urn;

    use super::*;

    #[test]
    fn test_render_templates_built_in() {
        let tera = &create_tera(TEMPLATES.to_vec(), None).unwrap();
        let item_urn = &Urn::from("PackageA/ModuleB/FamilyC/Item");
        for &shape in ["Icon", "IconCard", "IconGroup", "Group"].iter() {
            for &snippet_mode in [&Remote, &Local].iter() {
                let generator = ElementSnippetTask {
                    remote_url: "a remote url".to_string(),
                    package_urn: "PackageA".to_string(),
                    item_urn: String::from(&item_urn.value),
                    path_to_base: String::from(&item_urn.path_to_base),
                    element_shape: String::from(shape),
                    snippet_mode: snippet_mode.clone(),
                    procedure_name: format!("Item{}", String::from(shape)),
                    variable_name: "item".to_string(),
                    primary_label: "Item".to_string(),
                    technical_label: None,
                    description_label: None,
                    template: get_default_template_item_snippet(),
                    full_destination_source_path: format!(
                        "target/tests/element_snippet/source.{}.puml",
                        shape
                    ),
                    full_destination_image_path: format!(
                        "target/tests/element_snippet/source.{}.png",
                        shape
                    ),
                    properties: HashMap::default(),
                };
                generator.cleanup(&vec![CleanupScope::All]).unwrap();
                generator.render_atomic_templates(tera).unwrap();
                let content = read_to_string(generator.full_destination_source_path).unwrap();
                if snippet_mode.eq(&Remote) {
                    assert!(content.contains(r##"!global $LIB_BASE_LOCATION="a remote url""##));
                } else {
                    assert!(!content.contains(r##"!global $LIB_BASE_LOCATION="a remote url""##));
                }
                assert!(content.contains(r##"include('PackageA/ModuleB/FamilyC/Item')"##));
                assert!(content.contains(format!("{}(", generator.procedure_name).as_str()));
            }
        }
    }

    #[test]
    fn test_render_templates_custom() {
        let tera = &create_tera(TEMPLATES.to_vec(), Some("test/tera/**".to_string())).unwrap();
        let item_urn = &Urn::from("PackageA/ModuleB/FamilyC/CustomItem");
        for &snippet_mode in [&Remote, &Local].iter() {
            let generator = ElementSnippetTask {
                remote_url: "a remote url".to_string(),
                package_urn: "PackageA".to_string(),
                item_urn: String::from(&item_urn.value),
                path_to_base: String::from(&item_urn.path_to_base),
                element_shape: format!("Custom"),
                snippet_mode: snippet_mode.clone(),
                procedure_name: format!("ItemCustom"),
                variable_name: "item".to_string(),
                primary_label: "Item".to_string(),
                technical_label: None,
                description_label: None,
                template: "custom_item_snippet.tera".to_string(),
                full_destination_source_path: format!(
                    "target/tests/element_snippet/source.Custom.puml"
                ),
                full_destination_image_path: format!(
                    "target/tests/element_snippet/source.Custom.png"
                ),
                properties: HashMap::default(),
            };
            generator.cleanup(&vec![CleanupScope::All]).unwrap();
            generator.render_atomic_templates(tera).unwrap();
            let content = read_to_string(generator.full_destination_source_path).unwrap();
            if snippet_mode.eq(&Remote) {
                assert!(content.contains(r##"!global $LIB_BASE_LOCATION="a remote url""##));
            } else {
                assert!(!content.contains(r##"!global $LIB_BASE_LOCATION="a remote url""##));
            }
            assert!(content.contains(r##"include('PackageA/ModuleB/FamilyC/CustomItem')"##));
            assert!(content.contains(format!("{}(", generator.procedure_name).as_str()));
        }
    }
}
