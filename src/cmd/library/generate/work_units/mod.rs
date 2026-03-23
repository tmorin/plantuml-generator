//! WorkUnit implementations for library generation tasks.
//!
//! This module provides phase-aware WorkUnit wrappers for different hierarchy levels:
//! - `LibraryGenerationTask`: Wraps library-level tasks
//! - `PackageGenerationTask`: Wraps package-level tasks
//! - `ModuleGenerationTask`: Wraps module-level tasks
//! - `ItemGenerationTask`: Wraps item-level tasks
//!
//! Each wrapper implements the `WorkUnit` trait to support parallel execution
//! in the thread pool. Tasks are wrapped in `Arc` to allow safe sharing across
//! threads while maintaining the 'static lifetime requirement.

use std::sync::Arc;

use tera::Tera;

use crate::cmd::library::generate::task::{CleanupScope, Task};
use crate::plantuml::PlantUML;
use crate::threading::WorkUnit;

/// Enumeration of library generation phases.
///
/// # Phase/Context Dependencies
///
/// - `Cleanup`: Requires cleanup scopes, no other context needed
/// - `CreateResources`: No context required
/// - `RenderAtomicTemplatesSnippets`: Requires Tera context; renders ElementSnippetTask outputs
/// - `RenderAtomicTemplatesOther`: Requires Tera context; renders remaining atomic template tasks
/// - `RenderComposedTemplates`: Requires Tera context (will error if missing)
/// - `RenderSources`: Requires PlantUML context (will error if missing)
///
/// Using a factory method ensures the correct context is provided for each phase.
#[allow(dead_code)]
enum Phase {
    Cleanup(Arc<Vec<CleanupScope>>),
    CreateResources,
    RenderAtomicTemplatesSnippets,
    RenderAtomicTemplatesOther,
    RenderComposedTemplates,
    RenderSources,
}

#[allow(dead_code)]
impl Phase {
    fn name(&self) -> &str {
        match self {
            Phase::Cleanup(_) => "cleanup",
            Phase::CreateResources => "create_resources",
            Phase::RenderAtomicTemplatesSnippets => "render_atomic_templates_snippets",
            Phase::RenderAtomicTemplatesOther => "render_atomic_templates_other",
            Phase::RenderComposedTemplates => "render_composed_templates",
            Phase::RenderSources => "render_sources",
        }
    }
}

/// Context required to execute task phases.
///
/// Each phase requires specific context to be present:
/// - `tera`: Required for rendering phases (atomic and composed templates)
/// - `plantuml`: Required for the sources rendering phase
///
/// Missing context will result in a descriptive error during execution.
#[allow(dead_code)]
struct PhaseContext {
    tera: Option<Arc<Tera>>,
    plantuml: Option<Arc<PlantUML>>,
}

/// WorkUnit wrapper for library-level generation tasks.
///
/// Wraps library-level tasks (`LibraryBootstrapTask`, `LibraryDocumentationTask`,
/// `LibrarySummaryTask`) to enable parallel execution via the thread pool.
#[allow(dead_code)]
pub struct LibraryGenerationTask {
    task: Arc<dyn Task + Send + Sync>,
    task_identifier: String,
    phase: Phase,
    context: PhaseContext,
}

#[allow(dead_code)]
impl LibraryGenerationTask {
    /// Creates a new LibraryGenerationTask for the cleanup phase.
    pub fn cleanup(
        task: Arc<dyn Task + Send + Sync>,
        task_identifier: String,
        scopes: Arc<Vec<CleanupScope>>,
    ) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::Cleanup(scopes),
            context: PhaseContext {
                tera: None,
                plantuml: None,
            },
        }
    }

    /// Creates a new LibraryGenerationTask for the create_resources phase.
    pub fn create_resources(task: Arc<dyn Task + Send + Sync>, task_identifier: String) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::CreateResources,
            context: PhaseContext {
                tera: None,
                plantuml: None,
            },
        }
    }

    /// Creates a new LibraryGenerationTask for the render_atomic_templates_snippets phase.
    pub fn render_atomic_templates_snippets(
        task: Arc<dyn Task + Send + Sync>,
        task_identifier: String,
        tera: Arc<Tera>,
    ) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::RenderAtomicTemplatesSnippets,
            context: PhaseContext {
                tera: Some(tera),
                plantuml: None,
            },
        }
    }

    /// Creates a new LibraryGenerationTask for the render_atomic_templates_other phase.
    pub fn render_atomic_templates_other(
        task: Arc<dyn Task + Send + Sync>,
        task_identifier: String,
        tera: Arc<Tera>,
    ) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::RenderAtomicTemplatesOther,
            context: PhaseContext {
                tera: Some(tera),
                plantuml: None,
            },
        }
    }

    /// Creates a new LibraryGenerationTask for the render_composed_templates phase.
    pub fn render_composed_templates(
        task: Arc<dyn Task + Send + Sync>,
        task_identifier: String,
        tera: Arc<Tera>,
    ) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::RenderComposedTemplates,
            context: PhaseContext {
                tera: Some(tera),
                plantuml: None,
            },
        }
    }

    /// Creates a new LibraryGenerationTask for the render_sources phase.
    pub fn render_sources(
        task: Arc<dyn Task + Send + Sync>,
        task_identifier: String,
        plantuml: Arc<PlantUML>,
    ) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::RenderSources,
            context: PhaseContext {
                tera: None,
                plantuml: Some(plantuml),
            },
        }
    }
}

