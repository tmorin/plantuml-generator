# CLAUDE.md

**plantuml-generator** — a Rust CLI that generates PlantUML diagrams and library resources. Distributed as a binary, Docker image, and Debian package.

## Commit messages: Conventional Commits (required)

Format: `<type>(<scope>): <description>` — e.g. `fix(plantuml): handle missing java runtime`

- **Type** (required): `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`
- **Scope** (recommended): `cli`, `plantuml`, `library`, `diagram`, `workspace`, `urn`, `tera`, `deps`, `config`
- **Description**: imperative present tense, no leading capital, no trailing period, ≤50 chars
- **Breaking change**: add `!` after type/scope and a `BREAKING CHANGE:` footer
- Full spec: https://www.conventionalcommits.org/

## Architecture

```
src/
├── main.rs         # Entry point
├── lib.rs          # Library target (exposed for Criterion benchmarks)
├── app.rs          # Application orchestration
├── cli.rs          # CLI argument parsing (Clap)
├── cmd/            # Command implementations: library, diagram, workspace, completion
├── threading/      # Worker thread pool (config, pool, resource_monitor, errors, traits)
├── plantuml.rs     # PlantUML rendering and execution
├── tera.rs         # Template processing
├── urn.rs          # URN handling
├── utils.rs        # Utility functions
└── constants.rs
tests/              # End-to-end tests
benches/            # Criterion benchmarks (thread_pool, diagram_generate)
```

Runtime deps: Java (>=11), Inkscape (>=1.2) for library generation, libssl-dev/pkg-config. GraphViz `dot` is optional — diagrams fall back to the built-in smetana layout engine without it.

## Commands

```bash
cargo build --release              # optimized build
cargo run -- <command>             # run from source
cargo test                         # unit + integration tests
cargo test --test e2e_diagram_generate -- --nocapture   # one e2e suite, verbose
cargo bench --bench diagram_generate_benchmark
cargo bench --bench thread_pool_benchmark --features bench   # needs the "bench" feature
cargo fmt && cargo clippy -- -D warnings   # required before committing (CI enforces both)
```

Full CLI surface: `plantuml-generator --help` (library generate/schema, diagram generate, workspace init/install).

## Gotchas

- `cargo clippy` runs with `-D warnings` in CI — clippy warnings fail the build, not just lint.
- Cross-compiling or building without system OpenSSL needs `--features vendored-openssl`.
- `thread_pool_benchmark` won't build without `--features bench` (it exposes normally-private internals).
- `PLANTUML_IGNORE_DOT=1` simulates a missing GraphViz `dot` binary — useful for testing the smetana fallback path.

## Environment variables

| Variable | Purpose | Default |
|---|---|---|
| `PLANTUML_GENERATOR_THREADS` | Worker threads for the thread pool (diagram rendering, library processing), range 1–256, invalid/absent → CPU core count. Tuning tips in README "Multi-threading" | CPU core count |
| `RUST_LOG` | Logging level (`debug`, `info`, `warn`, or module-scoped e.g. `plantuml_generator::plantuml=debug`) | off |
| `GRAPHVIZ_DOT` | Path to a custom GraphViz `dot` binary | auto-detected |
| `PLANTUML_IGNORE_DOT` | Force the GraphViz-missing code path (dev/test only) | unset |
| `RUST_BACKTRACE` | `1` or `full` for panic backtraces | off |

## Release

```bash
convco version                              # check current version
cargo release "<version>" --no-publish --execute   # bumps version, tags, triggers CI/CD
```
CI builds Debian packages and Docker images automatically for tags across x86_64/aarch64/powerpc64le/s390x — see `.github/workflows/Continuous-Integration.yml` for the exact matrix, and `scripts/release-final.sh` for the release script.

## Elsewhere in this repo

- `README.md` — full command reference, multi-threading tuning tips, install instructions
- `.github/CONTRIBUTING.md` — contribution process (short: conventional commits + issues first)
- `Dockerfile` — container build
