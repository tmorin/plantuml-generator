//! Benchmark: thread pool parallelization speedup for library generation workloads.
//!
//! This benchmark measures the performance improvement from parallelization in the
//! library generation pipeline. It simulates the CPU-bound work (template rendering,
//! image processing) performed during library generation using synthetic tasks.
//!
//! # Benchmark Groups
//!
//! - `sequential` (1 thread): Baseline for speedup calculation
//! - `parallel_4` (4 threads): Typical multi-core configuration
//! - `parallel_8` (8 threads): High core-count configuration
//!
//! # Expected Results
//!
//! The parallelization target is a speedup of **≥ 1.4× (40% faster)** compared to
//! sequential execution. In practice, CPU-bound workloads with independent tasks
//! achieve near-linear speedup up to the number of available CPU cores.
//!
//! # Running
//!
//! ```bash
//! cargo bench --bench thread_pool_benchmark
//! ```
//!
//! Results are saved in `target/criterion/` as HTML reports.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use plantuml_generator::threading::{Config, ThreadPool, WorkUnit};
use std::hint::black_box;

// ---------------------------------------------------------------------------
// Synthetic CPU-bound task that simulates library generation work
// ---------------------------------------------------------------------------

/// A synthetic work unit that performs CPU-bound computation simulating
/// the kind of work done during library generation (template rendering,
/// string manipulation, image processing).
struct CpuBoundTask {
    id: usize,
    /// Number of iterations of CPU work to perform per task.
    work_units: u32,
}

impl WorkUnit for CpuBoundTask {
    fn identifier(&self) -> String {
        format!("cpu_task_{}", self.id)
    }

    fn execute(&self) -> Result<(), String> {
        // Simulate CPU-bound work similar to template rendering and string
        // manipulation performed in library generation tasks.
        let mut acc: u64 = self.id as u64;
        for i in 0..self.work_units {
            // Mix of arithmetic that prevents compiler from optimizing away.
            acc = black_box(acc.wrapping_mul(6364136223846793005).wrapping_add(i as u64));
            acc ^= acc >> 32;
        }
        // Ensure the result is used so the compiler doesn't eliminate the work.
        let _ = black_box(acc);
        Ok(())
    }
}

/// Creates a vector of CPU-bound work units for benchmarking.
fn make_tasks(count: usize, work_units_per_task: u32) -> Vec<Box<dyn WorkUnit>> {
    (0..count)
        .map(|i| {
            Box::new(CpuBoundTask {
                id: i,
                work_units: work_units_per_task,
            }) as Box<dyn WorkUnit>
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Benchmark: sequential vs parallel throughput
// ---------------------------------------------------------------------------

/// Benchmarks the thread pool with varying thread counts.
///
/// Uses 64 tasks with ~50k CPU iterations each — representative of a small
/// library generation batch (e.g., rendering 64 templates).
fn bench_thread_pool_throughput(c: &mut Criterion) {
    const TASK_COUNT: usize = 64;
    const WORK_PER_TASK: u32 = 50_000;

    let mut group = c.benchmark_group("library_generate_throughput");
    group.throughput(Throughput::Elements(TASK_COUNT as u64));

    for thread_count in [1, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("threads", thread_count),
            &thread_count,
            |b, &tc| {
                b.iter(|| {
                    let pool = ThreadPool::new(Config::new(tc));
                    let tasks = make_tasks(TASK_COUNT, WORK_PER_TASK);
                    pool.execute(tasks).expect("benchmark tasks must not fail");
                });
            },
        );
    }

    group.finish();
}

/// Benchmarks wall-clock time for 1 vs 4 vs 8 threads on a larger workload.
///
/// Uses 128 tasks simulating a medium library generation scenario.
fn bench_thread_pool_wall_time(c: &mut Criterion) {
    const TASK_COUNT: usize = 128;
    const WORK_PER_TASK: u32 = 25_000;

    let mut group = c.benchmark_group("library_generate_wall_time");

    for thread_count in [1, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("threads", thread_count),
            &thread_count,
            |b, &tc| {
                b.iter(|| {
                    let pool = ThreadPool::new(Config::new(tc));
                    let tasks = make_tasks(TASK_COUNT, WORK_PER_TASK);
                    pool.execute(tasks).expect("benchmark tasks must not fail");
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_thread_pool_throughput,
    bench_thread_pool_wall_time
);
criterion_main!(benches);