impl WorkUnit for LibraryGenerationTask {
    fn identifier(&self) -> String {
        format!("{}::{}", self.task_identifier, self.phase.name())
    }

    fn execute(&self) -> Result<(), String> {
        match &self.phase {
            Phase::Cleanup(scopes) => self
                .task
                .cleanup(scopes)
                .map_err(|e| format!("{}::cleanup: {}", self.task_identifier, e)),
            Phase::CreateResources => self
                .task
                .create_resources()
                .map_err(|e| format!("{}::create_resources: {}", self.task_identifier, e)),
            Phase::RenderAtomicTemplatesSnippets => {
                let tera = self.context.tera.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_atomic_templates_snippets: Tera context missing",
                        self.task_identifier
                    )
                })?;
                self.task
                    .render_atomic_templates_snippets(tera)
                    .map_err(|e| {
                        format!(
                            "{}::render_atomic_templates_snippets: {}",
                            self.task_identifier, e
                        )
                    })
            }
            Phase::RenderAtomicTemplatesOther => {
                let tera = self.context.tera.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_atomic_templates_other: Tera context missing",
                        self.task_identifier
                    )
                })?;
                self.task.render_atomic_templates_other(tera).map_err(|e| {
                    format!(
                        "{}::render_atomic_templates_other: {}",
                        self.task_identifier, e
                    )
                })
            }
            Phase::RenderComposedTemplates => {
                let tera = self.context.tera.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_composed_templates: Tera context missing",
                        self.task_identifier
                    )
                })?;
                self.task.render_composed_templates(tera).map_err(|e| {
                    format!("{}::render_composed_templates: {}", self.task_identifier, e)
                })
            }
            Phase::RenderSources => {
                let plantuml = self.context.plantuml.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_sources: PlantUML context missing",
                        self.task_identifier
                    )
                })?;
                self.task
                    .render_sources(plantuml)
                    .map_err(|e| format!("{}::render_sources: {}", self.task_identifier, e))
            }
        }
    }
}

/// WorkUnit wrapper for package-level generation tasks.
///
/// Wraps package-level tasks (`PackageBootstrapTask`, `PackageDocumentationTask`,
/// `PackageEmbeddedTask`, `PackageExampleTask`) to enable parallel execution via
/// the thread pool.
#[allow(dead_code)]
pub struct PackageGenerationTask {
    task: Arc<dyn Task + Send + Sync>,
    task_identifier: String,
    phase: Phase,
    context: PhaseContext,
}

#[allow(dead_code)]
impl PackageGenerationTask {
    /// Creates a new PackageGenerationTask for the cleanup phase.
    pub fn cleanup(
        task: Arc<dyn Task + Send + Sync>,
        task_identifier: String,
        scopes: Arc<Vec<CleanupScope>>,
    ) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::Cleanup(scopes),
            context: PhaseContext {
                tera: None,
                plantuml: None,
            },
        }
    }

    /// Creates a new PackageGenerationTask for the create_resources phase.
    pub fn create_resources(task: Arc<dyn Task + Send + Sync>, task_identifier: String) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::CreateResources,
            context: PhaseContext {
                tera: None,
                plantuml: None,
            },
        }
    }

    /// Creates a new PackageGenerationTask for the render_composed_templates phase.
    pub fn render_composed_templates(
        task: Arc<dyn Task + Send + Sync>,
        task_identifier: String,
        tera: Arc<Tera>,
    ) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::RenderComposedTemplates,
            context: PhaseContext {
                tera: Some(tera),
                plantuml: None,
            },
        }
    }

    /// Creates a new PackageGenerationTask for the render_sources phase.
    pub fn render_sources(
        task: Arc<dyn Task + Send + Sync>,
        task_identifier: String,
        plantuml: Arc<PlantUML>,
    ) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::RenderSources,
            context: PhaseContext {
                tera: None,
                plantuml: Some(plantuml),
            },
        }
    }
}

impl WorkUnit for PackageGenerationTask {
    fn identifier(&self) -> String {
        format!("{}::{}", self.task_identifier, self.phase.name())
    }

