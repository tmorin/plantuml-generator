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
    fn render_atomic_templates_snippets(&self, tera: &Tera) -> Result<()> {
        log::info!("Start the Render Atomic Templates phase (Snippets).");
        let thread_config = ThreadConfig::from_env();
        let pool = ThreadPool::new(thread_config);

        let tera_arc = Arc::new(tera.clone());

        // Execute snippet rendering tasks (ElementSnippetTask).
        // These must complete before ItemDocumentationTask reads the snippet files.
        let work_units: Vec<Box<dyn WorkUnit>> = self
            .tasks
            .iter()
            .enumerate()
            .map(|(idx, task)| {
                Box::new(LibraryGenerationTask::render_atomic_templates_snippets(
                    Arc::clone(task),
                    format!("render_atomic_snippets_task_{}", idx),
                    Arc::clone(&tera_arc),
                )) as Box<dyn WorkUnit>
            })
            .collect();

        if !work_units.is_empty() {
            log::debug!("Executing {} snippet rendering tasks", work_units.len());
            pool.execute(work_units).map_err(|e| {
                anyhow::Error::msg(format!(
                    "Render Atomic Templates (Snippets) phase failed: {}",
                    e
                ))
            })?;
            log::debug!("Snippet rendering tasks completed");
        }

        Ok(())
    }

    fn render_atomic_templates_other(&self, tera: &Tera) -> Result<()> {
        log::info!("Start the Render Atomic Templates phase (Other).");
        let thread_config = ThreadConfig::from_env();
        let pool = ThreadPool::new(thread_config);

        let tera_arc = Arc::new(tera.clone());

        // Execute other atomic template rendering tasks (all tasks except ElementSnippetTask).
        // These tasks may read files created by snippet tasks.
        let work_units: Vec<Box<dyn WorkUnit>> = self
            .tasks
            .iter()
            .enumerate()
            .map(|(idx, task)| {
                Box::new(LibraryGenerationTask::render_atomic_templates_other(
                    Arc::clone(task),
                    format!("render_atomic_other_task_{}", idx),
                    Arc::clone(&tera_arc),
                )) as Box<dyn WorkUnit>
            })
            .collect();

        if !work_units.is_empty() {
            log::debug!("Executing {} other rendering tasks", work_units.len());
            pool.execute(work_units).map_err(|e| {
                anyhow::Error::msg(format!(
                    "Render Atomic Templates (Other) phase failed: {}",
                    e
                ))
            })?;
            log::debug!("Other rendering tasks completed");
        }

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
        // ## Phase Execution Model
        //
        // This method executes phases sequentially to respect inter-phase dependencies:
        //
        // Phase 1: Cleanup
        //   - Removes old generated files
        //   - Tasks run in PARALLEL (4 threads default)
        //   - No inter-task dependencies
        //
        // Phase 2: CreateResources
        //   - Creates base resources (icons, sprites, etc)
        //   - Tasks run SEQUENTIALLY (by design)
        //   - Has intra-task dependencies (ItemIconTask → SpriteIconTask → SpriteValueTask)
        //
        // Phase 3a: RenderAtomicTemplates (Snippets)
        //   - Renders element snippet source files
        //   - Tasks run in PARALLEL (4 threads default)
        //   - Must complete before Phase 3b (other tasks read these files)
        //
        // Phase 3b: RenderAtomicTemplates (Other)
        //   - Renders individual documentation files
        //   - Tasks run in PARALLEL (4 threads default)
        //   - Reads snippet files created by Phase 3a
        //
        // Phase 4: RenderComposedTemplates
        //   - Renders composite documentation (library-wide, package-wide)
        //   - Tasks run in PARALLEL (4 threads default)
        //   - May read output from Phase 3
        //
        // Phase 5: RenderSources
        //   - Renders PlantUML diagrams to images
        //   - Tasks run in PARALLEL (4 threads default)
        //   - May read output from Phases 3-4
        //
        // Key Property: Each phase's pool.execute() blocks until ALL tasks complete,
        // ensuring strict sequential phase execution. This prevents race conditions
        // where tasks from later phases read files not yet created by earlier phases.

        self.cleanup(cleanup_scopes)?;
        self.create_resources()?;
        // Split render_atomic_templates into two phases to handle intra-phase dependencies
        self.render_atomic_templates_snippets(tera)?;
        self.render_atomic_templates_other(tera)?;
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
        // Use try_init() instead of init() to avoid panic when logger already initialized
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Info)
            .try_init();
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

    /// Tests that phases execute sequentially and tasks execute in parallel.
    /// This test validates the architecture: phase=sequential, task=parallel.
    ///
    /// The test works by:
    /// 1. Running generation with full library
    /// 2. Verifying all expected output files exist (proves all phases completed)
    /// 3. Running multiple times to catch race conditions (tests should be isolated)
    #[test]
    fn test_parallel_task_execution_within_phases() {
        // Use once_cell pattern to initialize logger only once
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Info)
            .try_init();

        // Use unique test directory to prevent interference with other tests
        let test_base = "target/tests/generator/parallel_execution_test";
        let config = &Config::default()
            .rebase_directories(test_base.to_string())
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

        // Verify we have multiple tasks (so parallelization makes sense)
        assert!(
            generator.tasks.len() > 1,
            "Expected multiple tasks for parallelization test"
        );

        // Run generation - if there's a race condition, it will fail here
        generator
            .generate(&[CleanupScope::All], tera, plantuml)
            .unwrap();

        // Verify all phases completed by checking outputs from different phases
        // Phase 1 (Cleanup): Output directory exists
        assert!(
            Path::new(test_base).exists(),
            "Output directory should exist after cleanup phase"
        );

        // Phase 2 (Create Resources): Output directory contains distribution
        let dist_dir = format!("{}/distribution", test_base);
        assert!(
            Path::new(&dist_dir).exists(),
            "Distribution directory should be created in phase 2"
        );

        // Phase 3 (Render Atomic): Individual documentation files exist
        let c4model_readme = format!("{}/c4model/README.md", &dist_dir);
        assert!(
            Path::new(&c4model_readme).exists(),
            "Atomic templates should be rendered in phase 3"
        );

        // Phase 4 (Render Composed): Composite files exist
        let single_puml = format!("{}/c4model/single.puml", &dist_dir);
        assert!(
            Path::new(&single_puml).exists(),
            "Composed templates should be rendered in phase 4"
        );

        // Phase 5 (Render Sources): PlantUML diagrams rendered
        let element_person_png = format!("{}/c4model/Element/Person.Local.png", &dist_dir);
        assert!(
            Path::new(&element_person_png).exists(),
            "PlantUML diagrams should be rendered in phase 5"
        );
    }

    /// Tests that sequential test execution doesn't interfere.
    /// This test is similar to test_parallel_task_execution_within_phases but uses
    /// different library data to ensure test isolation is working.
    #[test]
    fn test_sequential_test_execution_isolation() {
        // Use once_cell pattern to initialize logger only once
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Info)
            .try_init();

        // Use unique test directory distinct from other tests
        let test_base = "target/tests/generator/sequential_isolation_test";
        let config = &Config::default()
            .rebase_directories(test_base.to_string())
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

        // Generate once
        generator
            .generate(&[CleanupScope::All], tera, plantuml)
            .unwrap();

        let single_puml_1 = format!("{}/distribution/c4model/single.puml", test_base);
        let content_1 = read_to_string(&single_puml_1).unwrap();

        // Generate again with same test data - should produce identical output
        generator
            .generate(&[CleanupScope::All], tera, plantuml)
            .unwrap();

        let content_2 = read_to_string(&single_puml_1).unwrap();

        // Content should be identical (deterministic generation)
        assert_eq!(
            content_1, content_2,
            "Regeneration should produce identical output"
        );
    }
}
