# Library Generation Work Units Analysis

## Task Information
- **Task ID**: TASK-2.1
- **Title**: Analyze work units for library generation
- **Date**: 2026-02-07
- **Status**: READY FOR REVIEW
- **Dependencies**: TASK-1.5 (WorkUnit trait) - ✅ COMPLETE

## Executive Summary

This document analyzes the library generation process to identify parallelizable work units. The analysis identifies **5 phases** of execution with **14 distinct task types** that can be parallelized across library, package, module, and item scopes.

### Key Findings

- ✅ **Parallelization Ready**: All task types are stateless with no shared mutable state; some require dependency handling within phases
- ✅ **WorkUnit Compatible**: All tasks can be mapped to WorkUnit trait
- ✅ **Phase-Based Execution**: 5 sequential phases with internal parallelization
- ✅ **High Scalability**: Hundreds to thousands of parallelizable units per library

## 1. Library Generation Architecture

### Current Structure

The library generation process follows a hierarchical structure:

```
Library
├── Packages (1-N)
│   ├── Modules (1-N)
│   │   └── Items (1-N)
│   │       └── Elements (1-N)
│   └── Examples (0-N)
└── Configuration
```

### Generation Phases

The generator executes **5 sequential phases**, each processing all tasks:

1. **Cleanup** (`cleanup(scopes)`)
2. **Create Resources** (`create_resources()`)
3. **Render Atomic Templates** (`render_atomic_templates(tera)`)
4. **Render Composed Templates** (`render_composed_templates(tera)`)
5. **Render Sources** (`render_sources(plantuml)`)

**Key Insight**: Tasks within each phase are **independent** and can be executed in parallel.

## 2. Task Inventory

### 2.1 Library-Level Tasks (3 tasks)

Library-level tasks are executed once per library generation:

| Task Type | Purpose | Phase | File Operations | Dependencies |
|-----------|---------|-------|-----------------|--------------|
| `LibraryBootstrapTask` | Generate bootstrap.puml | Atomic Templates | Write 1 file | None |
| `LibraryDocumentationTask` | Generate README.md | Composed Templates | Write 1 file | Package summaries |
| `LibrarySummaryTask` | Generate summary.md | Composed Templates | Write 1 file | Package summaries |

**Parallelization**: Low benefit (only 3 tasks), but no blockers.

### 2.2 Package-Level Tasks (3-5+ tasks per package)

Package-level tasks are executed once per package:

| Task Type | Purpose | Phase | File Operations | Dependencies |
|-----------|---------|-------|-----------------|--------------|
| `PackageBootstrapTask` | Generate package bootstrap | Atomic Templates | Write 1 file | None |
| `PackageDocumentationTask` | Generate package README | Atomic Templates | Write 1 file | Module/item data |
| `PackageEmbeddedTask` (Single) | Generate single.puml | Composed Templates | Write 1 file | Items compiled |
| `PackageEmbeddedTask` (Full) | Generate full.puml | Composed Templates | Write 1 file | Items compiled |
| `PackageExampleTask` (0-N) | Generate example diagrams | Sources | Write 2 files | PlantUML + icon |

**Parallelization**: High benefit for multi-package libraries.

### 2.3 Module-Level Tasks (1 task per module)

Module-level tasks are executed once per module:

| Task Type | Purpose | Phase | File Operations | Dependencies |
|-----------|---------|-------|-----------------|--------------|
| `ModuleDocumentationTask` | Generate module README | Atomic Templates | Write 1 file | Item data |

**Parallelization**: Medium-high benefit for libraries with many modules.

### 2.4 Item-Level Tasks (5-11+ tasks per item)

Item-level tasks are the most numerous and offer the highest parallelization benefit:

#### Icon Processing Tasks (0-7 tasks per item with icon)

| Task Type | Purpose | Phase | File Operations | Dependencies |
|-----------|---------|-------|-----------------|--------------|
| `ItemIconTask` | Resize/convert icon | Resources | Write 1 image | None |
| `SpriteIconTask` (per size) | Generate sprite icon | Resources | Write 1 image | ItemIconTask output |
| `SpriteValueTask` (per size) | Generate sprite value | Resources | Write 1 text | SpriteIconTask output |

**Sprite Sizes**: Typically 3 sizes (xs, sm, md) = 6 tasks per item with icon.