    fn execute(&self) -> Result<(), String> {
        match &self.phase {
            Phase::Cleanup(scopes) => self
                .task
                .cleanup(scopes)
                .map_err(|e| format!("{}::cleanup: {}", self.task_identifier, e)),
            Phase::CreateResources => self
                .task
                .create_resources()
                .map_err(|e| format!("{}::create_resources: {}", self.task_identifier, e)),
            Phase::RenderAtomicTemplatesSnippets => {
                let tera = self.context.tera.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_atomic_templates_snippets: Tera context missing",
                        self.task_identifier
                    )
                })?;
                self.task
                    .render_atomic_templates_snippets(tera)
                    .map_err(|e| {
                        format!(
                            "{}::render_atomic_templates_snippets: {}",
                            self.task_identifier, e
                        )
                    })
            }
            Phase::RenderAtomicTemplatesOther => {
                let tera = self.context.tera.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_atomic_templates_other: Tera context missing",
                        self.task_identifier
                    )
                })?;
                self.task.render_atomic_templates_other(tera).map_err(|e| {
                    format!(
                        "{}::render_atomic_templates_other: {}",
                        self.task_identifier, e
                    )
                })
            }
            Phase::RenderComposedTemplates => {
                let tera = self.context.tera.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_composed_templates: Tera context missing",
                        self.task_identifier
                    )
                })?;
                self.task.render_composed_templates(tera).map_err(|e| {
                    format!("{}::render_composed_templates: {}", self.task_identifier, e)
                })
            }
            Phase::RenderSources => {
                let plantuml = self.context.plantuml.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_sources: PlantUML context missing",
                        self.task_identifier
                    )
                })?;
                self.task
                    .render_sources(plantuml)
                    .map_err(|e| format!("{}::render_sources: {}", self.task_identifier, e))
            }
        }
    }
}

/// WorkUnit wrapper for module-level generation tasks.
///
/// Wraps module-level tasks (`ModuleDocumentationTask`) to enable parallel
/// execution via the thread pool.
#[allow(dead_code)]
pub struct ModuleGenerationTask {
    task: Arc<dyn Task + Send + Sync>,
    task_identifier: String,
    phase: Phase,
    context: PhaseContext,
}

#[allow(dead_code)]
impl ModuleGenerationTask {
    /// Creates a new ModuleGenerationTask for the cleanup phase.
    pub fn cleanup(
        task: Arc<dyn Task + Send + Sync>,
        task_identifier: String,
        scopes: Arc<Vec<CleanupScope>>,
    ) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::Cleanup(scopes),
            context: PhaseContext {
                tera: None,
                plantuml: None,
            },
        }
    }

    /// Creates a new ModuleGenerationTask for the create_resources phase.
    pub fn create_resources(task: Arc<dyn Task + Send + Sync>, task_identifier: String) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::CreateResources,
            context: PhaseContext {
                tera: None,
                plantuml: None,
            },
        }
    }

    /// Creates a new ModuleGenerationTask for the render_composed_templates phase.
    pub fn render_composed_templates(
        task: Arc<dyn Task + Send + Sync>,
        task_identifier: String,
        tera: Arc<Tera>,
    ) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::RenderComposedTemplates,
            context: PhaseContext {
                tera: Some(tera),
                plantuml: None,
            },
        }
    }

    /// Creates a new ModuleGenerationTask for the render_sources phase.
    pub fn render_sources(
        task: Arc<dyn Task + Send + Sync>,
        task_identifier: String,
        plantuml: Arc<PlantUML>,
    ) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::RenderSources,
            context: PhaseContext {
                tera: None,
                plantuml: Some(plantuml),
            },
        }
    }
}

impl WorkUnit for ModuleGenerationTask {
    fn identifier(&self) -> String {
        format!("{}::{}", self.task_identifier, self.phase.name())
    }

    fn execute(&self) -> Result<(), String> {
        match &self.phase {
            Phase::Cleanup(scopes) => self
                .task
                .cleanup(scopes)
                .map_err(|e| format!("{}::cleanup: {}", self.task_identifier, e)),
            Phase::CreateResources => self
                .task
                .create_resources()
                .map_err(|e| format!("{}::create_resources: {}", self.task_identifier, e)),
            Phase::RenderAtomicTemplatesSnippets => {
                let tera = self.context.tera.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_atomic_templates_snippets: Tera context missing",
                        self.task_identifier
                    )
                })?;
                self.task
                    .render_atomic_templates_snippets(tera)
                    .map_err(|e| {
                        format!(
                            "{}::render_atomic_templates_snippets: {}",
                            self.task_identifier, e
                        )
                    })
            }
            Phase::RenderAtomicTemplatesOther => {
                let tera = self.context.tera.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_atomic_templates_other: Tera context missing",
                        self.task_identifier
                    )
                })?;
                self.task.render_atomic_templates_other(tera).map_err(|e| {
                    format!(
                        "{}::render_atomic_templates_other: {}",
                        self.task_identifier, e
                    )
                })
            }
            Phase::RenderComposedTemplates => {
                let tera = self.context.tera.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_composed_templates: Tera context missing",
                        self.task_identifier
                    )
                })?;
                self.task.render_composed_templates(tera).map_err(|e| {
                    format!("{}::render_composed_templates: {}", self.task_identifier, e)
                })
            }
            Phase::RenderSources => {
                let plantuml = self.context.plantuml.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_sources: PlantUML context missing",
                        self.task_identifier
                    )
                })?;
                self.task
                    .render_sources(plantuml)
                    .map_err(|e| format!("{}::render_sources: {}", self.task_identifier, e))
            }
        }
    }
}

