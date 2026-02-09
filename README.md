# git-utils

Git utilities for managing repositories and branches interactively.

## Features

- **git-branch-switch**: Interactive branch switcher with inquire
- **git-branch-delete**: Safe branch deletion with merge status checking
- **git-repo**: Repository management (clone, list) with organized directory structure
- **git-utils**: Setup and configuration tool

## Installation

### From source

```bash
cargo install --path crates/git-branch-switch
cargo install --path crates/git-branch-delete
cargo install --path crates/git-repo
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

## Configuration

### Git config

Add recommended settings to your `.gitconfig`:

```bash
git-utils setup --gitconfig >> ~/.gitconfig
```

Or manually add:

```ini
[git-repo]
    root = ~/src
    prefer-ssh = true

[git-branch-delete]
    base = main

[alias]
    bs = !git-branch-switch
    bd = !git-branch-delete
    repo = !git-repo
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
git repo clone https://github.com/user/repo
# → Clones to ~/src/github.com/user/repo

# List repositories
git repo ls

# List with details
git repo ls --long

# Show dirty repositories only
git repo ls --dirty

# Switch between repositories (shell function)
grs
```

## Development

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

### Format and lint

```bash
cargo fmt
cargo clippy
```

## License

MIT