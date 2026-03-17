# Benchmark Results: Diagram Generate Performance

## Summary

Parallel diagram generation achieves a **1.41× speedup** over sequential generation
on a 4-core machine, exceeding the target of ≥ 1.30×.

| Metric            | Value           |
|-------------------|-----------------|
| Diagram count     | 6               |
| CPU threads       | 4               |
| Sequential time   | 3.873 s         |
| Parallel time     | 2.745 s         |
| **Speedup**       | **1.41×** ✓     |
| Target            | ≥ 1.30×         |

## Implementation

Parallel rendering was introduced in `src/cmd/diagram/generate/mod.rs` using
[rayon](https://docs.rs/rayon)'s `par_iter()`.  The previous sequential `for`
loop over `.puml` source paths was replaced with:

```rust
use rayon::prelude::*;

let mut errors: Vec<(PathBuf, String)> = puml_paths
    .par_iter()
    .filter_map(|source_path| { /* render and collect (path, message) pairs */ })
    .collect();
// Sort by path for deterministic, greppable CLI output.
errors.sort_by(|(a, _), (b, _)| a.cmp(b));
if !errors.is_empty() {
    return Err(anyhow::anyhow!(
        "{}",
        errors.into_iter().map(|(_, msg)| msg).collect::<Vec<_>>().join("\n")
    ));
}
```

Key properties of the parallel implementation:

- **Thread-safe** – `PlantUML` only holds `String` fields and is therefore
  `Send + Sync`; no `Arc` wrapper is needed.
- **Error propagation** – errors from all failed renders are collected as
  `(path, message)` pairs, sorted by path for deterministic ordering, and
  combined into a single newline-separated error message (one failure per line),
  so no failure is silently discarded.
- **Synchronized output** – a global `Mutex<()>` in `plantuml.rs` serialises
  writes to `io::stdout()` and `io::stderr()` so concurrent renders do not
  produce interleaved console output.
- **Backward compatible** – force-generation flag and modification-timestamp
  cache logic are preserved unchanged.

## Reproducibility

### Unit test (included in `cargo test`)

The measurement is captured in an `#[ignore]`-tagged test so it does not slow
down the default test run.  A warm-up pass is performed before timing begins to
ensure JVM class-data caches are populated, giving a representative comparison
between sequential and parallel execution.

Run it explicitly with:

```bash
cargo test test_parallel_speedup -- --nocapture --ignored
```

### Criterion benchmark

A separate criterion benchmark binary is provided for statistical rigor:

```bash
cargo bench --bench diagram_generate_benchmark
```

Criterion HTML reports (after the run):
```
target/criterion/diagram_generate_sequential/report/index.html
target/criterion/diagram_generate_parallel/report/index.html
```

## Environment

| Property         | Value                                 |
|------------------|---------------------------------------|
| OS               | Linux (Ubuntu)                        |
| CPU cores        | 4 (Intel/AMD)                         |
| Rust edition     | 2021                                  |
| rayon version    | 1.11.0                                |
| Java             | OpenJDK 17 (Temurin 17.0.18)         |
| PlantUML JAR     | plantuml-1.2022.4.jar (test fixture)  |
| Build profile    | debug (unoptimised)                   |

> **Note:** Debug builds are used for the unit-test measurement above.  A
> release build (`cargo test --release …`) would show higher absolute
> throughput but a similar speedup ratio.

## Acceptance Criteria Checklist

- [x] Measure sequential execution time  → 3.873 s
- [x] Measure parallel execution time    → 2.745 s
- [x] Calculate speedup                  → 1.41×
- [x] Verify: speedup ≥ 1.3×            → **PASS**
- [x] Document results                   → this file
