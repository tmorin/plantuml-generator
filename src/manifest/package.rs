use serde::{Deserialize, Serialize};

use crate::manifest::example::Example;
use crate::manifest::module::Module;
use crate::manifest::package::templates::PackageTemplates;
use crate::urn::Urn;

mod templates {
    use serde::{Deserialize, Serialize};

    use crate::constants::{
        get_default_template_package_bootstrap, get_default_template_package_documentation,
    };

    #[derive(Serialize, Deserialize, Debug)]
    pub struct PackageTemplates {
        /// The template used to generate `<library>/<package>/bootstrap.puml`.
        #[serde(default = "get_default_template_package_bootstrap")]
        pub bootstrap: String,
        /// The template used to generate `<library>/<package>/README.md`.
        #[serde(default = "get_default_template_package_documentation")]
        pub documentation: String,
    }

    impl Default for PackageTemplates {
        fn default() -> Self {
            PackageTemplates {
                bootstrap: get_default_template_package_bootstrap(),
                documentation: get_default_template_package_documentation(),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    /// The URN of the package.
    pub urn: Urn,
    /// The modules provided by the package.
    #[serde(default)]
    pub modules: Vec<Module>,
    /// The example provided by the package.
    #[serde(default)]
    pub examples: Vec<Example>,
    /// The definition of the templates.
    #[serde(default)]
    pub templates: PackageTemplates,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialized() {
        let yaml = r#"
            urn: package/urn
            templates:
                bootstrap: templates_bootstrap_path
        "#;
        let package: Package = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(package.urn.value, "package/urn");
        assert!(package.modules.is_empty());
        assert!(package.examples.is_empty());
        assert_eq!(package.templates.bootstrap, "templates_bootstrap_path");
        assert!(!package.templates.documentation.is_empty());
    }
}