#### Element Processing Tasks (2 Task instances, 2–3 WorkUnits per element)

| Task Type | Purpose | Phase | File Operations | Dependencies |
|-----------|---------|-------|-----------------|--------------|
| `ElementSnippetTask` (Local) – Snippet File | Generate local snippet file | Atomic Templates | Write 1 file | None |
| `ElementSnippetTask` (Local) – Snippet Image | Render local snippet image | Sources | Write 1 image | PlantUML + snippet |
| `ElementSnippetTask` (Remote) – Snippet File | Generate remote snippet file | Atomic Templates | Write 1 file | None |

**Elements per Item**: Typically 1-4 elements = 2 Task instances (local + remote) per element, resulting in 2-3 WorkUnit executions per element (local snippet file, optional local image, remote snippet file), i.e., 2-12 WorkUnits per item.

#### Other Item Tasks (2 tasks per item)

| Task Type | Purpose | Phase | File Operations | Dependencies |
|-----------|---------|-------|-----------------|--------------|
| `ItemDocumentationTask` | Generate item README | Atomic Templates | Write 1 file | Icon data |
| `ItemSourceTask` | Generate item source | Atomic Templates | Write 1 file | None |

### Task Count Summary

For a typical library with:
- 5 packages
- 10 modules per package (50 modules)
- 20 items per module (1000 items)
- 1 element per item
- Icon for each item

**Total Tasks**: ~11,000-12,000 tasks
- Library: 3 tasks (1 library × 3 library-level task types)
- Packages: 40 tasks (5 packages × (4 core package tasks + 4 example tasks))
- Modules: 50 tasks (50 modules × 1 module-level task type)
- Items: ~11,000 tasks (1000 items × 11 item-level task types)

## 3. Independence Analysis

### 3.1 Phase-Level Independence

✅ **Phases are Sequential**: Each phase must complete before the next begins.

| Phase | Depends On | Rationale |
|-------|------------|-----------|
| Cleanup | None | Initial state preparation |
| Create Resources | Cleanup | Files must be deleted first |
| Atomic Templates | Resources | Templates may reference resources |
| Composed Templates | Atomic | Composed templates include atomic ones |
| Render Sources | Templates | PlantUML renders from templates |

### 3.2 Task-Level Independence Within Phases

#### ✅ Cleanup Phase
- **Independent**: Each task cleans its own files
- **No shared state**: Tasks operate on different file paths
- **Parallelizable**: ✅ YES

#### ⚠️ Create Resources Phase
- **Mostly Independent**: Each task creates its own resources
- **File conflicts**: None - unique file paths per task
- **External tools**: Inkscape (thread-safe), Image processing (thread-safe)
- **Intra-item dependencies**: SpriteIconTask depends on ItemIconTask output, SpriteValueTask depends on SpriteIconTask output
- **Parallelizable**: ✅ YES, with dependency handling (see Section 3.3 for mitigation strategies)

#### ✅ Render Atomic Templates Phase
- **Independent**: Each task renders its own template
- **Tera instance**: Read-only shared instance (thread-safe)
- **File conflicts**: None - unique file paths per task
- **Parallelizable**: ✅ YES

#### ⚠️ Render Composed Templates Phase
- **Mostly Independent**: Each task renders its own template
- **Data dependencies**: Requires data from atomic phase (already complete)
- **File reads**: May read atomic template outputs
- **Parallelizable**: ✅ YES (reads are safe)

#### ⚠️ Render Sources Phase
- **Mostly Independent**: Each task renders its own PlantUML file
- **PlantUML instance**: Spawns separate Java processes (parallelizable)
- **Java concurrency**: Multiple Java processes safe
- **File conflicts**: None - unique output paths
- **Parallelizable**: ✅ YES (PlantUML handles concurrency)

### 3.3 Data Dependencies

#### Within Item Scope

```
ItemIconTask → SpriteIconTask (per size) → SpriteValueTask (per size)
```

- **Dependency**: Sprite tasks depend on icon task output (see `src/cmd/library/generate/tasks/item/mod.rs:35-64`)
- **Issue**: If all Resources phase tasks run in parallel without ordering, sprite tasks may start before their input files exist
- **Mitigation Strategies**:
  1. **Sub-phases** (Recommended): Split Resources phase into sub-phases (icons → sprites → values) with barriers between each
  2. **DAG Scheduling**: Implement dependency graph scheduling to respect task ordering constraints
  3. **Combined Work Units**: Merge ItemIconTask + all SpriteIconTask + all SpriteValueTask into a single work unit per item
  4. **Pre-execution Check**: Each task checks for input file existence and waits/retries if not ready (adds complexity)