/// WorkUnit wrapper for item-level generation tasks.
///
/// Wraps item-level tasks (`ItemIconTask`, `SpriteIconTask`, `SpriteValueTask`,
/// `ElementSnippetTask`, `ItemDocumentationTask`, `ItemSourceTask`) to enable
/// parallel execution via the thread pool.
#[allow(dead_code)]
pub struct ItemGenerationTask {
    task: Arc<dyn Task + Send + Sync>,
    task_identifier: String,
    phase: Phase,
    context: PhaseContext,
}

#[allow(dead_code)]
impl ItemGenerationTask {
    /// Creates a new ItemGenerationTask for the cleanup phase.
    pub fn cleanup(
        task: Arc<dyn Task + Send + Sync>,
        task_identifier: String,
        scopes: Arc<Vec<CleanupScope>>,
    ) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::Cleanup(scopes),
            context: PhaseContext {
                tera: None,
                plantuml: None,
            },
        }
    }

    /// Creates a new ItemGenerationTask for the create_resources phase.
    pub fn create_resources(task: Arc<dyn Task + Send + Sync>, task_identifier: String) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::CreateResources,
            context: PhaseContext {
                tera: None,
                plantuml: None,
            },
        }
    }

    /// Creates a new ItemGenerationTask for the render_composed_templates phase.
    pub fn render_composed_templates(
        task: Arc<dyn Task + Send + Sync>,
        task_identifier: String,
        tera: Arc<Tera>,
    ) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::RenderComposedTemplates,
            context: PhaseContext {
                tera: Some(tera),
                plantuml: None,
            },
        }
    }

    /// Creates a new ItemGenerationTask for the render_sources phase.
    pub fn render_sources(
        task: Arc<dyn Task + Send + Sync>,
        task_identifier: String,
        plantuml: Arc<PlantUML>,
    ) -> Self {
        Self {
            task,
            task_identifier,
            phase: Phase::RenderSources,
            context: PhaseContext {
                tera: None,
                plantuml: Some(plantuml),
            },
        }
    }
}

impl WorkUnit for ItemGenerationTask {
    fn identifier(&self) -> String {
        format!("{}::{}", self.task_identifier, self.phase.name())
    }

