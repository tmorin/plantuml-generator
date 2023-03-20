use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::constants::get_default_group_element_stereotype;
use crate::constants::get_default_icon_card_element_stereotype;
use crate::constants::get_default_icon_element_stereotype;
use crate::constants::get_default_icon_group_element_stereotype;
use crate::urn::Urn;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Shape {
    Icon {
        /// The name of stereotype.
        #[serde(default = "get_default_icon_element_stereotype")]
        stereotype_name: String,
        /// A set of custom properties.
        #[serde(default)]
        properties: HashMap<String, Value>,
    },
    IconCard {
        /// The name of stereotype.
        #[serde(default = "get_default_icon_card_element_stereotype")]
        stereotype_name: String,
        /// A set of custom properties.
        #[serde(default)]
        properties: HashMap<String, Value>,
    },
    IconGroup {
        /// The name of stereotype.
        #[serde(default = "get_default_icon_group_element_stereotype")]
        stereotype_name: String,
        /// A set of custom properties.
        #[serde(default)]
        properties: HashMap<String, Value>,
    },
    Group {
        /// The name of stereotype.
        #[serde(default = "get_default_group_element_stereotype")]
        stereotype_name: String,
        /// A set of custom properties.
        #[serde(default)]
        properties: HashMap<String, Value>,
    },
    Custom {
        /// A set of custom properties.
        #[serde(default)]
        properties: HashMap<String, Value>,
    },
}

impl Shape {
    pub fn get_name(&self) -> String {
        String::from(match self {
            Shape::Icon { .. } => "Icon",
            Shape::IconCard { .. } => "IconCard",
            Shape::IconGroup { .. } => "IconGroup",
            Shape::Group { .. } => "Group",
            Shape::Custom { .. } => "Custom",
        })
    }
    pub fn get_element_name(&self, item_urn: &Urn) -> String {
        match self {
            Shape::Icon { .. } => item_urn.name.to_string(),
            Shape::IconCard { .. } => format!("{}{}", item_urn.name, "Card"),
            Shape::IconGroup { .. } => format!("{}{}", item_urn.name, "Group"),
            Shape::Group { .. } => item_urn.name.to_string(),
            Shape::Custom { .. } => item_urn.name.to_string(),
        }
    }
    pub fn get_local_snippet_image_path(&self, item_urn: &Urn, icon_format: &str) -> String {
        format!(
            "{}/{}.Local.{}",
            item_urn.get_parent().value,
            self.get_element_name(item_urn),
            icon_format
        )
    }
    pub fn get_local_snippet_puml_path(&self, item_urn: &Urn) -> String {
        format!(
            "{}/{}.Local.puml",
            item_urn.get_parent().value,
            self.get_element_name(item_urn),
        )
    }
    pub fn get_remote_snippet_image_path(&self, item_urn: &Urn, icon_format: &str) -> String {
        format!(
            "{}/{}.Remote.{}",
            item_urn.get_parent().value,
            self.get_element_name(item_urn),
            icon_format
        )
    }
    pub fn get_remote_snippet_puml_path(&self, item_urn: &Urn) -> String {
        format!(
            "{}/{}.Remote.puml",
            item_urn.get_parent().value,
            self.get_element_name(item_urn),
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Element {
    /// The shape of the element and its related configuration.
    pub shape: Shape,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialized_icon() {
        let yaml = r#"
            shape:
                type: Icon
                stereotype_name: CustomStereotype
        "#;
        let element: Element = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(element.shape.get_name(), "Icon");
        assert_eq!(
            element.shape.get_element_name(&Urn::from("p/m/f/Test")),
            "Test"
        );
        match element.shape {
            Shape::Icon {
                stereotype_name,
                properties: _,
            } => assert_eq!(stereotype_name, "CustomStereotype"),
            _ => panic!("should not reach this point"),
        };
    }

    #[test]
    fn test_deserialized_icon_card() {
        let yaml = r#"
            shape:
                type: IconCard
        "#;
        let element: Element = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(element.shape.get_name(), "IconCard");
        assert_eq!(
            element.shape.get_element_name(&Urn::from("p/m/f/Test")),
            "TestCard"
        );
        match element.shape {
            Shape::IconCard {
                stereotype_name: custom_stereotype,
                properties: _,
            } => assert_eq!(
                custom_stereotype,
                get_default_icon_card_element_stereotype()
            ),
            _ => panic!("should not reach this point"),
        };
    }

    #[test]
    fn test_deserialized_icon_group() {
        let yaml = r#"
            shape:
                type: IconGroup
        "#;
        let element: Element = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(element.shape.get_name(), "IconGroup");
        assert_eq!(
            element.shape.get_element_name(&Urn::from("p/m/f/Test")),
            "TestGroup"
        );
        match element.shape {
            Shape::IconGroup {
                stereotype_name: custom_stereotype,
                properties: _,
            } => assert_eq!(
                custom_stereotype,
                get_default_icon_group_element_stereotype()
            ),
            _ => panic!("should not reach this point"),
        };
    }

    #[test]
    fn test_deserialized_group() {
        let yaml = r#"
            shape:
                type: Group
        "#;
        let element: Element = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(element.shape.get_name(), "Group");
        assert_eq!(
            element.shape.get_element_name(&Urn::from("p/m/f/Test")),
            "Test"
        );
        match element.shape {
            Shape::Group {
                stereotype_name: custom_stereotype,
                properties: _,
            } => assert_eq!(custom_stereotype, get_default_group_element_stereotype()),
            _ => panic!("should not reach this point"),
        };
    }

    #[test]
    fn test_deserialized_custom() {
        let yaml = r#"
            shape:
                type: Custom
                properties:
                    keyA: valueA
        "#;
        let element: Element = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(element.shape.get_name(), "Custom");
        assert_eq!(
            element.shape.get_element_name(&Urn::from("p/m/f/Test")),
            "Test"
        );
        match element.shape {
            Shape::Custom { properties, .. } => {
                assert_eq!(properties.get("keyA").unwrap(), "valueA")
            }
            _ => panic!("should not reach this point"),
        };
    }
}
