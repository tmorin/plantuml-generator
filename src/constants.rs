pub const ICON_ELEMENT_STEREOTYPE: &str = "IconElement";

pub fn get_default_icon_element_stereotype() -> String {
    ICON_ELEMENT_STEREOTYPE.to_string()
}

pub const ICON_CARD_ELEMENT_STEREOTYPE: &str = "IconCardElement";

pub fn get_default_icon_card_element_stereotype() -> String {
    ICON_CARD_ELEMENT_STEREOTYPE.to_string()
}

pub const ICON_GROUP_ELEMENT_STEREOTYPE: &str = "IconGroupElement";

pub fn get_default_icon_group_element_stereotype() -> String {
    ICON_GROUP_ELEMENT_STEREOTYPE.to_string()
}

pub const GROUP_ELEMENT_STEREOTYPE: &str = "GroupElement";

pub fn get_default_group_element_stereotype() -> String {
    GROUP_ELEMENT_STEREOTYPE.to_string()
}

pub const TEXT_WIDTH_MAX: u32 = 200;

pub fn get_default_text_width_max() -> u32 {
    TEXT_WIDTH_MAX
}

pub const MSG_WIDTH_MAX: u32 = 150;

pub fn get_default_msg_width_max() -> u32 {
    MSG_WIDTH_MAX
}

pub const FONT_SIZE_XS: u32 = 10;

pub fn get_default_font_size_xs() -> u32 {
    FONT_SIZE_XS
}

pub const FONT_SIZE_SM: u32 = 12;

pub fn get_default_font_size_sm() -> u32 {
    FONT_SIZE_SM
}

pub const FONT_SIZE_MD: u32 = 16;

pub fn get_default_font_size_md() -> u32 {
    FONT_SIZE_MD
}

pub const FONT_SIZE_LG: u32 = 20;

pub fn get_default_font_size_lg() -> u32 {
    FONT_SIZE_LG
}

pub const ICON_HEIGHT: u32 = 50;

pub fn get_default_icon_height() -> u32 {
    ICON_HEIGHT
}

pub const ICON_FORMAT: &str = "png";

pub fn get_default_icon_format() -> String {
    ICON_FORMAT.to_string()
}

pub const FONT_COLOR: &str = "#212121";

pub fn get_default_font_color() -> String {
    FONT_COLOR.to_string()
}

pub const FONT_COLOR_LIGHT: &str = "#757575";

pub fn get_default_font_color_light() -> String {
    FONT_COLOR_LIGHT.to_string()
}

pub const SOURCE_DIRECTORY: &str = ".";

pub fn get_default_source_directory() -> String {
    SOURCE_DIRECTORY.to_string()
}

pub const OUTPUT_DIRECTORY: &str = "distribution";

pub fn get_default_output_directory() -> String {
    OUTPUT_DIRECTORY.to_string()
}

pub const CACHE_DIRECTORY: &str = ".cache";

pub fn get_default_cache_directory() -> String {
    CACHE_DIRECTORY.to_string()
}

pub const TERA_DISCOVERY_PATTERN: &str = "templates/**";

pub fn get_default_tera_discovery_pattern() -> String {
    TERA_DISCOVERY_PATTERN.to_string()
}

pub const PLANTUML_VERSION: &str = "1.2024.7";

pub fn get_default_plantuml_version() -> String {
    PLANTUML_VERSION.to_string()
}

pub const PLANTUML_JAR: &str = ".cache/plantuml-1.2024.7.jar";

pub fn get_default_plantuml_jar() -> String {
    PLANTUML_JAR.to_string()
}

pub const JAVA_BINARY: &str = "java";

pub fn get_default_java_binary() -> String {
    JAVA_BINARY.to_string()
}

pub const INKSCAPE_BINARY: &str = "inkscape";

pub fn get_default_inkscape_binary() -> String {
    INKSCAPE_BINARY.to_string()
}

pub const SPRITE_XS: &str = "xs";
pub const SPRITE_SM: &str = "sm";
pub const SPRITE_MD: &str = "md";
pub const SPRITE_LG: &str = "lg";
pub const SPRITES: [&str; 4] = [SPRITE_XS, SPRITE_SM, SPRITE_MD, SPRITE_LG];

pub const TEMPLATE_ITEM_DOCUMENTATION: &str = "item_documentation.tera";

pub fn get_default_template_item_documentation() -> String {
    TEMPLATE_ITEM_DOCUMENTATION.to_string()
}

pub const TEMPLATE_ITEM_SOURCE: &str = "item_source.tera";

pub fn get_default_template_item_source() -> String {
    TEMPLATE_ITEM_SOURCE.to_string()
}

pub const TEMPLATE_ITEM_SNIPPET: &str = "item_snippet.tera";

pub fn get_default_template_item_snippet() -> String {
    TEMPLATE_ITEM_SNIPPET.to_string()
}

pub const TEMPLATE_LIBRARY_BOOTSTRAP: &str = "library_bootstrap.tera";

pub fn get_default_template_library_bootstrap() -> String {
    TEMPLATE_LIBRARY_BOOTSTRAP.to_string()
}

pub const TEMPLATE_LIBRARY_DOCUMENTATION: &str = "library_documentation.tera";

pub fn get_default_template_library_documentation() -> String {
    TEMPLATE_LIBRARY_DOCUMENTATION.to_string()
}

pub const TEMPLATE_LIBRARY_SUMMARY: &str = "library_summary.tera";

pub fn get_default_template_library_summary() -> String {
    TEMPLATE_LIBRARY_SUMMARY.to_string()
}

pub const TEMPLATE_MODULE_DOCUMENTATION: &str = "module_documentation.tera";

pub fn get_default_template_module_documentation() -> String {
    TEMPLATE_MODULE_DOCUMENTATION.to_string()
}

pub const TEMPLATE_PACKAGE_BOOTSTRAP: &str = "package_bootstrap.tera";

pub fn get_default_template_package_bootstrap() -> String {
    TEMPLATE_PACKAGE_BOOTSTRAP.to_string()
}

pub const TEMPLATE_PACKAGE_EMBEDDED: &str = "package_embedded.tera";

pub fn get_default_template_package_embedded() -> String {
    TEMPLATE_PACKAGE_EMBEDDED.to_string()
}

pub const TEMPLATE_PACKAGE_DOCUMENTATION: &str = "package_documentation.tera";

pub fn get_default_template_package_documentation() -> String {
    TEMPLATE_PACKAGE_DOCUMENTATION.to_string()
}

pub const TEMPLATE_PACKAGE_EXAMPLE: &str = "package_example.tera";

pub const WORKSPACE_MANIFEST: &str = ".pgen-workspace.yaml";

pub fn get_default_workspace_manifest() -> String {
    WORKSPACE_MANIFEST.to_string()
}
