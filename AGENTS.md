# AGENTS.md

## Project Overview

**plantuml-generator** is a command-line utility written in Rust that generates PlantUML diagrams and resources. It provides multiple commands for working with PlantUML, including library generation, diagram rendering, and workspace management. The project uses Cargo as its build system and is distributed as a binary, Docker image, and Debian package.

### Key Technologies

- **Language**: Rust (Edition 2021)
- **Build System**: Cargo
- **Package Manager**: Cargo
- **Testing Framework**: Rust's built-in test framework
- **Key Dependencies**: Clap (CLI), Serde (serialization), Tera (templating), Reqwest (HTTP), Image/Raster (image processing)
- **External Runtime Dependencies**: Java (>= 11), Inkscape (>= 1.2), libssl-dev, pkg-config
- **Optional Dependencies**: GraphViz (dot) for diagram layout

## System Architecture

```
plantuml-generator/
├── src/
│   ├── main.rs           # Entry point
│   ├── app.rs            # Application orchestration
│   ├── cli.rs            # CLI argument parsing (Clap)
│   ├── cmd/              # Command implementations (library, diagram, workspace)
│   ├── plantuml/         # PlantUML rendering and execution
│   ├── tera.rs           # Template processing
│   ├── urn.rs            # URN handling
│   ├── utils.rs          # Utility functions
│   └── constants.rs      # Constants
├── tests/                # End-to-end tests
├── Cargo.toml            # Package manifest
└── Dockerfile            # Container image
```

## Setup Commands

### Install Rust Toolchain

```bash
# Install Rust and Cargo if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Install System Dependencies (Ubuntu/Debian)

```bash
# Required dependencies
sudo apt-get update
sudo apt-get install -y pkg-config libssl-dev openjdk-11-jre

# Optional but recommended: Inkscape (for library generation)
sudo add-apt-repository ppa:inkscape.dev/stable -y
sudo apt-get update
sudo apt-get install -y inkscape

# Optional: GraphViz (for advanced diagram layout - auto-detected)
sudo apt-get install -y graphviz
```

### Install Development Dependencies

```bash
# Install cargo-deb for building Debian packages
cargo install cargo-deb

# Install cargo-binstall for easier binary installation
cargo install cargo-binstall

# Install convco and cargo-release for release management (optional)
cargo install convco cargo-release
```

### Install Project Dependencies

```bash
# No additional dependencies needed - Cargo handles everything
cargo build
```

## Development Workflow

### Start Development Build

```bash
# Build in debug mode (faster compilation)
cargo build

# Build in release mode (optimized binary)
cargo build --release
```

### Run the Binary

```bash
# From debug build
./target/debug/plantuml-generator --help

# From release build
./target/release/plantuml-generator --help
```

### Reload/Watch Mode

```bash
# Install cargo-watch if not already installed
cargo install cargo-watch

# Watch for changes and rebuild automatically
cargo watch -x build

# Watch and run tests automatically
cargo watch -x test
```

### Environment Setup

```bash
# Set RUST_LOG for debugging output
export RUST_LOG=debug

# Set GRAPHVIZ_DOT if using custom GraphViz installation (optional)
# export GRAPHVIZ_DOT=/path/to/dot

# Run with environment variable
RUST_LOG=debug cargo run -- diagram generate
```

### Available Commands

All commands follow the pattern: `plantuml-generator <COMMAND> [OPTIONS]`

```bash
# Library commands
plantuml-generator library generate [--manifest <FILE>]
plantuml-generator library schema

# Diagram commands
plantuml-generator diagram generate [OPTIONS]

# Workspace commands
plantuml-generator workspace init [--path <PATH>]
plantuml-generator workspace install [--artifact <NAME>]

# General help
plantuml-generator --help
plantuml-generator <COMMAND> --help
```

## Testing Instructions

### Run All Tests

```bash
# Run all tests (unit and integration)
cargo test

# Run tests with output from passing tests
cargo test -- --nocapture

# Run tests with specific log level
RUST_LOG=debug cargo test

# Run tests sequentially (useful for debugging)
cargo test -- --test-threads=1
```

### Run Specific Test Categories

```bash
# Run unit tests only (in src/)
cargo test --lib

# Run integration/end-to-end tests
cargo test --test '*'

# Run a specific test by name
cargo test test_diagram_generate

# Run tests matching a pattern
cargo test diagram --
```

### End-to-End Tests

```bash
# Located in tests/ directory
# Run with verbose output to see what's happening
RUST_LOG=debug cargo test --test e2e_diagram_generate -- --nocapture
```

### Test Coverage and Validation

```bash
# Run tests with backtrace enabled
RUST_BACKTRACE=1 cargo test

