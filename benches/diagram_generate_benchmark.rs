//! Benchmark: diagram generate sequential vs parallel performance.
//!
//! Measures wall-clock time for rendering a batch of PlantUML diagrams both
//! sequentially and in parallel using rayon.
//!
//! These Criterion benchmarks only record and report timings; they do **not**
//! assert any minimum speedup. The 1.3× speedup check is implemented
//! separately in the ignored unit test `test_parallel_speedup`.
//!
//! Run with:
//!   cargo bench --bench diagram_generate_benchmark
//!
//! Results are also reproducible via the ignored unit test:
//!   cargo test test_parallel_speedup -- --nocapture --ignored

use criterion::{criterion_group, criterion_main, Criterion};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tempfile::TempDir;

/// Path to the PlantUML JAR bundled for testing.
const PLANTUML_JAR: &str = "test/plantuml-1.2022.4.jar";
/// Number of diagrams to render per benchmark iteration.
const DIAGRAM_COUNT: usize = 6;

/// Creates `DIAGRAM_COUNT` simple PlantUML source files in `dir` and returns
/// the paths to those files.
fn create_test_diagrams(dir: &TempDir) -> Vec<PathBuf> {
    (0..DIAGRAM_COUNT)
        .map(|i| {
            let path = dir.path().join(format!("bench_diagram_{}.puml", i));
            std::fs::write(
                &path,
                format!(
                    "@startuml bench_{i}\nobject A_{i}\nobject B_{i}\nA_{i} -> B_{i}\n@enduml\n"
                ),
            )
            .unwrap_or_else(|e| panic!("failed to write diagram {i}: {e}"));
            path
        })
        .collect()
}

/// Renders a single PlantUML source file by invoking the bundled JAR via Java.
///
/// Uses `Command::output()` to capture stdout/stderr (mirroring `PlantUML::render`).
/// The layout engine can be overridden via the `PLANTUML_LAYOUT` environment
/// variable; it defaults to `smetana` so no GraphViz installation is required.
fn render_one(path: &Path) {
    let layout = std::env::var("PLANTUML_LAYOUT").unwrap_or_else(|_| "smetana".to_string());
    let output = Command::new("java")
        .arg("-jar")
        .arg(PLANTUML_JAR)
        .arg(path)
        .arg(format!("-Playout={layout}"))
        .output()
        .unwrap_or_else(|e| panic!("failed to invoke java for {:?}: {e}", path));
    assert!(
        output.status.success(),
        "plantuml returned failure for {:?}: {}",
        path,
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Criterion benchmark group: sequential diagram rendering.
///
/// Each iteration renders all `DIAGRAM_COUNT` files one after another in a
/// single thread.
fn bench_sequential(c: &mut Criterion) {
    let dir = TempDir::new().expect("failed to create temp dir");
    let paths = create_test_diagrams(&dir);

    c.bench_function("diagram_generate_sequential", |b| {
        b.iter(|| {
            for path in &paths {
                render_one(path);
            }
        });
    });
}

/// Criterion benchmark group: parallel diagram rendering.
///
/// Each iteration renders all `DIAGRAM_COUNT` files concurrently using the
/// global rayon thread pool.
fn bench_parallel(c: &mut Criterion) {
    let dir = TempDir::new().expect("failed to create temp dir");
    let paths = create_test_diagrams(&dir);

    c.bench_function("diagram_generate_parallel", |b| {
        b.iter(|| {
            paths.par_iter().for_each(|path| {
                render_one(path);
            });
        });
    });
}

/// Custom Criterion configuration tuned for benchmarks that invoke external
/// processes (each sample takes multiple seconds).
fn configured_criterion() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(5))
        .measurement_time(Duration::from_secs(60))
}

criterion_group! {
    name = benches;
    config = configured_criterion();
    targets = bench_sequential, bench_parallel
}
criterion_main!(benches);
