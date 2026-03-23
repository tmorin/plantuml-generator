/// End-to-end tests for multi-threading feature validation.
///
/// These tests exercise the `PLANTUML_GENERATOR_THREADS` environment variable
/// and verify that parallel diagram generation produces correct output across
/// different thread configurations.
///
/// # Acceptance Criteria (TASK-4.1)
///
/// - All CLI commands run successfully with multi-threading enabled
/// - Performance improvement: parallel mode must not produce fewer/incorrect outputs
/// - Edge cases handled: single thread, invalid env var, empty workload, many files
/// - Results documented via assertion messages
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use tempfile::TempDir;

mod common;

// Path to the pre-downloaded PlantUML jar checked into the repo.
// Passing this explicitly to every `diagram generate` call avoids network
// downloads during testing, which would slow CI and risk rate-limiting.
const PLANTUML_JAR: &str = "test/plantuml-1.2022.4.jar";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn create_puml(dir: &Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, content).expect("Failed to write .puml file");
    path
}

fn simple_sequence(label: &str) -> String {
    format!(
        "@startuml\nAlice -> Bob: {}\nBob -> Alice: Ack\n@enduml",
        label
    )
}

fn run_diagram_generate(
    source_dir: &Path,
    cache_dir: &Path,
    threads: Option<&str>,
    extra_args: &[&str],
) -> std::process::Output {
    let binary = common::get_binary_path();
    let mut cmd = Command::new(&binary);
    cmd.arg("diagram")
        .arg("generate")
        .arg("-s")
        .arg(source_dir)
        .arg("-C")
        .arg(cache_dir)
        .arg("-P")
        .arg(PLANTUML_JAR)
        .arg("-f"); // Always force generation in tests

    for arg in extra_args {
        cmd.arg(arg);
    }

    if let Some(t) = threads {
        cmd.env("PLANTUML_GENERATOR_THREADS", t);
    } else {
        cmd.env_remove("PLANTUML_GENERATOR_THREADS");
    }

    cmd.output().expect("Failed to execute plantuml-generator")
}

// ---------------------------------------------------------------------------
// CLI smoke tests — all commands must succeed with threading enabled
// ---------------------------------------------------------------------------

/// Verify that `--help` works with multi-threading env var set.
#[test]
fn test_threading_help_command_works() {
    let binary = common::get_binary_path();
    let output = Command::new(&binary)
        .arg("--help")
        .env("PLANTUML_GENERATOR_THREADS", "4")
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "help command must succeed even with threading env var set"
    );
}

/// Verify `diagram generate --help` works.
#[test]
fn test_threading_diagram_help_works() {
    let binary = common::get_binary_path();
    let output = Command::new(&binary)
        .args(["diagram", "generate", "--help"])
        .env("PLANTUML_GENERATOR_THREADS", "4")
        .output()
        .expect("Failed to execute binary");

    // In this binary, clap help is printed to stderr and exits with 0.
    assert!(
        output.status.success(),
        "diagram generate --help must succeed"
    );
}

// ---------------------------------------------------------------------------
// Single-thread mode
// ---------------------------------------------------------------------------

/// Verify that diagram generation works correctly in single-thread mode
/// (`PLANTUML_GENERATOR_THREADS=1`).
#[test]
fn test_threading_single_thread_generates_correctly() {
    let source_dir = TempDir::new().expect("Failed to create source dir");
    let cache_dir = TempDir::new().expect("Failed to create cache dir");

    create_puml(
        source_dir.path(),
        "single.puml",
        &simple_sequence("single-thread"),
    );

    let output = run_diagram_generate(source_dir.path(), cache_dir.path(), Some("1"), &[]);

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Single-thread diagram generation failed");
    }

    assert!(
        source_dir.path().join("single.png").exists(),
        "Output PNG must be created in single-thread mode"
    );
}

