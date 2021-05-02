use heck::SnakeCase;
use serde::{Deserialize, Serialize};

use crate::urn::Urn;

#[derive(Serialize, Deserialize, Debug)]
pub struct Example {
    /// The name of the example.
    pub name: String,
    /// The name of the template to render the `.puml` file.
    pub template: String,
}

impl Example {
    pub fn get_source_path(&self, package_urn: &Urn) -> String {
        format!("{}/{}.puml", package_urn.value, self.name.to_snake_case())
    }
    pub fn get_destination_path(&self, package_urn: &Urn, icon_format: &str) -> String {
        format!(
            "{}/{}.{}",
            package_urn.value,
            self.name.to_snake_case(),
            icon_format
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialized() {
        let yaml = r#"
            name: example
            template: example_template
        "#;
        let example: Example = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(example.name, "example");
        assert_eq!(example.template, "example_template");
    }
}