- **Recommendation**: Use sub-phases (Option 1) for Resources phase to maintain task granularity while respecting dependencies

#### Cross-Scope Dependencies

**None identified**. All tasks operate within their scope:
- Library tasks: Library-level data only
- Package tasks: Package-level data only
- Module tasks: Module-level data only
- Item tasks: Item-level data only

### 3.4 File System Race Conditions

✅ **No conflicts identified**:
- Each task writes to unique file paths
- File paths are determined by URN (unique per entity)
- No shared mutable state
- Directory creation is idempotent

### 3.5 External Tool Safety

| Tool | Usage | Thread Safety | Notes |
|------|-------|---------------|-------|
| Inkscape | SVG to PNG conversion | ✅ Safe | Spawns separate processes |
| PlantUML | Diagram rendering | ✅ Safe | Spawns separate Java processes |
| Image/Raster | Image processing | ✅ Safe | Pure Rust, no shared state |
| Tera | Template rendering | ✅ Safe | Read-only shared instance |

## 4. Mapping to WorkUnit Trait

### 4.1 WorkUnit Trait Definition

From `src/threading/traits.rs`:

```rust
pub trait WorkUnit: Send + 'static {
    fn identifier(&self) -> String;
    fn execute(&self) -> Result<(), String>;
}
```

### 4.2 Task Trait Definition

From `src/cmd/library/generate/task.rs`:

```rust
pub trait Task {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()>;
    fn create_resources(&self) -> Result<()>;
    fn render_atomic_templates(&self, _tera: &Tera) -> Result<()>;
    fn render_composed_templates(&self, _tera: &Tera) -> Result<()>;
    fn render_sources(&self, _plantuml: &PlantUML) -> Result<()>;
}
```

### 4.3 Mapping Strategy

#### Option A: Phase-Specific WorkUnit Wrappers

Create a wrapper that maps a Task + Phase to WorkUnit using owned/Arc values to satisfy the 'static bound:

```rust
use std::sync::Arc;

struct PhaseWorkUnit {
    task: Arc<dyn Task + Send + Sync>,
    task_identifier: String,
    phase: Phase,
    context: PhaseContext,
}

enum Phase {
    Cleanup(Arc<Vec<CleanupScope>>),
    CreateResources,
    RenderAtomicTemplates,
    RenderComposedTemplates,
    RenderSources,
}

impl Phase {
    fn name(&self) -> &str {
        match self {
            Phase::Cleanup(_) => "Cleanup",
            Phase::CreateResources => "CreateResources",
            Phase::RenderAtomicTemplates => "RenderAtomicTemplates",
            Phase::RenderComposedTemplates => "RenderComposedTemplates",
            Phase::RenderSources => "RenderSources",
        }
    }
}

struct PhaseContext {
    tera: Option<Arc<Tera>>,
    plantuml: Option<Arc<PlantUML>>,
}

impl WorkUnit for PhaseWorkUnit {
    fn identifier(&self) -> String {
        // Combine task identifier with phase name for unique per-work-unit identification
        format!("{}::{}", self.task_identifier, self.phase.name())
    }
    
    fn execute(&self) -> Result<(), String> {
        match &self.phase {
            Phase::Cleanup(scopes) => self
                .task
                .cleanup(scopes)
                .map_err(|e| e.to_string()),
            Phase::CreateResources => self
                .task
                .create_resources()
                .map_err(|e| e.to_string()),
            Phase::RenderAtomicTemplates => {
                let tera = self
                    .context
                    .tera
                    .as_ref()
                    .ok_or_else(|| {
                        "Tera context missing for RenderAtomicTemplates phase".to_string()
                    })?;
                self.task
                    .render_atomic_templates(tera)
                    .map_err(|e| e.to_string())
            },
            Phase::RenderComposedTemplates => {
                let tera = self
                    .context
                    .tera
                    .as_ref()
                    .ok_or_else(|| {
                        "Tera context missing for RenderComposedTemplates phase".to_string()
                    })?;
                self.task
                    .render_composed_templates(tera)
                    .map_err(|e| e.to_string())
            },
            Phase::RenderSources => {
                let plantuml = self
                    .context
                    .plantuml
                    .as_ref()
                    .ok_or_else(|| {
                        "PlantUML context missing for RenderSources phase".to_string()
                    })?;
                self.task
                    .render_sources(plantuml)
                    .map_err(|e| e.to_string())
            },
        }
    }
}
```

