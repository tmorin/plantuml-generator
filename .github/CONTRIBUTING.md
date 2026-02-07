# Contributing to plantuml-generator

We appreciate your thought to contribute to open source. :heart:

## Pull Request Requirements

When submitting a pull request, ensure the following requirements are met:

### 1. PR Title: Conventional Commits Format

The PR title must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>
```

**Valid types:**
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, missing semicolons, etc.)
- `refactor`: Code refactoring without changing functionality
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `build`: Changes to build system or dependencies
- `ci`: Changes to CI/CD configuration
- `chore`: Other changes that don't modify src or test files
- `revert`: Reverts a previous commit

**Examples of valid PR titles:**
```
✅ feat: add PlantUML C4 model support
✅ fix(parser): resolve null pointer exception in diagram parser
✅ ci: add automated PR validation
✅ feat!: breaking change to library API
```

**Why this matters:** When using GitHub's "Squash and merge" option, the PR title becomes the commit message in the main branch.

### 2. PR Description: Issue Reference Required

The PR description must reference an issue or ticket number. You can reference issues using:

- Standalone: `#123` or `GH-123`
- With keywords: `Closes #123`, `Fixes #456`, `Resolves #789`

**Keywords that automatically close issues:**
- `close`, `closes`, `closed`
- `fix`, `fixes`, `fixed`
- `resolve`, `resolves`, `resolved`

**Example PR description:**
```markdown
This PR adds automated validation for commit messages to ensure
consistency across the project.

Closes #123
```

**Why this matters:** When using "Squash and merge", the PR description is included in the commit body, providing traceability.

### 3. Merging: Use "Squash and Merge"

When merging PRs, use GitHub's **"Squash and merge"** button. This:
- Combines all commits into a single commit in the main branch
- Uses the PR title as the commit message
- Includes the PR description in the commit body
- Keeps the git history clean and linear

## Development Workflow

1. **Create a feature branch** from `main`
2. **Make your changes** with any number of commits (during development)
3. **Ensure the PR title follows Conventional Commits format**
4. **Ensure the PR description contains an issue reference**
5. **Ensure all CI checks pass** (including automated validation)
6. **Request review** from maintainers
7. **Address any review comments**
8. **Merge using "Squash and merge"** when approved

## Commit Messages During Development

During development, you can make commits with any message format that helps your workflow. However, we recommend following Conventional Commits even for development commits as a good practice.

**The important part is the PR title and description**, as these become the final commit message when squashed.

## Automated Checks

All PRs are automatically validated for:
- ✅ PR title follows Conventional Commits format
- ✅ PR description contains issue/ticket reference
- ✅ Code linting and tests pass

If any check fails, you'll see helpful error messages explaining how to fix the issue.

## General Guidelines

For little fixes or improvements, feel free to submit pull requests following the requirements above.

For larger changes, please [open a GitHub Issue](https://github.com/tmorin/plantuml-generator/issues) first to discuss the proposed changes.

## Questions?

If you have questions about these guidelines, please open an issue for discussion.
