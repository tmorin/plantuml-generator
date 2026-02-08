use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use anyhow::Result;
use tera::Tera;

use crate::cmd::library::generate::config::Config;
use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::cmd::library::generate::tasks::item::parse_item;
use crate::cmd::library::generate::tasks::library::parse_library;
use crate::cmd::library::generate::tasks::module::parse_module;
use crate::cmd::library::generate::tasks::package::parse_package;
use crate::cmd::library::generate::work_units::LibraryGenerationTask;
use crate::cmd::library::manifest::library::Library;
use crate::plantuml::PlantUML;
use crate::threading::{Config as ThreadConfig, ThreadPool, WorkUnit};
use crate::urn::Urn;

pub struct Generator {
    config: Config,
    tasks: Vec<Arc<dyn Task + Send + Sync>>,
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
        let mut tasks: Vec<Arc<dyn Task + Send + Sync>> = Vec::new();

        let bootstrap_tasks = parse_library(config, library)?;
        for task in bootstrap_tasks {
            // Since parse functions return Box<dyn Task>, we need to convert
            // We store tasks as Arc<dyn Task + Send + Sync> for threading
            let concrete_task: Box<dyn Task + Send + Sync> = task;
            tasks.push(Arc::from(concrete_task));
        }

        for package in &library.packages {
            if package.urn.is_included_in(_urns) {
                let package_tasks = parse_package(config, library, package)?;
                for task in package_tasks {
                    let concrete_task: Box<dyn Task + Send + Sync> = task;
                    tasks.push(Arc::from(concrete_task));
                }
                for module in &package.modules {
                    if module.urn.is_included_in(_urns) {
                        let module_tasks = parse_module(config, library, package, module)?;
                        for task in module_tasks {
                            let concrete_task: Box<dyn Task + Send + Sync> = task;
                            tasks.push(Arc::from(concrete_task));
                        }
                        for item in &module.items {
                            if item.urn.is_included_in(_urns) {
                                let item_tasks =
                                    parse_item(config, library, package, module, item)?;
                                for task in item_tasks {
                                    let concrete_task: Box<dyn Task + Send + Sync> = task;
                                    tasks.push(Arc::from(concrete_task));
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

    fn cleanup(&self, scopes: &[CleanupScope]) -> Result<()> {
        log::info!("Start the Cleanup phase.");
        let thread_config = ThreadConfig::from_env();
        let pool = ThreadPool::new(thread_config);

        let scopes_arc = Arc::new(scopes.to_vec());
        let work_units: Vec<Box<dyn WorkUnit>> = self
            .tasks
            .iter()
            .enumerate()
            .map(|(idx, task)| {
                Box::new(LibraryGenerationTask::cleanup(
                    Arc::clone(task),
                    format!("cleanup_task_{}", idx),
                    Arc::clone(&scopes_arc),
                )) as Box<dyn WorkUnit>
            })
            .collect();

        pool.execute(work_units)
            .map_err(|e| anyhow::Error::msg(format!("Cleanup phase failed: {}", e)))?;

        Ok(())
    }
    fn create_resources(&self) -> Result<()> {
        log::info!("Start the Create Resources phase.");
        // Note: Create Resources phase has intra-task dependencies (e.g., ItemIconTask → SpriteIconTask)
        // Running these in parallel can cause failures when tasks try to access files created by other tasks.
        // For now, we execute this phase sequentially to maintain data consistency.
        // See: LIBRARY_GENERATION_WORK_UNITS_ANALYSIS.md, Section 3.2
        for task in &self.tasks {
            task.create_resources()?;
        }
        Ok(())
    }
    fn render_atomic_templates(&self, tera: &Tera) -> Result<()> {
        log::info!("Start the Render Atomic Templates phase.");
        let thread_config = ThreadConfig::from_env();
        let pool = ThreadPool::new(thread_config);

        let tera_arc = Arc::new(tera.clone());
        let work_units: Vec<Box<dyn WorkUnit>> = self
            .tasks
            .iter()
            .enumerate()
            .map(|(idx, task)| {
                Box::new(LibraryGenerationTask::render_atomic_templates(
                    Arc::clone(task),
                    format!("render_atomic_task_{}", idx),
                    Arc::clone(&tera_arc),
                )) as Box<dyn WorkUnit>
            })
            .collect();

        pool.execute(work_units).map_err(|e| {
            anyhow::Error::msg(format!("Render Atomic Templates phase failed: {}", e))
        })?;

        Ok(())
    }
    fn render_composed_templates(&self, tera: &Tera) -> Result<()> {
        log::info!("Start the Render Composed Templates phase.");
        let thread_config = ThreadConfig::from_env();
        let pool = ThreadPool::new(thread_config);

        let tera_arc = Arc::new(tera.clone());
        let work_units: Vec<Box<dyn WorkUnit>> = self
            .tasks
            .iter()
            .enumerate()
            .map(|(idx, task)| {
                Box::new(LibraryGenerationTask::render_composed_templates(
                    Arc::clone(task),
                    format!("render_composed_task_{}", idx),
                    Arc::clone(&tera_arc),
                )) as Box<dyn WorkUnit>
            })
            .collect();

        pool.execute(work_units).map_err(|e| {
            anyhow::Error::msg(format!("Render Composed Templates phase failed: {}", e))
        })?;

        Ok(())
    }
    fn render_sources(&self, plantuml: &PlantUML) -> Result<()> {
        log::info!("Start the Render Sources phase.");
        let thread_config = ThreadConfig::from_env();
        let pool = ThreadPool::new(thread_config);

        let plantuml_arc = Arc::new(plantuml.clone());
        let work_units: Vec<Box<dyn WorkUnit>> = self
            .tasks
            .iter()
            .enumerate()
            .map(|(idx, task)| {
                Box::new(LibraryGenerationTask::render_sources(
                    Arc::clone(task),
                    format!("render_sources_task_{}", idx),
                    Arc::clone(&plantuml_arc),
                )) as Box<dyn WorkUnit>
            })
            .collect();

        pool.execute(work_units)
            .map_err(|e| anyhow::Error::msg(format!("Render Sources phase failed: {}", e)))?;

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
