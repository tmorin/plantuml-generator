# Design

## Architecture Impact
Dependency updates are constrained to Cargo manifests and lockfile, with minimal code adaptations where upstream crate APIs changed.

## Planned Changes
1. Bump dependency versions in `Cargo.toml`:
   - `zip` from `0.6` to `8`
   - `criterion` from `0.5` to `0.8`
   - `sha2` from `0.10` to `0.11`
2. Regenerate `Cargo.lock` with `cargo update`.
3. Apply API compatibility fixes:
   - Zip API: fallback for `enclosed_name()` now uses `PathBuf::new`.
   - SHA-2 formatting in tests: convert digest bytes to lowercase hex manually.

## Error Handling
- Keep existing contextual errors in workspace install path handling (`anyhow::Context`).
- Maintain deterministic checksum generation in tests with explicit byte-to-hex conversion.

## Validation Strategy
- `cargo fmt`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo build --release`
