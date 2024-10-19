use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::cmd::library::manifest::item::Item;
use crate::cmd::library::manifest::module::templates::ModuleTemplates;
use crate::urn::Urn;

mod templates {
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use crate::constants::get_default_template_module_documentation;

    #[derive(Serialize, Deserialize, Debug, JsonSchema)]
    pub struct ModuleTemplates {
        /// The template name used to generate `<library>/<package>/<module>/README.md`.
        #[serde(default = "get_default_template_module_documentation")]
        pub documentation: String,
    }

    impl Default for ModuleTemplates {
        fn default() -> Self {
            ModuleTemplates {
                documentation: get_default_template_module_documentation(),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Module {
    /// The URN of the module.
    pub urn: Urn,
    /// The items provided by the module.
    #[serde(default)]
    pub items: Vec<Item>,
    /// The items provided by the module.
    #[serde(default)]
    pub templates: ModuleTemplates,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialized() {
        let yaml = r#"
            urn: module/urn
            templates:
                documentation: templates_documentation_path
        "#;
        let module: Module = serde_yaml_ok::from_str(yaml).unwrap();
        assert_eq!(module.urn.value, "module/urn");
        assert!(module.items.is_empty());
        assert_eq!(
            module.templates.documentation,
            "templates_documentation_path"
        );
    }
}
