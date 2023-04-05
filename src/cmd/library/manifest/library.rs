use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::cmd::library::manifest::library::customization::Customization;
use crate::cmd::library::manifest::library::templates::LibraryTemplates;
use crate::cmd::library::manifest::package::Package;

pub mod customization {
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use crate::constants::get_default_font_color_light;
    use crate::constants::get_default_font_size_lg;
    use crate::constants::get_default_font_size_md;
    use crate::constants::get_default_font_size_sm;
    use crate::constants::get_default_font_size_xs;
    use crate::constants::get_default_icon_format;
    use crate::constants::get_default_icon_height;
    use crate::constants::get_default_msg_width_max;
    use crate::constants::get_default_text_width_max;
    use crate::constants::{get_default_font_color, SPRITE_LG, SPRITE_MD, SPRITE_SM, SPRITE_XS};

    #[derive(Serialize, Deserialize, Debug, JsonSchema)]
    pub struct Customization {
        /// The image format used to generate icons.
        #[serde(default = "get_default_icon_format")]
        pub icon_format: String,
        /// The height of the icons.
        #[serde(default = "get_default_icon_height")]
        pub icon_height: u32,
        /// The max width for text.
        #[serde(default = "get_default_text_width_max")]
        pub text_width_max: u32,
        /// The max width for message.
        #[serde(default = "get_default_msg_width_max")]
        pub msg_width_max: u32,
        /// The extra-small size value.
        #[serde(default = "get_default_font_size_xs")]
        pub font_size_xs: u32,
        /// The small size value.
        #[serde(default = "get_default_font_size_sm")]
        pub font_size_sm: u32,
        /// The medium size value.
        #[serde(default = "get_default_font_size_md")]
        pub font_size_md: u32,
        /// The large size value.
        #[serde(default = "get_default_font_size_lg")]
        pub font_size_lg: u32,
        /// The default font color.
        #[serde(default = "get_default_font_color")]
        pub font_color: String,
        /// A lighter font color.
        #[serde(default = "get_default_font_color_light")]
        pub font_color_light: String,
    }

    impl Customization {
        pub fn list_sprite_sizes(&self) -> Vec<(&str, u32)> {
            vec![
                (SPRITE_XS, self.font_size_xs),
                (SPRITE_SM, self.font_size_sm),
                (SPRITE_MD, self.font_size_md),
                (SPRITE_LG, self.font_size_lg),
            ]
        }
    }

    impl Default for Customization {
        fn default() -> Self {
            Customization {
                icon_format: get_default_icon_format(),
                icon_height: get_default_icon_height(),
                text_width_max: get_default_text_width_max(),
                msg_width_max: get_default_msg_width_max(),
                font_size_xs: get_default_font_size_xs(),
                font_size_sm: get_default_font_size_sm(),
                font_size_md: get_default_font_size_md(),
                font_size_lg: get_default_font_size_lg(),
                font_color: get_default_font_color(),
                font_color_light: get_default_font_color_light(),
            }
        }
    }
}

mod templates {
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use crate::constants::{
        get_default_template_library_bootstrap, get_default_template_library_documentation,
        get_default_template_library_summary,
    };

    #[derive(Serialize, Deserialize, Debug, JsonSchema)]
    pub struct LibraryTemplates {
        /// The template name used to generate `<library>/bootstrap.puml`. */
        #[serde(default = "get_default_template_library_bootstrap")]
        pub bootstrap: String,
        /// The template name used to generate `<library>/README.md`. */
        #[serde(default = "get_default_template_library_documentation")]
        pub documentation: String,
        /// The template name used to generate `<library>/SUMMARY.md`. */
        #[serde(default = "get_default_template_library_summary")]
        pub summary: String,
    }

    impl Default for LibraryTemplates {
        fn default() -> Self {
            LibraryTemplates {
                bootstrap: get_default_template_library_bootstrap(),
                documentation: get_default_template_library_documentation(),
                summary: get_default_template_library_summary(),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Library {
    /// The name of the library.
    pub name: String,
    /// The URL used to fetched the library remotely.
    pub remote_url: String,
    /// The packages provided by the library.
    #[serde(default)]
    pub packages: Vec<Package>,
    /// The definition of templates.
    #[serde(default)]
    pub templates: LibraryTemplates,
    /// The configuration of he library.
    #[serde(default)]
    pub customization: Customization,
    /// An optional tera directory.
    #[serde(default)]
    pub tera_discovery_pattern: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialized() {
        let yaml = r#"
            name: testlib
            remote_url: testlib.local:3000/distribution
            customization:
                icon_format: svg
        "#;
        let library: Library = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(library.name, "testlib");
        assert!(library.packages.is_empty());
        assert_eq!(library.templates.bootstrap, "library_bootstrap.tera");
        assert_eq!(
            library.templates.documentation,
            "library_documentation.tera"
        );
        assert_eq!(library.customization.icon_format, String::from("svg"));
        assert_eq!(library.customization.font_size_xs, 10);
    }

    #[test]
    fn test_deserialized_templates() {
        let yaml = r#"
            name: testlib
            remote_url: testlib.local:3000/distribution
            templates:
                bootstrap: dummy_path
        "#;
        let library: Library = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(library.name, "testlib");
        assert!(library.packages.is_empty());
        assert_eq!(library.templates.bootstrap, "dummy_path");
        assert_eq!(
            library.templates.documentation,
            "library_documentation.tera"
        );
    }

    #[test]
    fn test_deserialized_packages() {
        let yaml = r#"
            name: testlib
            remote_url: testlib.local:3000/distribution
            packages:
                - urn: testlib/packagetest0
                - urn: testlib/packagetest1
        "#;
        let library: Library = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(library.name, "testlib");
        assert_eq!(library.packages.len(), 2);
    }
}