# Run tests and print all output
cargo test -- --nocapture --test-threads=1

# Verify the build passes all tests
cargo test --release
```

## Code Style and Conventions

### Rust Conventions

- **Edition**: 2021
- **Formatting**: Use `cargo fmt` for automatic formatting
- **Linting**: Clippy with strict warnings enforced
- **Naming**: Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)

### Formatting

```bash
# Format all code
cargo fmt

# Check formatting without making changes
cargo fmt -- --check

# Format specific file
cargo fmt -- <file>
```

### Linting with Clippy

```bash
# Run Clippy checks (same as CI)
cargo clippy -- -D warnings

# Run Clippy with all checks
cargo clippy --all-targets --all-features -- -D warnings

# Fix some issues automatically
cargo clippy --fix
```

### Code Organization

- **Modules**: Located in `src/` with clear separation by function (cmd, plantuml, tera, urn, utils)
- **Main entry point**: `src/main.rs` - keeps it simple
- **CLI parsing**: `src/cli.rs` using Clap
- **Command implementations**: `src/cmd/` directory, one module per command group
- **Tests**: Located alongside code with `#[cfg(test)]` or in `tests/` for integration tests

### Comments and Documentation

- Add comments to explain "why", not "what"
- Use doc comments (`///`) for public API documentation
- Keep comments concise and clear
- Document non-obvious algorithm choices

Example:
```rust
/// Generates a PlantUML diagram from the given source file.
///
/// # Arguments
/// * `source_path` - Path to the .puml source file
///
/// # Returns
/// Result containing the generated diagram or error
pub fn generate_diagram(source_path: &Path) -> Result<Diagram> {
    // Implementation
}
```

## Build and Deployment

### Build Targets

```bash
# Build for current target (default: x86_64-unknown-linux-gnu)
cargo build --release

# Build for specific target (requires target installation)
rustup target add <target>
cargo build --release --target=<target>

# Supported targets (see CI/CD for full list):
# - x86_64-unknown-linux-gnu
# - aarch64-unknown-linux-gnu
# - powerpc64le-unknown-linux-gnu
# - s390x-unknown-linux-gnu
```

### Cross-Compilation

```bash
# Install cross for cross-compilation
cargo binstall --no-confirm cross

# Build with vendored OpenSSL (for compatibility)
cross build --release --target=<target> --features vendored-openssl
```

### Build Artifacts

#### Binary Archive

```bash
# Create release binary
cargo build --release

# Create archive
tar -C target/release -czf linux_x86_64_plantuml-generator.tar.gz plantuml-generator
```

#### Debian Package

```bash
# Install cargo-deb
cargo install cargo-deb

# Build .deb package
cargo deb --target=x86_64-unknown-linux-gnu --profile release

# With vendored OpenSSL
cargo deb --target=x86_64-unknown-linux-gnu --profile release -- --features vendored-openssl

# Package location: target/x86_64-unknown-linux-gnu/debian/*.deb
```

#### Docker Image

```bash
# Build locally
docker build -t plantuml-generator:local .

# Build with specific build args (optional)
docker build --build-arg git_sha=$(git rev-parse HEAD) -t plantuml-generator:local .

# Run container
docker run --rm plantuml-generator:local --help
docker run --rm -v $(pwd)/diagrams:/diagrams plantuml-generator:local diagram generate /diagrams
```

### Release Process

```bash
# 1. Install release tools
cargo install convco cargo-release

# 2. Check current version
convco version

# 3. Bump version and create release
cargo release "<version>" --no-publish --execute

# This triggers CI/CD pipeline via git tag
```

## Pull Request Guidelines

### Title Format

Prefix your PR title with the component or feature:

```
[component] Brief description of changes

Examples:
[cli] Add new diagram format option
[plantuml] Improve error handling for invalid diagrams
[docs] Update library generation documentation
```

### Required Checks Before Submission

```bash
# 1. Run formatter
cargo fmt

# 2. Run linter (must pass without warnings)
cargo clippy -- -D warnings

# 3. Run all tests
cargo test

# 4. Verify release build compiles
cargo build --release

# Quick check all required steps
cargo fmt && cargo clippy -- -D warnings && cargo test && cargo build --release
```

### Review Process

- All PRs require passing CI/CD checks (lint, test, build)
- Clippy warnings are treated as errors (`-D warnings`)
- All tests must pass before merge
- Docker and binary artifacts are built automatically for tags

### Commit Message Conventions

