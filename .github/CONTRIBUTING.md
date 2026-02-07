# Contributing to plantuml-generator

We appreciate your thought to contribute to open source. :heart:

## Commit Message Requirements

All commits must follow these requirements:

### 1. Conventional Commits Format

Commits must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
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

**Examples:**
```
feat: add PlantUML C4 model support
fix(parser): resolve null pointer exception in diagram parser
docs: update installation instructions
feat!: breaking change to library API
```

### 2. Issue/Ticket References

Every commit must reference an issue or ticket number. You can reference issues in:

- The commit subject: `feat: add feature #123`
- The commit body or footer using keywords:
  ```
  feat: add new diagram type
  
  This implements support for sequence diagrams.
  
  Closes #123
  ```

**Keywords that automatically close issues:**
- `close`, `closes`, `closed`
- `fix`, `fixes`, `fixed`
- `resolve`, `resolves`, `resolved`

### 3. Single Commit per PR

Before a PR can be merged, it should contain only **one commit**. This keeps the git history clean and makes it easier to track changes.

**To squash multiple commits:**

```bash
# Method 1: Interactive rebase
git rebase -i HEAD~n  # where n is the number of commits
# Mark first commit as 'pick', others as 'squash' or 'fixup'
git push --force-with-lease

# Method 2: Soft reset
git reset --soft HEAD~n  # where n is the number of commits
git commit -m "your-combined-commit-message"
git push --force-with-lease
```

Alternatively, use GitHub's "Squash and merge" button when merging.

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes with commits following the guidelines above
3. Squash commits into a single commit before requesting review
4. Ensure all CI checks pass
5. Request review from maintainers
6. Address any review comments
7. Once approved, the PR will be merged

## Automated Checks

All PRs are automatically validated for:
- ✅ Conventional Commits format
- ✅ Issue/ticket references
- ✅ Single commit requirement
- ✅ Code linting and tests

If any check fails, you'll see helpful error messages explaining how to fix the issue.

## General Guidelines

For little fixes or improvements, feel free to submit pull requests following the commit message requirements above.

For larger changes, please [open a GitHub Issue](https://github.com/tmorin/plantuml-generator/issues) first to discuss the proposed changes.

## Questions?

If you have questions about these guidelines, please open an issue for discussion.