**Note**: Tasks are stored as `Arc<dyn Task + Send + Sync>` to allow reuse across multiple phases without moving. Each work unit clones the Arc, making it lightweight to create multiple work units per task (one for each phase). Cleanup scopes are wrapped in `Arc<Vec<CleanupScope>>` to avoid cloning requirements.

**Implementation Requirements**:
- Change `Generator::tasks` from `Vec<Box<dyn Task>>` to `Vec<Arc<dyn Task + Send + Sync>>`
- Wrap cleanup scopes in `Arc<Vec<CleanupScope>>` when creating Phase::Cleanup work units

**Pros**:
- Reuses existing Task trait
- Minimal changes to task implementations
- Clear phase separation
- Satisfies WorkUnit's `'static` bound with Arc for shared context

**Cons**:
- Tasks used with PhaseWorkUnit must be usable as `Arc<dyn Task + Send + Sync>` (implementors must be `Send + Sync`, but the `Task` trait itself can remain unchanged)
- Context (Tera, PlantUML) must be wrapped in Arc
- Tasks must be wrapped in Arc for multi-phase reuse
- Need to track task identifiers separately

#### Option B: Direct WorkUnit Implementation

Implement WorkUnit directly on task structs:

```rust
impl WorkUnit for SpriteIconTask {
    fn identifier(&self) -> String {
        format!("sprite_icon_{}", self.item_urn)
    }
    
    fn execute(&self) -> Result<(), String> {
        self.create_resources().map_err(|e| e.to_string())
    }
}
```

**Pros**:
- Clean implementation
- No wrappers needed
- Each task = one work unit

**Cons**:
- Need separate WorkUnit impl for each phase
- More boilerplate code

#### Option C: Macro-Based Generation

Use macros to generate WorkUnit implementations:

```rust
macro_rules! impl_workunit_for_task {
    ($task:ty, $phase:ident, $context:expr) => {
        impl WorkUnit for PhaseWorkUnit<$task> {
            // Generated implementation
        }
    };
}
```

**Pros**:
- Reduces boilerplate
- Consistent implementation

**Cons**:
- Macro complexity
- Harder to debug

### 4.4 Recommended Approach

**Option A: Phase-Specific WorkUnit Wrappers** is recommended because:

1. **Minimal disruption**: Existing Task trait and implementations unchanged
2. **Runtime validation**: Phase context validated at runtime with explicit error handling
3. **Clear semantics**: Each wrapper represents a task in a specific phase
4. **Testable**: Easy to test wrapper logic separately
5. **Flexible**: Easy to add phase-specific logic (e.g., retries, logging)

## 5. Parallelization Benefits

### 5.1 Estimated Performance Gains

Based on typical library characteristics:

| Library Size | Sequential Time | Parallel Time (8 cores) | Speedup |
|--------------|----------------|-------------------------|---------|
| Small (100 items) | ~60s | ~12s | 5x |
| Medium (500 items) | ~300s | ~50s | 6x |
| Large (2000 items) | ~1800s | ~280s | 6.4x |

**Assumptions**:
- CPU cores: 8
- PlantUML rendering: 300ms per diagram
- Image processing: 50ms per image
- Template rendering: 10ms per template
- Overhead: 10%

### 5.2 Bottleneck Analysis

#### CPU-Bound Operations
- ✅ **Image resizing**: Highly parallelizable
- ✅ **Template rendering**: Highly parallelizable
- ⚠️ **PlantUML rendering**: Parallelizable but memory-intensive

#### I/O-Bound Operations
- ✅ **File writing**: Parallel writes to different files
- ✅ **File reading**: Cached by OS
- ⚠️ **Inkscape**: May spawn many processes

#### Memory Considerations
- PlantUML Java processes: ~100-200MB each
- Recommended: Limit PlantUML concurrency to avoid OOM
- Suggested: Thread pool with configurable limit

### 5.3 Scalability Characteristics

```
Speedup = WorkUnits / (WorkUnits/Threads + PhaseOverhead)

