use serde::{Deserialize, Serialize};

use crate::cmd::library::manifest::example::Example;
use crate::cmd::library::manifest::module::Module;
use crate::cmd::library::manifest::package::rendering::PackageRendering;
use crate::cmd::library::manifest::package::templates::PackageTemplates;
use crate::urn::Urn;

mod templates {
    use serde::{Deserialize, Serialize};

    use crate::constants::{
        get_default_template_package_bootstrap, get_default_template_package_documentation,
        get_default_template_package_embedded,
    };

    #[derive(Serialize, Deserialize, Debug)]
    pub struct PackageTemplates {
        /// The template used to generate `<library>/<package>/bootstrap.puml`.
        #[serde(default = "get_default_template_package_bootstrap")]
        pub bootstrap: String,
        /// The template name used to generate `<library>/<package>/{single,full}.puml`.
        #[serde(default = "get_default_template_package_embedded")]
        pub embedded: String,
        /// The template used to generate `<library>/<package>/README.md`.
        #[serde(default = "get_default_template_package_documentation")]
        pub documentation: String,
    }

    impl Default for PackageTemplates {
        fn default() -> Self {
            PackageTemplates {
                bootstrap: get_default_template_package_bootstrap(),
                embedded: get_default_template_package_embedded(),
                documentation: get_default_template_package_documentation(),
            }
        }
    }
}

mod rendering {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Default)]
    pub struct PackageRendering {
        /// When true skip the generation of `<library>/<package>/{single,full}.puml`.
        #[serde(default)]
        pub skip_embedded: bool,
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
    /// The customization of the rendered resources.
    #[serde(default)]
    pub rendering: PackageRendering,
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
                embedded: templates_embedded_path
        "#;
        let package: Package = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(package.urn.value, "package/urn");
        assert!(package.modules.is_empty());
        assert!(package.examples.is_empty());
        assert_eq!(package.templates.bootstrap, "templates_bootstrap_path");
        assert_eq!(package.templates.embedded, "templates_embedded_path");
        assert!(!package.templates.documentation.is_empty());
        assert!(!package.rendering.skip_embedded);
    }

    #[test]
    fn test_deserialized_rendering() {
        let yaml = r#"
            urn: package/urn
            rendering:
                skip_embedded: true
        "#;
        let package: Package = serde_yaml::from_str(yaml).unwrap();
        assert!(package.rendering.skip_embedded);
    }
}