/// Verify that multiple diagrams are generated correctly in single-thread mode.
#[test]
fn test_threading_single_thread_multiple_diagrams() {
    let source_dir = TempDir::new().expect("Failed to create source dir");
    let cache_dir = TempDir::new().expect("Failed to create cache dir");

    for i in 0..4 {
        create_puml(
            source_dir.path(),
            &format!("seq_{}.puml", i),
            &simple_sequence(&format!("msg-{}", i)),
        );
    }

    let output = run_diagram_generate(source_dir.path(), cache_dir.path(), Some("1"), &[]);

    assert!(
        output.status.success(),
        "Single-thread generation of multiple diagrams must succeed. \
         STDERR: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    for i in 0..4 {
        let png = source_dir.path().join(format!("seq_{}.png", i));
        assert!(
            png.exists(),
            "Output PNG seq_{}.png must exist after single-thread generation",
            i
        );
    }
}

// ---------------------------------------------------------------------------
// Multi-thread mode
// ---------------------------------------------------------------------------

/// Verify that diagram generation works correctly in multi-thread mode
/// (`PLANTUML_GENERATOR_THREADS=4`).
#[test]
fn test_threading_multi_thread_generates_correctly() {
    let source_dir = TempDir::new().expect("Failed to create source dir");
    let cache_dir = TempDir::new().expect("Failed to create cache dir");

    create_puml(
        source_dir.path(),
        "parallel.puml",
        &simple_sequence("parallel"),
    );

    let output = run_diagram_generate(source_dir.path(), cache_dir.path(), Some("4"), &[]);

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Multi-thread diagram generation failed");
    }

    assert!(
        source_dir.path().join("parallel.png").exists(),
        "Output PNG must be created in multi-thread (4) mode"
    );
}

/// Verify that multiple diagrams are all generated correctly in multi-thread mode.
#[test]
fn test_threading_multi_thread_all_diagrams_generated() {
    let source_dir = TempDir::new().expect("Failed to create source dir");
    let cache_dir = TempDir::new().expect("Failed to create cache dir");

    const DIAGRAM_COUNT: usize = 8;
    for i in 0..DIAGRAM_COUNT {
        create_puml(
            source_dir.path(),
            &format!("diag_{}.puml", i),
            &simple_sequence(&format!("parallel-msg-{}", i)),
        );
    }

    let output = run_diagram_generate(source_dir.path(), cache_dir.path(), Some("4"), &[]);

    assert!(
        output.status.success(),
        "Multi-thread generation of {} diagrams must succeed. STDERR: {}",
        DIAGRAM_COUNT,
        String::from_utf8_lossy(&output.stderr)
    );

    for i in 0..DIAGRAM_COUNT {
        let png = source_dir.path().join(format!("diag_{}.png", i));
        assert!(
            png.exists(),
            "diag_{}.png must be generated in multi-thread mode",
            i
        );
    }
}

// ---------------------------------------------------------------------------
// Correctness: output is identical regardless of thread count
// ---------------------------------------------------------------------------