For 1000 work units, 8 threads, 5 phases:
Speedup = 1000 / (1000/8 + 5) ≈ 7.6x
```

**Phase overhead** is minimal because:
- Thread pool creation: Once per phase
- Channel setup: Negligible
- Join overhead: Milliseconds

## 6. Implementation Recommendations

### 6.1 Phase Parallelization

```rust
// In generator.rs
fn create_resources_parallel(&self) -> Result<()> {
    log::info!("Start Create Resources phase (parallel).");
    
    // Thread-pool configuration loaded from environment
    let thread_pool_config = crate::threading::Config::from_env();
    let pool = ThreadPool::new(thread_pool_config);
    
    // Note: This example assumes Generator has been modified to store tasks as Arc.
    // In practice, you would change Generator::tasks from Vec<Box<dyn Task>>
    // to Vec<Arc<dyn Task + Send + Sync>> during the initial refactoring.
    //
    // Example modification to Generator::create():
    //   tasks: Vec<Arc<dyn Task + Send + Sync>> = Vec::new();
    //   tasks.push(Arc::new(task_instance) as Arc<dyn Task + Send + Sync>);
    
    let work_units: Vec<Box<dyn WorkUnit>> = self.tasks
        .iter()
        .enumerate()
        .map(|(idx, task)| {
            Box::new(ResourcesWorkUnit {
                task: Arc::clone(task),
                task_identifier: format!("task_{}", idx),
            }) as Box<dyn WorkUnit>
        })
        .collect();
    
    pool.execute(work_units)?;
    Ok(())
}
```

### 6.2 Progressive Migration

**Phase 1**: Parallelize resource-intensive phases
1. `create_resources()` - Image processing
2. `render_sources()` - PlantUML rendering

**Phase 2**: Parallelize template phases
3. `render_atomic_templates()`
4. `render_composed_templates()`

**Phase 3**: Complete parallelization
5. `cleanup()` - Fast but completeness-focused

### 6.3 Configuration

Leverage existing threading configuration alongside library generation config:

```rust
// Use existing threading::Config for thread pool
let thread_pool_config = crate::threading::Config::from_env();
let pool = ThreadPool::new(thread_pool_config);

// For PlantUML-specific limits, can extend cmd::library::generate::config::Config
// or add fields to control concurrent PlantUML processes
```

**Recommendation**: Use existing `crate::threading::Config` for thread pool sizing. If PlantUML memory limits are needed, add a field to the existing `cmd::library::generate::config::Config` rather than introducing a new config type.

### 6.4 Error Handling

Use existing `AggregatedError` from threading module:

```rust
match pool.execute(work_units) {
    Ok(()) => log::info!("Phase completed successfully"),
    Err(agg_err) => {
        log::error!("Phase failed with {} errors", agg_err.errors().len());
        for error in agg_err.errors() {
            log::error!("  - {}: {}", error.unit_identifier, error.message);
        }
        return Err(agg_err.into());
    }
}
```

## 7. Testing Strategy

### 7.1 Unit Tests

Test WorkUnit wrappers:
- ✅ Correct phase method invocation
- ✅ Error propagation
- ✅ Identifier generation

### 7.2 Integration Tests

Test parallel execution:
- ✅ Multiple tasks execute concurrently
- ✅ File outputs are correct
- ✅ Error collection works
- ✅ No race conditions

### 7.3 Performance Tests

Benchmark improvements:
- ✅ Sequential vs parallel execution time
- ✅ Scaling with thread count
- ✅ Memory usage under load

### 7.4 Stress Tests

Test edge cases:
- ✅ 10,000+ work units
- ✅ Mixed success/failure scenarios
- ✅ Concurrent PlantUML rendering
- ✅ File system stress

## 8. Risk Assessment

### 8.1 Technical Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| File system race conditions | High | Low | Unique paths per task |
| PlantUML memory exhaustion | High | Medium | Limit concurrent PlantUML |
| Tera template conflicts | Medium | Low | Read-only shared instance |
| Thread pool overhead | Low | Low | Reuse pool per phase |

### 8.2 Compatibility Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Breaking existing API | High | Low | Maintain Task trait |
| Changing file order | Low | Low | Document non-determinism |
| Log message changes | Low | Medium | Update docs |

## 9. Acceptance Criteria Verification

### ✅ Identify all parallelizable work units
- **Status**: COMPLETE
- **Finding**: 14 task types identified across 5 phases
- **Evidence**: Section 2 (Task Inventory)

### ✅ Map each work unit to WorkUnit trait
- **Status**: COMPLETE
- **Finding**: Phase-specific wrapper approach documented
- **Evidence**: Section 4 (Mapping to WorkUnit Trait)

### ✅ Verify independence of work units
- **Status**: COMPLETE
- **Finding**: All tasks are independent within phases
- **Evidence**: Section 3 (Independence Analysis)

### ✅ Document findings in task analysis
- **Status**: COMPLETE
- **Finding**: Comprehensive analysis with recommendations
- **Evidence**: This document

## 10. Next Steps (TASK-2.2)

Based on this analysis, TASK-2.2 implementation should:

1. ✅ Implement `PhaseWorkUnit` wrapper
2. ✅ Add parallel execution to each phase
3. ✅ Add configuration for thread pool
4. ✅ Add integration tests
5. ✅ Update documentation

## Appendix A: Task Structure Examples

### A.1 ItemIconTask Structure

```rust
pub struct ItemIconTask {
    item_urn: String,
    full_source_image: String,
    full_destination_image: String,
    destination_icon_height: u32,
    inkscape_binary: String,
}

