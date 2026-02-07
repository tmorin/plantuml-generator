# TASK-3.2 Implementation Specification

**Task**: Parallelize diagram generation  
**Based On**: TASK-3.1 Analysis (see `DIAGRAM_GENERATION_PARALLELIZATION_ANALYSIS.md`)  
**Date**: 2026-02-07

## Overview

This document provides detailed implementation specifications for parallelizing the diagram generation command using the existing threading infrastructure.

## Implementation Steps

### 1. Make PlantUML Thread-Safe

**File**: `src/plantuml.rs`

**Change**: Add `Clone` derive to `PlantUML` struct

```rust
#[derive(Debug, Clone)]  // Add Clone
pub struct PlantUML {
    java_binary: String,
    plantuml_jar: String,
    plantuml_version: String,
}
```

**Rationale**: 
- PlantUML contains only String fields (no mutable state)
- Clone is cheap (3 String clones)
- Each work unit needs its own PlantUML instance

**Alternative**: Use `Arc<PlantUML>` if Clone proves expensive in practice

### 2. Create DiagramWorkUnit

**File**: `src/cmd/diagram/generate/mod.rs` (new section)

**Implementation**:

```rust
use crate::threading::WorkUnit;

struct DiagramWorkUnit {
    source_path: PathBuf,
    plantuml: PlantUML,
    plantuml_args: Vec<String>,
    force_generation: bool,
    last_generation_timestamp: i64,
}

impl DiagramWorkUnit {
    fn new(
        source_path: PathBuf,
        plantuml: PlantUML,
        plantuml_args: Vec<String>,
        force_generation: bool,
        last_generation_timestamp: i64,
    ) -> Self {
        Self {
            source_path,
            plantuml,
            plantuml_args,
            force_generation,
            last_generation_timestamp,
        }
    }
}

impl WorkUnit for DiagramWorkUnit {
    fn identifier(&self) -> String {
        self.source_path
            .to_str()
            .unwrap_or("unknown")
            .to_string()
    }

    fn execute(&self) -> Result<(), String> {
        // Get file modification timestamp
        let last_modification_timestamp = get_last_modified(&self.source_path)
            .map_err(|e| format!("Failed to get modification time: {}", e))?;

        // Check if generation is needed
        if self.force_generation 
            || last_modification_timestamp > self.last_generation_timestamp 
        {
            log::info!("generate {:?}", self.source_path);
            
            // Render the diagram
            self.plantuml
                .render(&self.source_path, Some(self.plantuml_args.clone()))
                .map_err(|e| format!("Failed to render: {}", e))?;
        } else {
            log::debug!("skip {:?} (not modified)", self.source_path);
        }

        Ok(())
    }
}
```

**Key Design Decisions**:

1. **Timestamp Check Inside WorkUnit**: Each work unit independently decides if rendering is needed
2. **Error Conversion**: Convert `anyhow::Error` to `String` for WorkUnit trait compatibility
3. **Logging**: Maintain existing log messages for consistency
4. **Clone plantuml_args**: WorkUnit needs to own its data for Send

### 3. Modify execute_diagram_generate

**File**: `src/cmd/diagram/generate/mod.rs`

**Changes to existing function**:

```rust
use crate::threading::{Config as ThreadConfig, ThreadPool};

pub fn execute_diagram_generate(arg_matches: &ArgMatches) -> Result<()> {
    // ... existing setup code (lines 98-119) ...
    
    // Download PlantUML JAR (BEFORE parallel execution)
    plantuml.download()?;
    
    // Get latest generation timestamp
    let last_generation_timestamp = get_last_generation_timestamp(last_gen_path)?;
    
    // Discover source files
    let puml_paths = get_puml_paths(config);
    
    // Parse plantuml_args once
    let plantuml_args = arg_matches
        .get_many::<String>("plantuml_args")
        .unwrap_or_default()
        .map(|v| v.to_string())
        .collect::<Vec<_>>();
    
    // Create thread pool
    let thread_config = ThreadConfig::from_env();
    let thread_pool = ThreadPool::new(thread_config);
    
    // Create work units
    let work_units: Vec<Box<dyn WorkUnit>> = puml_paths
        .into_iter()
        .map(|source_path| {
            Box::new(DiagramWorkUnit::new(
                source_path,
                plantuml.clone(),
                plantuml_args.clone(),
                force_generation,
                last_generation_timestamp,
            )) as Box<dyn WorkUnit>
        })
        .collect();
    
    // Execute in parallel
    thread_pool
        .execute(work_units)
        .map_err(|agg_err| {
            anyhow::Error::msg(format!(
                "Failed to generate {} diagram(s):\n{}",
                agg_err.len(),
                agg_err
            ))
        })?;
    
    // Save timestamp (only if all succeeded)
    save_last_generation_timestamp(last_gen_path)?;
    
    Ok(())
}
```

