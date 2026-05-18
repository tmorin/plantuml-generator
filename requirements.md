# Requirements

## Scope
Upgrade project dependencies for issue #106 while keeping existing behavior intact.

## EARS Requirements

1. WHEN dependency versions are upgraded in `Cargo.toml`, THE SYSTEM SHALL resolve and lock compatible versions in `Cargo.lock`.
2. WHEN upgraded dependencies introduce API changes, THE SYSTEM SHALL update impacted source and test code so compilation succeeds.
3. WHEN the dependency upgrade is completed, THE SYSTEM SHALL pass formatting, linting, tests, and release build checks.
4. IF dependency APIs change behavior for archive extraction or hashing, THEN THE SYSTEM SHALL preserve current functional behavior.