Follow [Conventional Commits](https://www.conventionalcommits.org/) specification strictly.

**Format**: `<type>(<scope>): <description>`

**Type** (required):
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, missing semicolons, etc.)
- `refactor`: Code refactoring without feature changes or bug fixes
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Build process, dependencies, tooling
- `ci`: CI/CD configuration changes

**Scope** (optional): Component or module affected (e.g., cli, plantuml, library, diagram)

**Description** (required):
- Use imperative mood: "add feature" not "added feature"
- Do not capitalize first letter
- No period at the end
- Maximum 50 characters

**Body** (optional): Provide detailed explanation if needed
- Explain what and why, not how
- Separate from description with blank line
- Wrap at 72 characters

**Footer** (optional): Reference issues and breaking changes
- `Fixes #123`
- `BREAKING CHANGE: description`

**Examples**:
```
feat(cli): add custom PlantUML arguments support

Allow users to pass custom arguments to PlantUML via --args flag.
This enables advanced layout engines and other PlantUML features.

Fixes #456
```

```
fix(plantuml): handle missing Java runtime gracefully

Improve error message when Java is not installed or not in PATH.
Suggest installation steps to users.
```

```
test(diagram): add integration tests for diagram generation

Add comprehensive end-to-end tests covering various diagram types
and edge cases.
```

```
docs: update setup instructions for Inkscape
```

## Security Considerations

### Secrets Management

- Do not commit credentials or API keys
- Use environment variables for sensitive data
- GitHub secrets are used for Docker Hub and artifact uploads in CI/CD

### Build Security

- OpenSSL is vendored for security updates (`vendored-openssl` feature)
- Dependencies are locked via `Cargo.lock`
- Security advisories can be checked with: `cargo audit`

### Input Validation

- All file paths should be validated before use
- User input from CLI is parsed by Clap (validates automatically)
- YAML/JSON parsing uses serde with validation

## Debugging and Troubleshooting

### Enable Debug Logging

```bash
# Run with debug logging
RUST_LOG=debug cargo run -- <command>

# More verbose logging
RUST_LOG=trace cargo run -- <command>

# Log specific modules
RUST_LOG=plantuml_generator::plantuml=debug cargo run -- diagram generate
```

### Common Issues

**Issue: Missing Java runtime**
```bash
# Check Java installation
java -version

# Install if needed
sudo apt-get install openjdk-11-jre
```

**Issue: Missing Inkscape (for library generation)**
```bash
# Check Inkscape
inkscape --version

# Install
sudo apt-get install inkscape
```

**Issue: OpenSSL errors during build**
```bash
# Use vendored OpenSSL feature
cargo build --release --features vendored-openssl
```

**Issue: Test failures related to graphviz**
```bash
# Tests can run without GraphViz - uses smetana engine by default
# If you want to test with GraphViz:
sudo apt-get install graphviz
```

### Performance Considerations

- Release builds are significantly faster than debug builds
- Use `cargo build --release` for benchmarking
- LTO (Link-Time Optimization) is enabled in release profile
- Single codegen unit in release profile for maximum optimization

### Backtrace for Debugging

```bash
# Enable Rust backtrace for panic debugging
RUST_BACKTRACE=1 cargo run -- <command>

# Full backtrace
RUST_BACKTRACE=full cargo run -- <command>
```

## Monorepo Notes

This is a single-package Rust project. All code is in the main `src/` directory with clear module separation.

## Environment Variables

| Variable | Purpose | Default | Example |
|----------|---------|---------|---------|
| `RUST_LOG` | Logging level | (off) | `debug`, `info`, `warn` |
| `GRAPHVIZ_DOT` | Path to GraphViz dot binary | (auto-detected) | `/usr/bin/dot` |
| `PLANTUML_IGNORE_DOT` | Simulate missing GraphViz dot for testing/development only | (unset) | `1` |
| `RUST_BACKTRACE` | Enable panic backtrace | (off) | `1`, `full` |

## Additional Resources

- **Repository**: https://github.com/tmorin/plantuml-generator
- **Issue Tracker**: https://github.com/tmorin/plantuml-generator/issues
- **PlantUML Documentation**: https://plantuml.com
- **Rust Book**: https://doc.rust-lang.org/book/
- **Cargo Documentation**: https://doc.rust-lang.org/cargo/
- **Clippy Lints**: https://rust-lang.github.io/rust-clippy/

## Maintenance Notes

- The project uses GitHub Actions for CI/CD (see `.github/workflows/`)
- Releases are automated via git tags and trigger builds for multiple architectures
- Both Debian packages and Docker images are built automatically on release
- The `release-final.sh` script handles version bumping and release creation