**Key Changes**:

1. **Lines to Remove**: 124-141 (old sequential loop)
2. **Lines to Add**: Thread pool creation and work unit execution
3. **JAR Download**: Ensure `plantuml.download()` happens BEFORE creating work units
4. **Error Handling**: Convert `AggregatedError` to `anyhow::Error` for consistency
5. **Timestamp**: Save only after successful parallel execution

### 4. Configuration

**Environment Variable**: `PLANTUML_GENERATOR_THREADS`

**Default**: Number of CPU cores (via `ThreadConfig::from_env()`)

**Usage**:
```bash
# Use 8 threads
export PLANTUML_GENERATOR_THREADS=8
plantuml-generator diagram generate

# Use default (CPU cores)
plantuml-generator diagram generate
```

**Documentation**: Update README.md or user documentation

### 5. Testing

#### 5.1 Update Existing Test

**File**: `src/cmd/diagram/generate/mod.rs::test_diagram_generation`

**Approach**: Test should still pass with parallel execution

**Verification Points**:
- All diagram files are generated
- Output correctness is unchanged
- Timestamp handling works

#### 5.2 Add Parallel-Specific Tests

**New Test 1: Parallel Execution with Multiple Files**

```rust
#[test]
fn test_parallel_diagram_generation() {
    // Setup: Create 10+ test .puml files
    // Execute: Run diagram generation with PLANTUML_GENERATOR_THREADS=4
    // Verify: All files generated correctly
    // Verify: Faster than sequential (timing check)
}
```

**New Test 2: Error Handling in Parallel Execution**

```rust
#[test]
fn test_parallel_diagram_generation_with_errors() {
    // Setup: Create mix of valid and invalid .puml files
    // Execute: Run diagram generation
    // Verify: Error aggregation works
    // Verify: Successful files are still generated
}
```

**New Test 3: Thread Configuration**

```rust
#[test]
#[serial]  // Requires serial_test for env var isolation
fn test_thread_count_from_env() {
    // Set PLANTUML_GENERATOR_THREADS=2
    // Execute: Run diagram generation
    // Verify: Uses configured thread count (via logging or other means)
}
```

#### 5.3 Integration Tests

Create `tests/parallel_diagram_generation.rs`:

```rust
use std::fs;
use std::path::Path;
use std::time::Instant;

#[test]
fn test_parallel_execution_performance() {
    // Setup: Create 20 test diagrams
    // Execute: Run with 1 thread
    // Time: Record duration (sequential)
    // Execute: Run with 4 threads
    // Time: Record duration (parallel)
    // Verify: Parallel is significantly faster
}
```

### 6. Documentation Updates

#### 6.1 README.md

Add section about parallelization:

```markdown
## Performance

### Parallel Diagram Generation

The `diagram generate` command automatically uses parallel execution to speed up
processing of multiple diagram files. By default, it uses all available CPU cores.

To control the number of threads:

\`\`\`bash
export PLANTUML_GENERATOR_THREADS=8
plantuml-generator diagram generate
\`\`\`

Thread count should be between 1 and 256. Invalid values fall back to CPU core count.

#### Performance Expectations

- **Single file**: No performance difference
- **Multiple files (4 cores)**: ~3-4× faster
- **Many files (8 cores)**: ~6-7× faster

Performance scales linearly up to the number of CPU cores for CPU-bound rendering.
```

