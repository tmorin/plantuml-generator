# GitHub Copilot Instructions

## Commit Message Guidelines

**ALL commit messages MUST strictly follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.**

### Mandatory Commit Format

Every commit message must follow this format:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Commit Types (REQUIRED)

Choose ONE of the following types for EVERY commit:

- **feat**: A new feature for the user
- **fix**: A bug fix
- **docs**: Documentation changes only
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes that affect the build system or external dependencies (example scopes: cargo, npm, docker)
- **ci**: Changes to CI configuration files and scripts (example scopes: github-actions, workflows)
- **chore**: Other changes that don't modify src or test files
- **revert**: Reverts a previous commit

### Scope (OPTIONAL but RECOMMENDED)

The scope provides additional contextual information. Common scopes for this project:

- `cli`: Command-line interface changes
- `plantuml`: PlantUML rendering and execution
- `library`: Library generation functionality
- `diagram`: Diagram generation functionality
- `workspace`: Workspace management
- `urn`: URN handling
- `tera`: Template processing
- `deps`: Dependency updates
- `config`: Configuration files

### Description (REQUIRED)

The description is a short summary of the code changes:

- **Use the imperative, present tense**: "change" not "changed" nor "changes"
- **Don't capitalize the first letter**
- **No period (.) at the end**
- **Maximum 50 characters** (aim for clarity and brevity)

### Examples of Valid Commits

✅ **Good examples:**
```
feat(cli): add support for custom PlantUML arguments
fix(plantuml): handle missing Java runtime gracefully
docs: update installation instructions for Inkscape
refactor(urn): simplify URN parsing logic
test(diagram): add integration tests for PNG generation
chore(deps): bump serde from 1.0.1 to 1.0.2
ci: add workflow for multi-arch builds
```

❌ **Bad examples (DO NOT USE):**
```
Add new feature                     # Missing type and format
Fixed bug.                          # Missing type, capitalized, has period
feat: Adding new CLI option         # Wrong tense ("Adding" instead of "add")
Feat(cli): New option              # Capitalized description
feat(cli): added a new option.     # Wrong tense and has period
update docs                         # Missing type prefix
```

### Breaking Changes

If a commit introduces a breaking change, it MUST be indicated in two ways:

1. Add `!` after the type/scope: `feat(api)!: remove deprecated endpoints`
2. Include `BREAKING CHANGE:` in the footer with description

Example:
```
feat(cli)!: change default output format to SVG

Previously, the default output format was PNG. This change sets
the default to SVG for better quality.

BREAKING CHANGE: Users relying on PNG as default output must now
explicitly specify --format png
```

### Multi-line Commit Messages

For complex changes, use the optional body and footer:

```
feat(library): add support for custom icon repositories

This change allows users to specify custom icon repositories
in the library manifest. The system will download and process
icons from multiple sources.

Resolves #123
Relates to #456
```

### Commit Message Validation

Before making any commit:

1. ✅ Verify the type is from the approved list
2. ✅ Check the description uses imperative mood
3. ✅ Ensure the description doesn't exceed 50 characters
4. ✅ Confirm no capitalization or period at the end
5. ✅ Validate the scope (if used) is relevant to the project

### Why This Matters

Conventional commits enable:

- **Automatic changelog generation**: Tools can parse commits to create meaningful changelogs
- **Semantic versioning**: Automated version bumping based on commit types
- **Clear history**: Easy to scan and understand project evolution
- **Better collaboration**: Consistent format across all contributors
- **Squash and merge**: When PRs are squashed, the resulting commit maintains conventional format

## General Coding Guidelines

When contributing code:

1. Follow existing code style and conventions (see AGENTS.md)
2. Write comprehensive tests for new features
3. Update documentation for user-facing changes
4. Run `cargo fmt` and `cargo clippy` before committing
5. Ensure all tests pass with `cargo test`

## References

- [Conventional Commits Specification](https://www.conventionalcommits.org/)
- [Contributing Guide](.github/CONTRIBUTING.md)
- [Project Documentation](AGENTS.md)
