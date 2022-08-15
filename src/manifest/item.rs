use serde::{Deserialize, Serialize};

use crate::manifest::element::Element;
use crate::manifest::icon::Icon;
use crate::manifest::item::templates::ItemTemplates;
use crate::urn::Urn;

mod templates {
    use serde::{Deserialize, Serialize};

    use crate::constants::{
        get_default_template_item_documentation, get_default_template_item_snippet,
        get_default_template_item_source,
    };

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ItemTemplates {
        /// The template used to generate `<library>/<package>/<module>/<Item>.md`.
        #[serde(default = "get_default_template_item_documentation")]
        pub documentation: String,
        /// The template used to generate `<library>/<package>/<module>/<Item>.puml`.
        #[serde(default = "get_default_template_item_source")]
        pub source: String,
        /// The template used to generate `<library>/<package>/<module>/<element>.snippet.[local|remote].puml`.
        #[serde(default = "get_default_template_item_snippet")]
        pub snippet: String,
    }

    impl Default for ItemTemplates {
        fn default() -> Self {
            ItemTemplates {
                documentation: get_default_template_item_documentation(),
                source: get_default_template_item_source(),
                snippet: get_default_template_item_snippet(),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    /// The URN of the Item.
    pub urn: Urn,
    /// The family of the Item.
    #[serde(default)]
    pub family: Option<String>,
    /// The icon of the Item.
    #[serde(default)]
    pub icon: Option<Icon>,
    /// The elements of the Item.
    #[serde(default)]
    pub elements: Vec<Element>,
    /// The definition of the templates.
    #[serde(default)]
    pub templates: ItemTemplates,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialized() {
        let yaml = r#"
            urn: item_urn
            family: item_family
            templates:
                snippet: item_templates_snippet
        "#;
        let item: Item = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(item.urn.value, "item_urn");
        assert_eq!(item.family.unwrap(), "item_family");
        assert!(item.elements.is_empty());
        assert!(!item.templates.source.is_empty());
        assert!(!item.templates.documentation.is_empty());
        assert_eq!(item.templates.snippet, "item_templates_snippet");
    }
}
