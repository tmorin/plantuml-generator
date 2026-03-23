# Multi-threading in plantuml-generator

`plantuml-generator` processes diagrams and library items in parallel using a
built-in thread pool backed by OS threads. This document describes the
threading model, configuration options, and usage examples.

## Overview

Both `diagram generate` and `library generate` commands distribute their work
units across a configurable pool of worker threads. The default thread count
equals the number of logical CPU cores reported by the operating system,
giving near-linear throughput improvements for large batches of diagrams.

## Configuration

### Environment variable: `PLANTUML_GENERATOR_THREADS`

| Attribute | Value |
|---|---|
| Variable name | `PLANTUML_GENERATOR_THREADS` |
| Accepted values | Integer between **1** and **256** (inclusive) |
| Default | Number of logical CPU cores |
| Fallback | CPU core count (used when the variable is absent or invalid) |

Invalid values (non-integers, zero, values above 256) produce a warning log
message and fall back to the default automatically — the tool never exits due
to a bad thread-count value.

## Usage Examples

### Use the default thread count (CPU cores)

```shell
plantuml-generator diagram generate -s ./diagrams
```

No configuration is needed. The tool detects the CPU core count at startup and
uses it as the worker thread count.

### Set a fixed thread count

```shell
PLANTUML_GENERATOR_THREADS=8 plantuml-generator diagram generate -s ./diagrams
```

Or export for the entire shell session:

```shell
export PLANTUML_GENERATOR_THREADS=8
plantuml-generator diagram generate -s ./diagrams
plantuml-generator library generate library.yaml
```

### Single-threaded mode (sequential, useful for debugging)

```shell
PLANTUML_GENERATOR_THREADS=1 plantuml-generator diagram generate -s ./diagrams
```

With a single thread the log output is free of interleaving, which makes it
easier to trace exactly what the tool is doing.

### Docker usage

Pass the environment variable through Docker's `-e` flag:

```shell
docker run --rm \
  -e PLANTUML_GENERATOR_THREADS=4 \
  -v "$(pwd)/diagrams:/diagrams" \
  thibaultmorin/plantuml-generator \
  diagram generate -s /diagrams
```

### CI / GitHub Actions

```yaml
- name: Generate diagrams
  env:
    PLANTUML_GENERATOR_THREADS: 4
  run: plantuml-generator diagram generate -s ./diagrams
```

## Performance Tips

### CPU-bound work (typical diagram rendering)

Diagram rendering is CPU-bound because PlantUML spawns a JVM process for each
file. The optimal thread count is equal to the number of logical CPU cores
(the default). Adding threads beyond the core count increases scheduling
overhead without improving throughput.

```shell
# Let the tool auto-detect (recommended for most cases)
plantuml-generator diagram generate -s ./diagrams
```

### Large batches of small diagrams

For hundreds or thousands of small diagrams, keeping `PLANTUML_GENERATOR_THREADS`
at the core count still gives the best results. The thread pool automatically
caps the number of active workers to the number of pending work units, so you
never pay the cost of idle threads.

### I/O-bound scenarios

If diagrams are stored on a network file system or slow storage, threads spend
time waiting on I/O rather than using the CPU. In this case a thread count
2–4× the core count can improve throughput:

```shell
PLANTUML_GENERATOR_THREADS=16 plantuml-generator diagram generate -s /mnt/nfs/diagrams
```

### Memory-constrained environments

Each worker thread may launch a JVM subprocess during rendering. If you
encounter out-of-memory errors, reduce the thread count:

```shell
PLANTUML_GENERATOR_THREADS=2 plantuml-generator diagram generate -s ./diagrams
```

### Benchmarking

Use the `RUST_LOG=info` environment variable to print per-file timing
information. Combined with a fixed `PLANTUML_GENERATOR_THREADS` value this
lets you measure speedup relative to sequential execution:

```shell
# Sequential baseline
RUST_LOG=info PLANTUML_GENERATOR_THREADS=1 \
  plantuml-generator diagram generate -s ./diagrams

# Parallel run with 8 threads
RUST_LOG=info PLANTUML_GENERATOR_THREADS=8 \
  plantuml-generator diagram generate -s ./diagrams
```

## Architecture Notes

The threading module (`src/threading/`) provides:

- **`Config`** — reads `PLANTUML_GENERATOR_THREADS` from the environment and
  validates the value.
- **`ThreadPool`** — spawns worker threads, distributes work via
  `std::sync::mpsc` channels, and collects errors.
- **`WorkUnit`** trait — implemented by each parallelizable task (e.g. a single
  diagram render or a library item generation step).
- **`AggregatedError`** — collects all per-work-unit failures so the tool
  reports every failure rather than stopping at the first one.

Stdout/stderr writes are serialised with a global mutex so log lines from
concurrent workers do not interleave.

## API Documentation

Generate the full Rust API documentation locally:

```shell
cargo doc --no-deps --open
```

This opens the rendered documentation in your browser. The `threading` module
entry point (`src/threading/mod.rs`) contains detailed doc-comments covering
all public types and their usage.
