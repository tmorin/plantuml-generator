use heck::ToUpperCamelCase;
use serde::{Deserialize, Serialize};

use crate::urn::Urn;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Icon {
    Source {
        /// The name of stereotype.
        source: String,
    },
    Reference {
        /// The name of stereotype.
        urn: Urn,
    },
}

impl Icon {
    pub fn get_icon_path(&self, item_urn: &Urn, icon_format: &str) -> String {
        match &self {
            Icon::Source { .. } => format!("{}.{}", item_urn.value, icon_format),
            Icon::Reference { urn } => format!("{}.{}", urn.value, icon_format),
        }
    }
    pub fn get_sprite_name(&self, urn: &Urn, size: &str) -> String {
        match &self {
            Icon::Source { .. } => format!("{}{}", urn.name, size.to_upper_camel_case()),
            Icon::Reference { urn } => format!("{}{}", urn.name, size.to_upper_camel_case()),
        }
    }
    pub fn get_sprite_image_path(&self, urn: &Urn, size: &str) -> String {
        match &self {
            Icon::Source { .. } => format!("{}{}.png", urn.value, size.to_upper_camel_case()),
            Icon::Reference { urn } => format!("{}{}.png", urn.value, size.to_upper_camel_case()),
        }
    }
    pub fn get_sprite_value_path(&self, urn: &Urn, size: &str) -> String {
        match &self {
            Icon::Source { .. } => format!("{}{}.puml", urn.value, size.to_upper_camel_case()),
            Icon::Reference { urn } => format!("{}{}.puml", urn.value, size.to_upper_camel_case()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialized_source() {
        let yaml = r#"
            type: Source
            source: the_source_path
        "#;
        let icon: Icon = serde_yaml::from_str(yaml).unwrap();
        match icon {
            Icon::Source { source } => assert_eq!(source, "the_source_path"),
            Icon::Reference { .. } => unreachable!(),
        }
    }

    #[test]
    fn test_deserialized_reference() {
        let yaml = r#"
            type: Reference
            urn: the_reference
        "#;
        let icon: Icon = serde_yaml::from_str(yaml).unwrap();
        match icon {
            Icon::Source { .. } => unreachable!(),
            Icon::Reference { urn } => assert_eq!(urn.value, "the_reference"),
        }
    }
}