/// Verify that single-thread and multi-thread modes produce the same set of output files.
///
/// Each run forces regeneration using its own fresh cache directory so we can
/// compare the set of generated files without interference from prior runs.
#[test]
fn test_threading_parallel_correctness_same_file_set() {
    const DIAGRAM_COUNT: usize = 6;

    // --- single-thread run ---
    let src1 = TempDir::new().expect("Failed to create src1");
    let cache1 = TempDir::new().expect("Failed to create cache1");
    for i in 0..DIAGRAM_COUNT {
        create_puml(
            src1.path(),
            &format!("c_{}.puml", i),
            &simple_sequence(&format!("correctness-{}", i)),
        );
    }
    let out1 = run_diagram_generate(src1.path(), cache1.path(), Some("1"), &[]);
    assert!(
        out1.status.success(),
        "Single-thread correctness run must succeed. STDERR: {}",
        String::from_utf8_lossy(&out1.stderr)
    );

    // --- multi-thread run (4 threads) ---
    let src4 = TempDir::new().expect("Failed to create src4");
    let cache4 = TempDir::new().expect("Failed to create cache4");
    for i in 0..DIAGRAM_COUNT {
        create_puml(
            src4.path(),
            &format!("c_{}.puml", i),
            &simple_sequence(&format!("correctness-{}", i)),
        );
    }
    let out4 = run_diagram_generate(src4.path(), cache4.path(), Some("4"), &[]);
    assert!(
        out4.status.success(),
        "Multi-thread correctness run must succeed. STDERR: {}",
        String::from_utf8_lossy(&out4.stderr)
    );

    // Both runs must produce the same file names.
    let collect_pngs = |dir: &Path| -> Vec<String> {
        let mut files: Vec<String> = fs::read_dir(dir)
            .expect("Failed to read dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.ends_with(".png"))
            .collect();
        files.sort();
        files
    };

    let pngs1 = collect_pngs(src1.path());
    let pngs4 = collect_pngs(src4.path());

    assert_eq!(
        pngs1.len(),
        DIAGRAM_COUNT,
        "Single-thread run must produce {} PNG files, got {:?}",
        DIAGRAM_COUNT,
        pngs1
    );
    assert_eq!(
        pngs1, pngs4,
        "Single-thread and multi-thread runs must produce identical file sets.\n\
         Single-thread: {:?}\nMulti-thread: {:?}",
        pngs1, pngs4
    );
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

/// Verify that an empty source directory completes without error.
#[test]
fn test_threading_empty_source_directory() {
    let source_dir = TempDir::new().expect("Failed to create source dir");
    let cache_dir = TempDir::new().expect("Failed to create cache dir");

    // No .puml files — should still succeed with zero work units.
    let output = run_diagram_generate(source_dir.path(), cache_dir.path(), Some("4"), &[]);

    assert!(
        output.status.success(),
        "Empty-source-directory run must succeed. STDERR: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Verify that a single diagram is generated correctly with multiple threads configured
/// (more threads than work units — edge case in thread pool).
#[test]
fn test_threading_more_threads_than_diagrams() {
    let source_dir = TempDir::new().expect("Failed to create source dir");
    let cache_dir = TempDir::new().expect("Failed to create cache dir");

    create_puml(
        source_dir.path(),
        "only_one.puml",
        &simple_sequence("only-one"),
    );

    // 8 threads, 1 diagram — pool must handle this without panic/deadlock.
    let output = run_diagram_generate(source_dir.path(), cache_dir.path(), Some("8"), &[]);

    assert!(
        output.status.success(),
        "More-threads-than-diagrams edge case must succeed. STDERR: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        source_dir.path().join("only_one.png").exists(),
        "only_one.png must be created when threads > work units"
    );
}

/// Verify that an invalid `PLANTUML_GENERATOR_THREADS` value falls back to the
/// default thread count and generation still succeeds.
#[test]
fn test_threading_invalid_env_var_falls_back_gracefully() {
    let source_dir = TempDir::new().expect("Failed to create source dir");
    let cache_dir = TempDir::new().expect("Failed to create cache dir");

    create_puml(
        source_dir.path(),
        "fallback.puml",
        &simple_sequence("fallback"),
    );

    let output = run_diagram_generate(
        source_dir.path(),
        cache_dir.path(),
        Some("not-a-number"),
        &[],
    );

    assert!(
        output.status.success(),
        "Invalid PLANTUML_GENERATOR_THREADS must fall back to default and succeed. \
         STDERR: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        source_dir.path().join("fallback.png").exists(),
        "fallback.png must be created when PLANTUML_GENERATOR_THREADS is invalid"
    );
}

/// Verify that `PLANTUML_GENERATOR_THREADS=0` (out of range) falls back to the
/// default thread count and generation still succeeds.
#[test]
fn test_threading_zero_threads_falls_back_gracefully() {
    let source_dir = TempDir::new().expect("Failed to create source dir");
    let cache_dir = TempDir::new().expect("Failed to create cache dir");

    create_puml(
        source_dir.path(),
        "zero_threads.puml",
        &simple_sequence("zero-threads"),
    );

    let output = run_diagram_generate(source_dir.path(), cache_dir.path(), Some("0"), &[]);

    assert!(
        output.status.success(),
        "PLANTUML_GENERATOR_THREADS=0 must fall back to default and succeed. \
         STDERR: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        source_dir.path().join("zero_threads.png").exists(),
        "zero_threads.png must be created when PLANTUML_GENERATOR_THREADS=0"
    );
}

/// Verify that `PLANTUML_GENERATOR_THREADS=257` (out of range) falls back to
/// the default and generation succeeds.
#[test]
fn test_threading_out_of_range_threads_falls_back_gracefully() {
    let source_dir = TempDir::new().expect("Failed to create source dir");
    let cache_dir = TempDir::new().expect("Failed to create cache dir");

    create_puml(
        source_dir.path(),
        "oor.puml",
        &simple_sequence("out-of-range"),
    );

    let output = run_diagram_generate(source_dir.path(), cache_dir.path(), Some("257"), &[]);

    assert!(
        output.status.success(),
        "PLANTUML_GENERATOR_THREADS=257 must fall back to default and succeed. \
         STDERR: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        source_dir.path().join("oor.png").exists(),
        "oor.png must be created when PLANTUML_GENERATOR_THREADS is out of range"
    );
}

// ---------------------------------------------------------------------------
// Performance: parallel mode must not be substantially slower than sequential
// ---------------------------------------------------------------------------

/// Verify that multi-thread mode does not take substantially longer than
/// single-thread mode for a batch of diagrams.
///
/// This is a smoke-level performance check: we do not assert strict speedup
/// (that is tested in the criterion benchmarks) but we do assert that the
/// parallel run finishes in reasonable wall-clock time relative to the
/// sequential run — specifically it must complete within 3× the single-thread
/// wall time (with a minimum absolute floor of 30 s), ruling out deadlocks or
/// severe contention while remaining stable in resource-constrained CI.
///
/// Both runs use identical diagram sources so that any difference in wall time
/// reflects threading behaviour only, not content variance.
#[test]
fn test_threading_parallel_wall_time_reasonable() {
    // Identical diagram content used by both sequential and parallel runs.
    const DIAGRAM_COUNT: usize = 4;
    let diagram_sources: Vec<String> = (0..DIAGRAM_COUNT)
        .map(|i| simple_sequence(&format!("perf-{}", i)))
        .collect();

    // --- sequential baseline (1 thread) ---
    let src1 = TempDir::new().expect("Failed to create src1");
    let cache1 = TempDir::new().expect("Failed to create cache1");
    for (i, content) in diagram_sources.iter().enumerate() {
        create_puml(src1.path(), &format!("perf_{}.puml", i), content);
    }
    let start1 = Instant::now();
    let out1 = run_diagram_generate(src1.path(), cache1.path(), Some("1"), &[]);
    let elapsed1 = start1.elapsed();
    assert!(
        out1.status.success(),
        "Sequential baseline must succeed. STDERR: {}",
        String::from_utf8_lossy(&out1.stderr)
    );

    // --- parallel run (4 threads) ---
    let src4 = TempDir::new().expect("Failed to create src4");
    let cache4 = TempDir::new().expect("Failed to create cache4");
    for (i, content) in diagram_sources.iter().enumerate() {
        create_puml(src4.path(), &format!("perf_{}.puml", i), content);
    }
    let start4 = Instant::now();
    let out4 = run_diagram_generate(src4.path(), cache4.path(), Some("4"), &[]);
    let elapsed4 = start4.elapsed();
    assert!(
        out4.status.success(),
        "Parallel run must succeed. STDERR: {}",
        String::from_utf8_lossy(&out4.stderr)
    );

    // Parallel wall time must not exceed max(3× sequential, 30 s).
    // The 30-second floor prevents spurious failures when the sequential run
    // is extremely fast (e.g., sub-second on a warm cache), making 3× too tight.
    let threshold = (elapsed1 * 3).max(Duration::from_secs(30));
    assert!(
        elapsed4 <= threshold,
        "Parallel wall time ({:.2?}) must not exceed threshold ({:.2?} = max(3×{:.2?}, 30 s)). \
         Possible deadlock or severe contention.",
        elapsed4,
        threshold,
        elapsed1,
    );

    // Minimum divisor (1ms) avoids division-by-zero when elapsed1 rounds to 0.0.
    const MIN_DIVISOR_SECS: f64 = 0.001;
    eprintln!(
        "[perf] sequential={:.2?}  parallel(4 threads)={:.2?}  ratio={:.2}×",
        elapsed1,
        elapsed4,
        elapsed4.as_secs_f64() / elapsed1.as_secs_f64().max(MIN_DIVISOR_SECS),
    );
}
