# git-utils

Git utilities for managing repositories and branches interactively.

## Features

- **git-branch-switch**: Interactive branch switcher with inquire
- **git-branch-delete**: Safe branch deletion with merge status checking
- **git-repos**: Repository management (clone, list) with organized directory structure
- **git-pr-merged**: List merged pull requests for release notes
- **git-utils**: Setup and configuration tool

## Installation

### Prerequisites

This project uses static linking for all dependencies, so you **do not need** to install libgit2 or other system libraries at runtime.

However, for **building from source**, you need:

- Rust toolchain (stable)
- CMake (for building vendored libgit2)
- C/C++ compiler (gcc, clang, or MSVC)
- pkg-config (on Unix-like systems)

On macOS:

```bash
brew install cmake pkg-config
```

On Ubuntu/Debian:

```bash
apt-get install cmake pkg-config build-essential
```

### From source

```bash
cargo install --path crates/git-branch-switch
cargo install --path crates/git-branch-delete
cargo install --path crates/git-repos
cargo install --path crates/git-pr-merged
cargo install --path crates/git-utils
```

### Setup

After installation, run the setup command:

```bash
git-utils setup
```

This will:

- Create `~/.git-utils/env.sh` and `~/.git-utils/env.fish`
- Add source lines to your shell rc file
- Set up the `grs` (git repository switch) shell function
- Export `GIT_REPOS_ROOT` environment variable (default: `~/src`)

**Note**: `git-repos` commands will work immediately after setup using the `GIT_REPOS_ROOT` environment variable. You can optionally configure `git-repos.root` in your `.gitconfig` for more control.

## Configuration

### Repository root

`git-repos` uses the following priority to determine the repository root:

1. Git config: `git-repos.root`
2. Environment variable: `GIT_REPOS_ROOT` (set by `git-utils setup`)

Choose one of:

```bash
# Option 1: Use environment variable (automatic after setup)
# Already set by git-utils setup in ~/.git-utils/env.sh

# Option 2: Use git config (optional, overrides environment variable)
git config --global git-repos.root ~/src
```

### Git config

Add recommended settings to your `.gitconfig`:

```bash
git-utils setup --gitconfig >> ~/.gitconfig
```

Or manually add:

```ini
[git-repos]
    root = ~/src
    prefer-ssh = true

[git-branch-delete]
    base = main

[alias]
    bs = !git-branch-switch
    bd = !git-branch-delete
    repos = !git-repos
    pr-merged = !git-pr-merged
```

## Usage

### Branch switching

```bash
# Interactive branch selection
git branch-switch

# Show recent branches
git branch-switch --recent

# Filter by pattern
git branch-switch feature/

# Show only merged branches
git branch-switch --merged
```

### Branch deletion

```bash
# Delete merged branches (safe)
git branch-delete

# Select branches interactively
git branch-delete --select

# Force delete all branches
git branch-delete --all --force
```

### Repository management

```bash
# Clone to organized directory structure
git repos clone https://github.com/user/repo
# → Clones to ~/src/github.com/user/repo

# List repositories
git repos ls

# List with details
git repos ls --long

# Show dirty repositories only
git repos ls --dirty

# Switch between repositories (shell function)
grs
```

### Pull request listing

```bash
# List PRs merged since latest tag
git pr-merged

# List PRs in specific range
git pr-merged v1.0.0..v1.1.0

# List PRs from last 10 commits
git pr-merged -n 10

# Output as JSON
git pr-merged --format json

# Output as Markdown (for release notes)
git pr-merged v1.0.0..HEAD --format markdown > CHANGELOG.md

# Open PRs in browser
git pr-merged v1.0.0..HEAD --web
```

## Development

### Prerequisites

Install cargo-make for task automation and ensure the required Rust components are installed:

```bash
# Install cargo-make (task runner)
cargo install cargo-make

# Install required Rust components
rustup component add rustfmt clippy

# Install nightly toolchain (required for cargo-udeps)
rustup toolchain install nightly
```

**Note**: Development tools (cargo-machete, cargo-udeps, rumdl) are automatically installed by cargo-make when you run tasks. No manual installation needed.

### Build

```bash
cargo build
```

### Test and Quality Checks

We use cargo-make for running tests and quality checks:

```bash
# Run all tests
cargo make test

# Check code formatting
cargo make fmt

# Fix code formatting
cargo make fmt-fix

# Run clippy linter
cargo make clippy

# Check for unused dependencies (fast, uses cargo-machete)
cargo make check-unused-dependencies

# Check for unused dependencies (slow but thorough, uses cargo-udeps)
cargo make check-unused-dependencies-udeps

# Check markdown formatting
cargo make markdown-fmt

# Fix markdown formatting
cargo make markdown-fmt-fix

# Run all checks (format, clippy, test, unused dependencies, markdown)
cargo make check-all
```

### Git Hooks

Install git hooks to automatically run checks before commit and push:

```bash
cargo make install-hooks
```

This installs:

- **pre-commit hook**: Runs `cargo make check-all` before each commit
- **pre-push hook**: Runs `cargo make check-all` before each push

The hooks ensure code quality by running format checks, clippy, and tests automatically. If checks fail, the commit/push will be blocked.

**Note**: Git hooks require `/bin/bash`. On Windows, ensure you have Git Bash installed (included with Git for Windows).

To uninstall hooks:

```bash
cargo make uninstall-hooks
```

To bypass hooks (not recommended):

```bash
git commit --no-verify
git push --no-verify
```

## License

MIT

This software statically links third-party libraries. See [THIRD-PARTY-LICENSES.md](THIRD-PARTY-LICENSES.md) for the complete list of dependencies and their licenses.