    fn execute(&self) -> Result<(), String> {
        match &self.phase {
            Phase::Cleanup(scopes) => self
                .task
                .cleanup(scopes)
                .map_err(|e| format!("{}::cleanup: {}", self.task_identifier, e)),
            Phase::CreateResources => self
                .task
                .create_resources()
                .map_err(|e| format!("{}::create_resources: {}", self.task_identifier, e)),
            Phase::RenderAtomicTemplatesSnippets => {
                let tera = self.context.tera.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_atomic_templates_snippets: Tera context missing",
                        self.task_identifier
                    )
                })?;
                self.task
                    .render_atomic_templates_snippets(tera)
                    .map_err(|e| {
                        format!(
                            "{}::render_atomic_templates_snippets: {}",
                            self.task_identifier, e
                        )
                    })
            }
            Phase::RenderAtomicTemplatesOther => {
                let tera = self.context.tera.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_atomic_templates_other: Tera context missing",
                        self.task_identifier
                    )
                })?;
                self.task.render_atomic_templates_other(tera).map_err(|e| {
                    format!(
                        "{}::render_atomic_templates_other: {}",
                        self.task_identifier, e
                    )
                })
            }
            Phase::RenderComposedTemplates => {
                let tera = self.context.tera.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_composed_templates: Tera context missing",
                        self.task_identifier
                    )
                })?;
                self.task.render_composed_templates(tera).map_err(|e| {
                    format!("{}::render_composed_templates: {}", self.task_identifier, e)
                })
            }
            Phase::RenderSources => {
                let plantuml = self.context.plantuml.as_ref().ok_or_else(|| {
                    format!(
                        "{}::render_sources: PlantUML context missing",
                        self.task_identifier
                    )
                })?;
                self.task
                    .render_sources(plantuml)
                    .map_err(|e| format!("{}::render_sources: {}", self.task_identifier, e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTask {
        calls: Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl Task for MockTask {
        fn cleanup(&self, _scopes: &[CleanupScope]) -> anyhow::Result<()> {
            self.calls.lock().unwrap().push("cleanup".to_string());
            Ok(())
        }

        fn create_resources(&self) -> anyhow::Result<()> {
            self.calls
                .lock()
                .unwrap()
                .push("create_resources".to_string());
            Ok(())
        }

        fn render_atomic_templates(&self, _tera: &Tera) -> anyhow::Result<()> {
            self.calls
                .lock()
                .unwrap()
                .push("render_atomic_templates".to_string());
            Ok(())
        }

        fn render_composed_templates(&self, _tera: &Tera) -> anyhow::Result<()> {
            self.calls
                .lock()
                .unwrap()
                .push("render_composed_templates".to_string());
            Ok(())
        }

        fn render_sources(&self, _plantuml: &PlantUML) -> anyhow::Result<()> {
            self.calls
                .lock()
                .unwrap()
                .push("render_sources".to_string());
            Ok(())
        }
    }

    #[test]
    fn test_library_generation_task_cleanup() {
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let task = Arc::new(MockTask {
            calls: calls.clone(),
        });
        let scopes = Arc::new(vec![CleanupScope::All]);
        let work_unit = LibraryGenerationTask::cleanup(task, "lib_task_1".to_string(), scopes);

        assert_eq!(work_unit.identifier(), "lib_task_1::cleanup");
        assert!(work_unit.execute().is_ok());
        assert_eq!(calls.lock().unwrap().as_slice(), &["cleanup"]);
    }

    #[test]
    fn test_library_generation_task_create_resources() {
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let task = Arc::new(MockTask {
            calls: calls.clone(),
        });
        let work_unit = LibraryGenerationTask::create_resources(task, "lib_task_2".to_string());

        assert_eq!(work_unit.identifier(), "lib_task_2::create_resources");
        assert!(work_unit.execute().is_ok());
        assert_eq!(calls.lock().unwrap().as_slice(), &["create_resources"]);
    }

    #[test]
    fn test_package_generation_task_cleanup() {
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let task = Arc::new(MockTask {
            calls: calls.clone(),
        });
        let scopes = Arc::new(vec![CleanupScope::Example]);
        let work_unit = PackageGenerationTask::cleanup(task, "pkg_task_1".to_string(), scopes);

        assert_eq!(work_unit.identifier(), "pkg_task_1::cleanup");
        assert!(work_unit.execute().is_ok());
        assert_eq!(calls.lock().unwrap().as_slice(), &["cleanup"]);
    }

    #[test]
    fn test_module_generation_task_cleanup() {
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let task = Arc::new(MockTask {
            calls: calls.clone(),
        });
        let scopes = Arc::new(vec![CleanupScope::Item]);
        let work_unit = ModuleGenerationTask::cleanup(task, "mod_task_1".to_string(), scopes);

        assert_eq!(work_unit.identifier(), "mod_task_1::cleanup");
        assert!(work_unit.execute().is_ok());
        assert_eq!(calls.lock().unwrap().as_slice(), &["cleanup"]);
    }

    #[test]
    fn test_item_generation_task_cleanup() {
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let task = Arc::new(MockTask {
            calls: calls.clone(),
        });
        let scopes = Arc::new(vec![CleanupScope::ItemIcon]);
        let work_unit = ItemGenerationTask::cleanup(task, "item_task_1".to_string(), scopes);

        assert_eq!(work_unit.identifier(), "item_task_1::cleanup");
        assert!(work_unit.execute().is_ok());
        assert_eq!(calls.lock().unwrap().as_slice(), &["cleanup"]);
    }

    #[test]
    fn test_library_generation_task_all_phases() {
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let task: Arc<dyn Task + Send + Sync> = Arc::new(MockTask {
            calls: calls.clone(),
        });

        // Test cleanup phase
        LibraryGenerationTask::cleanup(
            Arc::clone(&task),
            "lib_test".to_string(),
            Arc::new(vec![CleanupScope::All]),
        )
        .execute()
        .unwrap();

        // Test create_resources phase
        LibraryGenerationTask::create_resources(Arc::clone(&task), "lib_test".to_string())
            .execute()
            .unwrap();

        let recorded_calls = calls.lock().unwrap().clone();
        assert_eq!(recorded_calls.len(), 2);
        assert_eq!(recorded_calls[0], "cleanup");
        assert_eq!(recorded_calls[1], "create_resources");
    }

    #[test]
    fn test_work_unit_trait_object() {
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let task: Arc<dyn Task + Send + Sync> = Arc::new(MockTask {
            calls: calls.clone(),
        });
        let scopes = Arc::new(vec![CleanupScope::All]);

        // Create different work unit types as trait objects
        let lib_task: Box<dyn WorkUnit> = Box::new(LibraryGenerationTask::cleanup(
            Arc::clone(&task),
            "lib_1".to_string(),
            Arc::clone(&scopes),
        ));
        let pkg_task: Box<dyn WorkUnit> = Box::new(PackageGenerationTask::cleanup(
            Arc::clone(&task),
            "pkg_1".to_string(),
            Arc::clone(&scopes),
        ));
        let mod_task: Box<dyn WorkUnit> = Box::new(ModuleGenerationTask::cleanup(
            Arc::clone(&task),
            "mod_1".to_string(),
            Arc::clone(&scopes),
        ));
        let item_task: Box<dyn WorkUnit> = Box::new(ItemGenerationTask::cleanup(
            Arc::clone(&task),
            "item_1".to_string(),
            Arc::clone(&scopes),
        ));

        // Verify identifiers are correct
        assert_eq!(lib_task.identifier(), "lib_1::cleanup");
        assert_eq!(pkg_task.identifier(), "pkg_1::cleanup");
        assert_eq!(mod_task.identifier(), "mod_1::cleanup");
        assert_eq!(item_task.identifier(), "item_1::cleanup");

        // Execute all
        assert!(lib_task.execute().is_ok());
        assert!(pkg_task.execute().is_ok());
        assert!(mod_task.execute().is_ok());
        assert!(item_task.execute().is_ok());

        // Verify all cleanup methods were called
        assert_eq!(calls.lock().unwrap().len(), 4);
    }

    #[test]
    fn assert_work_units_are_send_sync() {
        // Compile-time verification that all WorkUnit types are Send + Sync
        // This ensures they can safely be used in parallel execution contexts
        fn is_send_sync<T: Send + Sync>() {}

        is_send_sync::<LibraryGenerationTask>();
        is_send_sync::<PackageGenerationTask>();
        is_send_sync::<ModuleGenerationTask>();
        is_send_sync::<ItemGenerationTask>();
    }

    // ── Error-handling tests for TASK-2.6 ────────────────────────────────────

    /// A task whose every phase can be configured to fail with a given message.
    struct FailingTask {
        error_message: String,
    }

    impl Task for FailingTask {
        fn cleanup(&self, _scopes: &[CleanupScope]) -> anyhow::Result<()> {
            Err(anyhow::Error::msg(self.error_message.clone()))
        }

        fn create_resources(&self) -> anyhow::Result<()> {
            Err(anyhow::Error::msg(self.error_message.clone()))
        }

        fn render_atomic_templates(&self, _tera: &Tera) -> anyhow::Result<()> {
            Err(anyhow::Error::msg(self.error_message.clone()))
        }

        fn render_atomic_templates_snippets(&self, _tera: &Tera) -> anyhow::Result<()> {
            Err(anyhow::Error::msg(self.error_message.clone()))
        }

        fn render_composed_templates(&self, _tera: &Tera) -> anyhow::Result<()> {
            Err(anyhow::Error::msg(self.error_message.clone()))
        }

        fn render_sources(&self, _plantuml: &PlantUML) -> anyhow::Result<()> {
            Err(anyhow::Error::msg(self.error_message.clone()))
        }
    }

    /// A task that panics unconditionally in all implemented phases.
    struct PanickingTask;

    impl Task for PanickingTask {
        fn cleanup(&self, _scopes: &[CleanupScope]) -> anyhow::Result<()> {
            panic!("simulated panic in cleanup");
        }

        fn render_atomic_templates(&self, _tera: &Tera) -> anyhow::Result<()> {
            panic!("simulated panic in render_atomic_templates");
        }
    }

    // ── I/O failure handling ─────────────────────────────────────────────────

    /// Verifies that an I/O failure in the cleanup phase is surfaced by the
    /// work unit with an error message that includes both the task identifier
    /// and the original I/O error text.
    #[test]
    fn test_io_failure_in_cleanup_phase() {
        let task = Arc::new(FailingTask {
            error_message: "I/O error: failed to remove file".to_string(),
        });
        let scopes = Arc::new(vec![CleanupScope::All]);
        let work_unit =
            LibraryGenerationTask::cleanup(task, "lib_io_fail_cleanup".to_string(), scopes);

        let result = work_unit.execute();
        assert!(result.is_err(), "Expected error from failing cleanup task");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("lib_io_fail_cleanup"),
            "Error should contain the task identifier; got: {msg}"
        );
        assert!(
            msg.contains("I/O error"),
            "Error should propagate the original I/O message; got: {msg}"
        );
    }

    /// Verifies that an I/O failure in the create_resources phase is surfaced
    /// with identifier and error text in the message.
    #[test]
    fn test_io_failure_in_create_resources_phase() {
        let task = Arc::new(FailingTask {
            error_message: "I/O error: disk full".to_string(),
        });
        let work_unit =
            LibraryGenerationTask::create_resources(task, "lib_io_fail_create".to_string());

        let result = work_unit.execute();
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("lib_io_fail_create"),
            "Error should contain the task identifier; got: {msg}"
        );
        assert!(
            msg.contains("disk full"),
            "Error should propagate the original message; got: {msg}"
        );
    }

    /// Verifies that an I/O failure in the render_composed_templates phase is
    /// surfaced with identifier and error text.
    #[test]
    fn test_io_failure_in_render_composed_templates_phase() {
        let task: Arc<dyn Task + Send + Sync> = Arc::new(FailingTask {
            error_message: "I/O error: permission denied".to_string(),
        });
        let tera = Arc::new(Tera::default());
        let work_unit = LibraryGenerationTask::render_composed_templates(
            task,
            "lib_io_fail_composed".to_string(),
            tera,
        );

        let result = work_unit.execute();
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("lib_io_fail_composed"),
            "Error should contain the task identifier; got: {msg}"
        );
        assert!(
            msg.contains("permission denied"),
            "Error should propagate the original message; got: {msg}"
        );
    }

    /// Verifies that an I/O failure in the render_atomic_templates_snippets
    /// phase is surfaced with identifier and error text.
    #[test]
    fn test_io_failure_in_render_atomic_templates_snippets_phase() {
        let task: Arc<dyn Task + Send + Sync> = Arc::new(FailingTask {
            error_message: "I/O error: network unreachable".to_string(),
        });
        let tera = Arc::new(Tera::default());
        let work_unit = LibraryGenerationTask::render_atomic_templates_snippets(
            task,
            "lib_io_fail_atomic_snippets".to_string(),
            tera,
        );

        let result = work_unit.execute();
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("lib_io_fail_atomic_snippets"),
            "Error should contain the task identifier; got: {msg}"
        );
        assert!(
            msg.contains("network unreachable"),
            "Error should propagate the original message; got: {msg}"
        );
    }

    /// Verifies that an I/O failure in the render_atomic_templates_other phase
    /// is surfaced with identifier and error text.
    #[test]
    fn test_io_failure_in_render_atomic_templates_other_phase() {
        let task: Arc<dyn Task + Send + Sync> = Arc::new(FailingTask {
            error_message: "I/O error: file not found".to_string(),
        });
        let tera = Arc::new(Tera::default());
        let work_unit = LibraryGenerationTask::render_atomic_templates_other(
            task,
            "lib_io_fail_atomic_other".to_string(),
            tera,
        );

        let result = work_unit.execute();
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("lib_io_fail_atomic_other"),
            "Error should contain the task identifier; got: {msg}"
        );
        assert!(
            msg.contains("file not found"),
            "Error should propagate the original message; got: {msg}"
        );
    }

    /// Verifies that an I/O failure in the render_sources phase is surfaced
    /// with identifier and error text.
    #[test]
    fn test_io_failure_in_render_sources_phase() {
        use crate::plantuml::create_plantuml;

        let task: Arc<dyn Task + Send + Sync> = Arc::new(FailingTask {
            error_message: "I/O error: write failed".to_string(),
        });
        // Use dummy values: FailingTask returns an error before PlantUML is invoked.
        let plantuml = Arc::new(create_plantuml("java", "plantuml.jar", "0.0.0").unwrap());
        let work_unit = LibraryGenerationTask::render_sources(
            task,
            "lib_io_fail_sources".to_string(),
            plantuml,
        );

        let result = work_unit.execute();
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("lib_io_fail_sources"),
            "Error should contain the task identifier; got: {msg}"
        );
        assert!(
            msg.contains("write failed"),
            "Error should propagate the original message; got: {msg}"
        );
    }

    // ── Error message format ──────────────────────────────────────────────────

    /// Verifies that the error message produced by a failing work unit contains
    /// both the task identifier and the phase name so operators can locate the
    /// failing task quickly.
    #[test]
    fn test_error_message_contains_identifier_and_phase() {
        let task = Arc::new(FailingTask {
            error_message: "underlying failure".to_string(),
        });
        let scopes = Arc::new(vec![CleanupScope::All]);
        let work_unit =
            LibraryGenerationTask::cleanup(task, "identifiable_task_xyz".to_string(), scopes);

        let msg = work_unit.execute().unwrap_err();
        assert!(
            msg.contains("identifiable_task_xyz"),
            "Error message must name the failing task; got: {msg}"
        );
        assert!(
            msg.contains("cleanup"),
            "Error message must name the failing phase; got: {msg}"
        );
    }

    // ── Panic handling in threads ─────────────────────────────────────────────

    /// Verifies that a task panic during parallel execution via ThreadPool is
    /// caught and reported as an error rather than crashing the process.
    #[test]
    fn test_panic_in_library_task_is_caught_by_thread_pool() {
        use crate::threading::{Config as ThreadConfig, ThreadPool};

        let task: Arc<dyn Task + Send + Sync> = Arc::new(PanickingTask);
        let scopes = Arc::new(vec![CleanupScope::All]);
        let work_units: Vec<Box<dyn WorkUnit>> = vec![Box::new(LibraryGenerationTask::cleanup(
            task,
            "panicking_lib_task".to_string(),
            scopes,
        ))];

        let pool = ThreadPool::new(ThreadConfig::new(2));
        let result = pool.execute(work_units);

        assert!(result.is_err(), "Expected error because task panicked");
        let agg = result.unwrap_err();
        assert!(
            !agg.is_empty(),
            "At least one error should be present after a panic"
        );
        // The ThreadPool reports panics under worker identifiers like "worker_0".
        // Task identifiers are not preserved when a thread panics; the failing
        // worker is identified via its worker index instead.
        let has_worker_identifier = agg
            .errors()
            .iter()
            .any(|e| e.unit_identifier.contains("worker_"));
        assert!(
            has_worker_identifier,
            "Error unit_identifier should contain a 'worker_' identifier; got: {:?}",
            agg.errors()
                .iter()
                .map(|e| &e.unit_identifier)
                .collect::<Vec<_>>()
        );
    }

    // ── Concurrent multiple failures ──────────────────────────────────────────

    /// Verifies that when several library-generation work units fail concurrently
    /// in the thread pool, ALL failures are collected rather than only the first
    /// one being reported.
    #[test]
    fn test_concurrent_task_failures_all_collected() {
        use crate::threading::{Config as ThreadConfig, ThreadPool};

        let scopes = Arc::new(vec![CleanupScope::All]);
        let task_count = 5_usize;

        let work_units: Vec<Box<dyn WorkUnit>> = (0..task_count)
            .map(|i| {
                let task: Arc<dyn Task + Send + Sync> = Arc::new(FailingTask {
                    error_message: format!("concurrent failure {i}"),
                });
                Box::new(LibraryGenerationTask::cleanup(
                    task,
                    format!("concurrent_task_{i}"),
                    Arc::clone(&scopes),
                )) as Box<dyn WorkUnit>
            })
            .collect();

        let pool = ThreadPool::new(ThreadConfig::new(4));
        let result = pool.execute(work_units);

        assert!(
            result.is_err(),
            "Expected aggregated error from concurrent failures"
        );
        let agg = result.unwrap_err();
        assert_eq!(
            agg.len(),
            task_count,
            "All {task_count} concurrent failures should be collected; got {}",
            agg.len()
        );
    }

    /// Verifies that mixed success/failure scenarios in concurrent execution
    /// report only the failing tasks and not the successful ones.
    #[test]
    fn test_concurrent_mixed_success_and_failure() {
        use crate::threading::{Config as ThreadConfig, ThreadPool};

        let scopes = Arc::new(vec![CleanupScope::All]);
        let task_count = 6_usize;
        let failing_indices: std::collections::HashSet<usize> =
            std::collections::HashSet::from([1, 3, 5]);

        let work_units: Vec<Box<dyn WorkUnit>> = (0..task_count)
            .map(|i| {
                let task: Arc<dyn Task + Send + Sync> = if failing_indices.contains(&i) {
                    Arc::new(FailingTask {
                        error_message: format!("failure at index {i}"),
                    })
                } else {
                    Arc::new(MockTask {
                        calls: Arc::new(std::sync::Mutex::new(Vec::new())),
                    })
                };
                Box::new(LibraryGenerationTask::cleanup(
                    task,
                    format!("mixed_task_{i}"),
                    Arc::clone(&scopes),
                )) as Box<dyn WorkUnit>
            })
            .collect();

        let pool = ThreadPool::new(ThreadConfig::new(4));
        let result = pool.execute(work_units);

        assert!(result.is_err(), "Expected error from failing tasks");
        let agg = result.unwrap_err();
        assert_eq!(
            agg.len(),
            failing_indices.len(),
            "Only failing tasks should be reported; expected {}, got {}",
            failing_indices.len(),
            agg.len()
        );
        for err in agg.errors() {
            assert!(
                err.message.contains("failure at index"),
                "Error message should identify the failing task; got: {}",
                err.message
            );
        }
    }

    // ── Cleanup on error ─────────────────────────────────────────────────────

    /// Verifies that when the cleanup phase fails for some tasks, the errors
    /// are all collected and execution continues for the remaining tasks
    /// (thread pool does NOT abort mid-run on first failure).
    #[test]
    fn test_cleanup_phase_failure_continues_remaining_tasks() {
        use crate::threading::{Config as ThreadConfig, ThreadPool};

        let scopes = Arc::new(vec![CleanupScope::All]);
        let completed_calls = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));

        // Alternating failing/succeeding tasks.
        let work_units: Vec<Box<dyn WorkUnit>> = (0..4_usize)
            .map(|i| {
                let task: Arc<dyn Task + Send + Sync> = if i % 2 == 0 {
                    Arc::new(FailingTask {
                        error_message: format!("cleanup I/O error task {i}"),
                    })
                } else {
                    Arc::new(MockTask {
                        calls: Arc::clone(&completed_calls),
                    })
                };
                Box::new(LibraryGenerationTask::cleanup(
                    task,
                    format!("cleanup_err_task_{i}"),
                    Arc::clone(&scopes),
                )) as Box<dyn WorkUnit>
            })
            .collect();

        let pool = ThreadPool::new(ThreadConfig::new(4));
        let result = pool.execute(work_units);

        // Two tasks should have failed.
        assert!(result.is_err());
        let agg = result.unwrap_err();
        assert_eq!(
            agg.len(),
            2,
            "Exactly 2 tasks should have failed; got {}",
            agg.len()
        );

        // The 2 succeeding tasks should have run their cleanup despite the failures.
        let calls = completed_calls.lock().unwrap();
        assert_eq!(
            calls.len(),
            2,
            "Both successful tasks should have executed cleanup; got {}",
            calls.len()
        );
    }
}