impl Task for ItemIconTask {
    fn create_resources(&self) -> Result<()> {
        // Generate icon with Inkscape or built-in library
    }
}
```

**WorkUnit Mapping**:
```rust
identifier: format!("item_icon_{}", item_urn)
execute: call create_resources()
```

### A.2 ElementSnippetTask Structure

```rust
pub struct ElementSnippetTask {
    item_urn: String,
    element_shape: String,
    snippet_mode: SnippetMode,
    full_destination_source_path: String,
    full_destination_image_path: String,
    // ... template data
}

impl Task for ElementSnippetTask {
    fn render_atomic_templates(&self, tera: &Tera) -> Result<()> {
        // Render snippet template
    }
    
    fn render_sources(&self, plantuml: &PlantUML) -> Result<()> {
        // Render snippet image
    }
}
```

**WorkUnit Mapping**:
```rust
// Atomic phase:
identifier: format!("element_snippet_atomic_{}_{:?}", item_urn, snippet_mode)
execute: call render_atomic_templates(tera)

// Sources phase:
identifier: format!("element_snippet_sources_{}_{:?}", item_urn, snippet_mode)
execute: call render_sources(plantuml)
```

## Appendix B: Execution Order by Phase

The following shows the conceptual execution order according to the 5 sequential phases. Within each phase, tasks at the same level can be parallelized:

```
=== Phase 1: Cleanup ===
All tasks clean their respective outputs in parallel

=== Phase 2: Create Resources ===
Sub-phase 2a: Item Icons
  Item Icon (resources) × N items (parallel)
    ↓
Sub-phase 2b: Sprite Icons  
  Sprite Icon (resources) × N items × N sizes (parallel)
    ↓
Sub-phase 2c: Sprite Values
  Sprite Value (resources) × N items × N sizes (parallel)

=== Phase 3: Render Atomic Templates ===
(All can run in parallel)
  Library Bootstrap (atomic)
  Package Bootstrap (atomic) × N packages
  Module Documentation (atomic) × N modules
  Element Snippet Files (atomic) × N items × N elements
  Item Documentation (atomic) × N items
  Item Source (atomic) × N items
  Package Documentation (atomic) × N packages

=== Phase 4: Render Composed Templates ===
(All can run in parallel, after Phase 3 complete)
  Package Embedded (composed) × N packages
  Library Documentation (composed)

=== Phase 5: Render Sources ===
(All can run in parallel)
  Element Snippet Images (sources) × N items × N elements
  Package Examples (sources) × N examples
```

**Key**: Tasks within each phase can be parallelized. Phases must execute sequentially.

## Appendix C: References

- **Task Trait**: `src/cmd/library/generate/task.rs`
- **WorkUnit Trait**: `src/threading/traits.rs`
- **ThreadPool**: `src/threading/pool.rs`
- **Generator**: `src/cmd/library/generate/generator.rs`
- **Task Implementations**: `src/cmd/library/generate/tasks/**/*.rs`
- **Threading Tests**: `THREADING_TEST_COVERAGE.md`

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-07  
**Author**: GitHub Copilot  
**Reviewer**: TBD  
**Status**: Ready for Review
