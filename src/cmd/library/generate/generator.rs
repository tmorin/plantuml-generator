use std::fmt::{Debug, Formatter};

use anyhow::Result;
use tera::Tera;

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::cmd::library::generate::tasks::item::parse_item;
use crate::cmd::library::generate::tasks::library::parse_library;
use crate::cmd::library::generate::tasks::module::parse_module;
use crate::cmd::library::generate::tasks::package::parse_package;
use crate::cmd::library::manifest::library::Library;
use crate::counter::Counter;
use crate::plantuml::PlantUML;
use crate::urn::Urn;

pub struct Generator {
    config: Config,
    tasks: Vec<Box<dyn Task>>,
}

impl Debug for Generator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("")
            .field("config", &self.config)
            .field("tasks", &self.tasks.len())
            .finish()
    }
}

impl Generator {
    pub fn create(config: &Config, library: &Library, _urns: &[Urn]) -> Result<Generator> {
        let mut tasks: Vec<Box<dyn Task>> = Vec::new();

        let bootstrap_tasks = parse_library(config, library)?;
        for task in bootstrap_tasks {
            tasks.push(task);
        }

        for package in &library.packages {
            if package.urn.is_included_in(_urns) {
                let package_tasks = parse_package(config, library, package)?;
                for task in package_tasks {
                    tasks.push(task);
                }
                for module in &package.modules {
                    if module.urn.is_included_in(_urns) {
                        let module_tasks = parse_module(config, library, package, module)?;
                        for task in module_tasks {
                            tasks.push(task);
                        }
                        for item in &module.items {
                            if item.urn.is_included_in(_urns) {
                                let item_tasks =
                                    parse_item(config, library, package, module, item)?;
                                for task in item_tasks {
                                    tasks.push(task);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(Generator {
            config: config.clone(),
            tasks,
        })
    }

    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        log::info!("Start the Cleanup phase.");
        for task in &self.tasks {
            task.cleanup(_scopes)?
        }
        Ok(())
    }
    fn create_resources(&self) -> Result<()> {
        log::info!("Start the Create Resources phase.");
        let mut counter = Counter::start(self.tasks.len());
        for task in &self.tasks {
            task.create_resources()?;
            counter.increase();
        }
        counter.stop();
        Ok(())
    }
    fn render_atomic_templates(&self, tera: &Tera) -> Result<()> {
        log::info!("Start the Render Atomic Templates phase.");
        let mut counter = Counter::start(self.tasks.len());
        for task in &self.tasks {
            task.render_atomic_templates(tera)?;
            counter.increase();
        }
        counter.stop();
        Ok(())
    }
    fn render_composed_templates(&self, tera: &Tera) -> Result<()> {
        log::info!("Start the Render Composed Templates phase.");
        let mut counter = Counter::start(self.tasks.len());
        for task in &self.tasks {
            task.render_composed_templates(tera)?;
            counter.increase();
        }
        counter.stop();
        Ok(())
    }
    fn render_sources(&self, plantuml: &PlantUML) -> Result<()> {
        log::info!("Start the Render Sources sources.");
        let mut counter = Counter::start(self.tasks.len());
        for task in &self.tasks {
            task.render_sources(plantuml)?;
            counter.increase();
        }
        counter.stop();
        Ok(())
    }

    pub fn generate(
        &self,
        cleanup_scopes: &[CleanupScope],
        tera: &Tera,
        plantuml: &PlantUML,
    ) -> Result<()> {
        self.cleanup(cleanup_scopes)?;
        self.create_resources()?;
        self.render_atomic_templates(tera)?;
        self.render_composed_templates(tera)?;
        self.render_sources(plantuml)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use std::path::Path;

    use log::LevelFilter;

    use crate::cmd::library::generate::templates::TEMPLATES;
    use crate::plantuml::create_plantuml;
    use crate::tera::create_tera;

    use super::*;

    #[test]
    fn test_full_generation() {
        env_logger::builder().filter_level(LevelFilter::Info).init();
        let config = &Config::default()
            .rebase_directories("target/tests/generator/library-full".to_string())
            .update_plantuml_jar("test/plantuml-1.2022.4.jar".to_string());
        let tera = &create_tera(TEMPLATES.to_vec(), Some("test/tera/**".to_string())).unwrap();
        let plantuml = &create_plantuml(
            &config.java_binary,
            &config.plantuml_jar,
            &config.plantuml_version,
        )
        .unwrap();
        let yaml = &read_to_string(Path::new("test/library-full.yaml")).unwrap();
        let library: &Library = &serde_yaml_ok::from_str(yaml).unwrap();
        let generator = &Generator::create(config, library, &[]).unwrap();
        generator
            .generate(&[CleanupScope::All], tera, plantuml)
            .unwrap();

        let c4model_single_content =
            read_to_string("target/tests/generator/library-full/distribution/c4model/single.puml")
                .unwrap();
        assert!(c4model_single_content
            .trim()
            .contains("!global $INCLUSION_MODE"));
        assert!(c4model_single_content
            .trim()
            .contains("!procedure C4Element("));
        assert!(c4model_single_content.trim().contains("!procedure Person("));
    }

    #[test]
    fn test_icon_reference() {
        let config = &Config::default()
            .rebase_directories("target/tests/generator/library-icon_reference".to_string())
            .update_plantuml_jar("test/plantuml-1.2022.4.jar".to_string());
        let tera = &create_tera(TEMPLATES.to_vec(), Some("test/tera/**".to_string())).unwrap();
        let plantuml = &create_plantuml(
            &config.java_binary,
            &config.plantuml_jar,
            &config.plantuml_version,
        )
        .unwrap();
        let yaml = &read_to_string(Path::new("test/library-icon_reference.yaml")).unwrap();
        let library: &Library = &serde_yaml_ok::from_str(yaml).unwrap();
        let generator = &Generator::create(config, library, &[]).unwrap();
        generator
            .generate(&[CleanupScope::All], tera, plantuml)
            .unwrap();
    }
}
