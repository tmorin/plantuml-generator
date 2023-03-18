use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::error::Error;
use crate::manifest::library::Library;
use crate::result::Result;
use crate::utils::{create_parent_directory, delete_file};

#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    /// The URN of the item.
    item_urn: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Module {
    /// The URN of the module.
    module_urn: String,
    /// The items of the library.
    items: Vec<Item>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    /// The URN of the package.
    package_urn: String,
    /// The modules of the library.
    modules: Vec<Module>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LibrarySummaryTask {
    /// The name of the library.
    library_name: String,
    /// The packages of the library.
    packages: Vec<Package>,
    /// The path to the output directory.
    output_directory: String,
    /// The name of the Tera template
    template: String,
}

impl LibrarySummaryTask {
    pub fn create(config: &Config, library: &Library) -> Result<LibrarySummaryTask> {
        Ok(LibrarySummaryTask {
            library_name: library.name.clone(),
            packages: library
                .packages
                .iter()
                .map(|p| Package {
                    package_urn: p.urn.value.clone(),
                    modules: p
                        .modules
                        .iter()
                        .map(|m| Module {
                            module_urn: m.urn.value.clone(),
                            items: m
                                .items
                                .iter()
                                .map(|i| Item {
                                    item_urn: i.urn.value.clone(),
                                })
                                .collect(),
                        })
                        .collect(),
                })
                .collect(),
            output_directory: config.output_directory.clone(),
            template: library.templates.summary.clone(),
        })
    }
    fn get_relative_destination_path(&self) -> Box<Path> {
        Box::from(Path::new("SUMMARY.md"))
    }
    fn get_full_destination_path(&self) -> Box<Path> {
        Path::new(&self.output_directory)
            .join(self.get_relative_destination_path())
            .into_boxed_path()
    }
}

impl Task for LibrarySummaryTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!("{} - LibrarySummaryTask - cleanup", self.library_name);
        delete_file(self.get_full_destination_path().as_ref())?;
        Ok(())
    }

    fn render_atomic_templates(&self, _tera: &Tera) -> Result<()> {
        log::debug!(
            "{} - LibrarySummaryTask - render templates",
            self.library_name
        );

        let destination_path = self.get_full_destination_path();

        // skip early when generation not required
        if destination_path.exists() {
            return Ok(());
        }

        // create the destination directory
        create_parent_directory(&destination_path)?;

        // create the destination file
        let destination_file = File::create(&destination_path).map_err(|e| {
            Error::Cause(
                "unable to create the destination file".to_string(),
                Box::from(e),
            )
        })?;

        let mut context = Context::new();
        context.insert("data", &self);
        _tera
            .render_to(&self.template, &context, destination_file)
            .map_err(|e| Error::Cause(format!("unable to render {}", &self.template), Box::from(e)))
    }
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use crate::cmd::library::generate::templates::TEMPLATES;
    use crate::constants::get_default_template_library_summary;
    use crate::tera::create_tera;
    use crate::urn::Urn;

    use super::*;

    #[test]
    fn test_template() {
        let tera = &create_tera(TEMPLATES.to_vec(), None).unwrap();
        let generator = LibrarySummaryTask {
            library_name: "a library".to_string(),
            packages: vec![
                Package {
                    package_urn: Urn::from("aws-q1-2022").value,
                    modules: vec![
                        Module {
                            module_urn: Urn::from("aws-q1-2022/Architecture").value,
                            items: vec![
                                Item {
                                    item_urn: Urn::from(
                                        "aws-q1-2022/Architecture/Analytics/AmazonAthena",
                                    )
                                    .value,
                                },
                                Item {
                                    item_urn: Urn::from(
                                        "aws-q1-2022/Architecture/Analytics/AmazonCloudSearch",
                                    )
                                    .value,
                                },
                                Item {
                                    item_urn: Urn::from(
                                        "aws-q1-2022/Architecture/Analytics/AmazonEmr",
                                    )
                                    .value,
                                },
                            ],
                        },
                        Module {
                            module_urn: Urn::from("aws-q1-2022/Category").value,
                            items: vec![],
                        },
                    ],
                },
                Package {
                    package_urn: Urn::from("aws-q2-2022").value,
                    modules: vec![],
                },
            ],
            output_directory: "target/tests/library_summary_generator".to_string(),
            template: get_default_template_library_summary(),
        };
        generator.cleanup(&[CleanupScope::All]).unwrap();
        generator.render_atomic_templates(tera).unwrap();
        let content = read_to_string(format!("{}/SUMMARY.md", generator.output_directory)).unwrap();
        assert!(content.contains("[Presentation](README.md)"));
        assert!(content.contains("# aws-q1-2022"));
        assert!(content.contains("- [Presentation](aws-q1-2022/README.md)"));
        assert!(
            content.contains("- [aws-q1-2022/Architecture](aws-q1-2022/Architecture/README.md)")
        );
        assert!(content.contains("- [aws-q1-2022/Architecture/Analytics/AmazonAthena](aws-q1-2022/Architecture/Analytics/AmazonAthena.md)"));
    }
}
