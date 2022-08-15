use crate::constants::{
    TEMPLATE_ITEM_DOCUMENTATION, TEMPLATE_ITEM_SNIPPET, TEMPLATE_ITEM_SOURCE,
    TEMPLATE_LIBRARY_BOOTSTRAP, TEMPLATE_LIBRARY_DOCUMENTATION, TEMPLATE_MODULE_DOCUMENTATION,
    TEMPLATE_PACKAGE_BOOTSTRAP, TEMPLATE_PACKAGE_DOCUMENTATION, TEMPLATE_PACKAGE_EMBEDDED,
    TEMPLATE_PACKAGE_EXAMPLE,
};

mod item_documentation;
mod item_snippet;
mod item_source;
mod library_bootstrap;
mod library_documentation;
mod module_documentation;
mod package_bootstrap;
mod package_documentation;
mod package_embedded;
mod package_example;

pub const TEMPLATES: &[(&str, &str); 10] = &[
    (TEMPLATE_ITEM_DOCUMENTATION, item_documentation::TEMPLATE),
    (TEMPLATE_ITEM_SNIPPET, item_snippet::TEMPLATE),
    (TEMPLATE_ITEM_SOURCE, item_source::TEMPLATE),
    (TEMPLATE_LIBRARY_BOOTSTRAP, library_bootstrap::TEMPLATE),
    (
        TEMPLATE_LIBRARY_DOCUMENTATION,
        library_documentation::TEMPLATE,
    ),
    (
        TEMPLATE_MODULE_DOCUMENTATION,
        module_documentation::TEMPLATE,
    ),
    (TEMPLATE_PACKAGE_BOOTSTRAP, package_bootstrap::TEMPLATE),
    (TEMPLATE_PACKAGE_EMBEDDED, package_embedded::TEMPLATE),
    (
        TEMPLATE_PACKAGE_DOCUMENTATION,
        package_documentation::TEMPLATE,
    ),
    (TEMPLATE_PACKAGE_EXAMPLE, package_example::TEMPLATE),
];
