# Developer Guide

## Prerequisites

### commitlint-rs

This project uses [commitlint-rs](https://github.com/KeisukeYamashita/commitlint-rs) to enforce Conventional Commit message format.

#### Installation

Install commitlint-rs using one of the following methods:

**Option 1: Cargo (Recommended)**
```bash
cargo install commitlint-rs
```

**Option 2: Cargo Binstall**
```bash
cargo binstall commitlint-rs
```

**Option 3: Docker**
```bash
docker pull 1915keke/commitlint
```

#### Verification

After installation, verify it's working:
```bash
commitlint --version
```

#### Git Hook Setup

After installing commitlint-rs, set up the git hook:

```bash
# Copy the hook template to .git/hooks/
cp hooks/commit-msg .git/hooks/commit-msg
chmod +x .git/hooks/commit-msg
```

The hook will automatically validate commit messages before they are created.

#### Commit Message Format

All commits must follow the Conventional Commits specification:

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, missing semi-colons, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `ci`: CI/CD changes
- `build`: Build system changes
- `perf`: Performance improvements
- `revert`: Revert a previous commit

**Examples:**
```bash
# Good commits
git commit -m "feat(cli): add new sync command"
git commit -m "fix(git): resolve branch detection issue"
git commit -m "docs: update installation instructions"

# Bad commits (will be rejected)
git commit -m "fixed stuff"
git commit -m "WIP"
git commit -m "Update code"
```

## Development Workflow

See [.claude/CLAUDE.md](.claude/CLAUDE.md) for PR best practices and workflow guidelines.