#### 6.2 CHANGELOG.md

Add entry:

```markdown
### [Version] - [Date]

#### Added
- Parallel diagram generation for improved performance with multiple files
- Environment variable `PLANTUML_GENERATOR_THREADS` to control parallelization

#### Performance
- Diagram generation now uses all CPU cores by default
- Expected 3-4× speedup on typical projects with 4+ cores
```

#### 6.3 AGENTS.md

Update "Development Workflow" section if needed:

```markdown
### Environment Setup

\`\`\`bash
# ... existing variables ...

# Optional: Control diagram generation parallelization (optional)
# export PLANTUML_GENERATOR_THREADS=8
\`\`\`
```

## Error Handling

### Aggregated Errors

When parallel execution encounters errors, the `AggregatedError` will contain:

```
Failed to generate 2 diagram(s):
  - Work unit 'path/to/diagram1.puml' failed: Failed to render: ...
  - Work unit 'path/to/diagram2.puml' failed: Failed to render: ...
```

### Error Scenarios

1. **PlantUML rendering fails**: Captured in work unit, added to aggregated error
2. **File read fails**: Captured in work unit, added to aggregated error
3. **Worker thread panics**: Captured by ThreadPool, added to aggregated error
4. **All files succeed**: No error, timestamp is updated
5. **Partial failure**: All errors aggregated, timestamp is NOT updated

### Error Recovery

On error:
1. All started work units complete (no early abort)
2. Errors are collected and reported together
3. Timestamp is NOT updated (forces retry on next run)
4. Exit with non-zero status code

## Performance Characteristics

### Expected Performance

Given N threads and M diagram files:

- **M < N**: Uses M threads (no benefit from extra threads)
- **M = N**: ~N× speedup (ideal parallelization)
- **M > N**: ~N× speedup (threads process files in batches)

### Overhead

- Thread spawn/join: ~1-2ms per thread
- Channel communication: ~microseconds per file
- Error aggregation: ~microseconds per error

### Recommendations

1. **Default (CPU cores)**: Best for most cases
2. **High thread count**: Only beneficial with many files
3. **Low thread count**: Useful for memory-constrained environments
4. **Single thread**: Useful for debugging or memory pressure

## Migration Path

### Phase 1: Implementation (TASK-3.2)
- Implement parallel execution with simple timestamp handling
- Default to CPU core count
- Environment variable configuration

### Phase 2: Monitoring (Future)
- Add performance metrics
- Log thread utilization
- Report speedup factors

### Phase 3: Optimization (Future)
- Per-file timestamp tracking
- Dynamic thread pool sizing
- Progress reporting for long-running generations
- Work-stealing for better load balancing

## Rollback Plan

If issues arise:

1. **Emergency Rollback**: Add `--sequential` flag to force old behavior
2. **Configuration Rollback**: Set `PLANTUML_GENERATOR_THREADS=1`
3. **Code Rollback**: Keep old sequential code in git history

## Testing Checklist

- [ ] PlantUML Clone implementation
- [ ] DiagramWorkUnit implementation
- [ ] ThreadPool integration
- [ ] Error handling and aggregation
- [ ] Timestamp synchronization
- [ ] Environment variable configuration
- [ ] Existing tests pass
- [ ] New parallel-specific tests added
- [ ] Integration tests added
- [ ] Performance benchmarks
- [ ] Documentation updated
- [ ] CHANGELOG updated

## Success Criteria

1. All existing tests pass
2. New tests demonstrate parallel execution
3. Performance improvement verified (2-4× with 4 cores)
4. Error handling works correctly
5. Configuration via environment variable works
6. Documentation is complete and accurate

## References

- Analysis: `DIAGRAM_GENERATION_PARALLELIZATION_ANALYSIS.md`
- Threading Module: `src/threading/mod.rs`
- ThreadPool Tests: `THREADING_TEST_COVERAGE.md`
- Current Implementation: `src/cmd/diagram/generate/mod.rs`
