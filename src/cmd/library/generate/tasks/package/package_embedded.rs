use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::error::Error;
use crate::manifest::package::Package;
use crate::result::Result;
use crate::utils::{create_parent_directory, delete_file, read_file_to_string};

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageEmbeddedTask {
    /// The expected output mode.
    mode: EmbeddedMode,
    /// The URN of the package.
    package_urn: String,
    /// The bootstrap content of the library.
    library_bootstrap_file: Option<String>,
    /// The bootstrap content of the package.
    package_bootstrap_file: Option<String>,
    /// The definition of the package's items.
    package_item_files: Vec<String>,
    /// The path to the output directory.
    output_directory: String,
    /// The name of the Tera template
    template: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum EmbeddedMode {
    Single,
    Full,
}

impl PackageEmbeddedTask {
    pub fn create(
        _config: &Config,
        _package: &Package,
        mode: EmbeddedMode,
    ) -> Result<PackageEmbeddedTask> {
        let output_directory = _config.output_directory.clone();

        let library_path = Path::new(&output_directory);
        let library_bootstrap_file = match mode {
            EmbeddedMode::Single => library_path
                .join("bootstrap.puml")
                .as_path()
                .to_str()
                .map(|str| str.to_string()),
            EmbeddedMode::Full => None,
        };

        let package_path = library_path.join(_package.urn.clone().to_string());
        let package_bootstrap_file = package_path
            .join("bootstrap.puml")
            .as_path()
            .to_str()
            .map(|str| str.to_string());

        let mut package_item_files: Vec<String> = Vec::new();
        for module in &_package.modules {
            for item in &module.items {
                let item_file = library_path
                    .join(format!("{}.puml", item.urn))
                    .as_path()
                    .to_str()
                    .map(|str| str.to_string());
                if item_file.is_some() {
                    package_item_files.push(item_file.unwrap_or_default());
                }
            }
        }

        Ok(PackageEmbeddedTask {
            mode,
            package_urn: _package.urn.clone().to_string(),
            library_bootstrap_file,
            package_bootstrap_file,
            package_item_files,
            output_directory,
            template: _package.templates.embedded.clone(),
        })
    }
    pub fn get_library_bootstrap(&self) -> String {
        read_file_to_string(&self.library_bootstrap_file)
    }
    pub fn get_package_bootstrap(&self) -> String {
        read_file_to_string(&self.package_bootstrap_file)
    }
    pub fn get_package_items(&self) -> String {
        self.package_item_files
            .iter()
            .map(|package_item_file| read_file_to_string(&Some(package_item_file.clone())))
            .map(|content| content.trim().to_string())
            .filter(|content| !content.is_empty())
            .collect::<Vec<String>>()
            .join("\n")
    }
    fn get_relative_destination_path(&self) -> Box<Path> {
        Box::from(Path::new(
            format!(
                "{}/{}.puml",
                self.package_urn,
                match self.mode {
                    EmbeddedMode::Single => "single",
                    EmbeddedMode::Full => "full",
                }
            )
                .as_str(),
        ))
    }
    fn get_embedded_destination_path(&self) -> Box<Path> {
        Path::new(&self.output_directory)
            .join(self.get_relative_destination_path())
            .into_boxed_path()
    }
}

impl Task for PackageEmbeddedTask {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::debug!("{} - PackageEmbeddedTask - cleanup", self.package_urn);
        delete_file(self.get_embedded_destination_path().as_ref())?;
        Ok(())
    }

    fn render_composed_templates(&self, _tera: &Tera) -> Result<()> {
        log::debug!(
            "{} - PackageEmbeddedTask - render templates",
            self.package_urn
        );

        let destination_path = self.get_embedded_destination_path();

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
        context.insert("library_bootstrap", &self.get_library_bootstrap());
        context.insert("package_bootstrap", &self.get_package_bootstrap());
        context.insert("package_items", &self.get_package_items());
        _tera
            .render_to(&self.template, &context, destination_file)
            .map_err(|e| Error::Cause(format!("unable to render {}", &self.template), Box::from(e)))
    }
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;
    use std::io::Write;

    use crate::cmd::library::generate::templates::TEMPLATES;
    use crate::constants::get_default_template_package_embedded;
    use crate::tera::create_tera;
    use crate::utils::delete_file_or_directory;

    use super::*;

    fn write_fixture_file(name: &str) {
        let path_as_string = format!(
            "target/tests/package_embedded_generator/package_urn/{}.txt",
            name
        );
        let path = Path::new(path_as_string.as_str());
        create_parent_directory(path).unwrap();
        let mut file = File::create(path).unwrap();
        file.write_all(name.as_bytes()).unwrap();
    }

    #[test]
    fn test_template_with_single() {
        let tera = &create_tera(TEMPLATES.to_vec(), None).unwrap();
        let task = PackageEmbeddedTask {
            mode: EmbeddedMode::Single,
            package_urn: "package_urn".to_string(),
            library_bootstrap_file: Some(
                "target/tests/package_embedded_generator/package_urn/library_bootstrap_file.txt"
                    .to_string(),
            ),
            package_bootstrap_file: Some(
                "target/tests/package_embedded_generator/package_urn/package_bootstrap_file.txt"
                    .to_string(),
            ),
            package_item_files: vec![
                "target/tests/package_embedded_generator/package_urn/package_item_file_a.txt"
                    .to_string(),
                "target/tests/package_embedded_generator/package_urn/package_item_file_b.txt"
                    .to_string(),
            ],
            output_directory: "target/tests/package_embedded_generator".to_string(),
            template: get_default_template_package_embedded(),
        };

        delete_file_or_directory(task.output_directory.as_ref()).unwrap();
        write_fixture_file("library_bootstrap_file");
        write_fixture_file("package_bootstrap_file");
        write_fixture_file("package_item_file_a");
        write_fixture_file("package_item_file_b");

        task.cleanup(&[CleanupScope::All]).unwrap();
        task.render_composed_templates(tera).unwrap();

        let content =
            read_to_string(format!("{}/package_urn/single.puml", task.output_directory)).unwrap();
        assert!(content.trim().contains("library_bootstrap_file"));
        assert!(content.trim().contains("package_bootstrap_file"));
        assert!(content.trim().contains("package_item_file_a"));
        assert!(content.trim().contains("package_item_file_b"));
    }

    #[test]
    fn test_template_with_full() {
        let tera = &create_tera(TEMPLATES.to_vec(), None).unwrap();
        let task = PackageEmbeddedTask {
            mode: EmbeddedMode::Full,
            package_urn: "package_urn".to_string(),
            library_bootstrap_file: None,
            package_bootstrap_file: Some(
                "target/tests/package_embedded_generator/package_urn/package_bootstrap_file.txt"
                    .to_string(),
            ),
            package_item_files: vec![
                "target/tests/package_embedded_generator/package_urn/package_item_file_a.txt"
                    .to_string(),
                "target/tests/package_embedded_generator/package_urn/package_item_file_b.txt"
                    .to_string(),
            ],
            output_directory: "target/tests/package_embedded_generator".to_string(),
            template: get_default_template_package_embedded(),
        };

        delete_file_or_directory(task.output_directory.as_ref()).unwrap();
        write_fixture_file("library_bootstrap_file");
        write_fixture_file("package_bootstrap_file");
        write_fixture_file("package_item_file_a");
        write_fixture_file("package_item_file_b");

        task.cleanup(&[CleanupScope::All]).unwrap();
        task.render_composed_templates(tera).unwrap();

        let content =
            read_to_string(format!("{}/package_urn/full.puml", task.output_directory)).unwrap();
        assert!(!content.trim().contains("library_bootstrap_file"));
        assert!(content.trim().contains("package_bootstrap_file"));
        assert!(content.trim().contains("package_item_file_a"));
        assert!(content.trim().contains("package_item_file_b"));
    }
}
