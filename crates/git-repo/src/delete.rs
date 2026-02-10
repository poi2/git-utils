use anyhow::{anyhow, Result};
use git2::Repository;
use inquire::{Confirm, Select};
use std::fs;
use std::path::PathBuf;

use crate::utils::get_repo_root;

pub fn delete_repo(
    repo_path: Option<String>,
    interactive: bool,
    force: bool,
    dry_run: bool,
) -> Result<()> {
    let repo_root = get_repo_root()?;

    if !repo_root.exists() {
        return Err(anyhow!(
            "Repository root does not exist: {}",
            repo_root.display()
        ));
    }

    let target_path = if interactive {
        // Interactive selection
        select_repo_interactive(&repo_root)?
    } else if let Some(path) = repo_path {
        // Direct specification
        let full_path = repo_root.join(&path);
        if !full_path.exists() {
            return Err(anyhow!("Repository not found: {}", path));
        }
        full_path
    } else {
        return Err(anyhow!(
            "Either specify a repository path or use --interactive"
        ));
    };

    let relative_path = target_path
        .strip_prefix(&repo_root)
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Check if it's a git repository
    if !target_path.join(".git").exists() {
        return Err(anyhow!("Not a git repository: {}", relative_path));
    }

    // Open repository to check status
    let repo = Repository::open(&target_path)?;

    // Safety checks (unless --force)
    if !force {
        // Check for uncommitted changes
        if has_uncommitted_changes(&repo) {
            eprintln!("Warning: Repository has uncommitted changes");
            if !dry_run {
                let answer = Confirm::new("Continue anyway?")
                    .with_default(false)
                    .prompt()?;
                if !answer {
                    return Ok(());
                }
            }
        }

        // Check for unpushed commits
        if has_unpushed_commits(&repo)? {
            eprintln!("Warning: Repository has unpushed commits");
            if !dry_run {
                let answer = Confirm::new("Continue anyway?")
                    .with_default(false)
                    .prompt()?;
                if !answer {
                    return Ok(());
                }
            }
        }
    }

    if dry_run {
        println!("Would delete: {}", relative_path);
        println!("Path: {}", target_path.display());
        return Ok(());
    }

    // Final confirmation
    let answer = Confirm::new(&format!("Delete repository '{}'?", relative_path))
        .with_default(false)
        .prompt()?;

    if !answer {
        println!("Cancelled");
        return Ok(());
    }

    // Delete the repository
    fs::remove_dir_all(&target_path)?;
    println!("Deleted repository: {}", relative_path);

    Ok(())
}

fn select_repo_interactive(repo_root: &PathBuf) -> Result<PathBuf> {
    let repos = find_git_repos(repo_root)?;

    if repos.is_empty() {
        return Err(anyhow!("No repositories found"));
    }

    let relative_paths: Vec<String> = repos
        .iter()
        .map(|p| {
            p.strip_prefix(repo_root)
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    let selection = Select::new("Select repository to delete:", relative_paths)
        .with_help_message("Use arrow keys to navigate, type to filter")
        .prompt()?;

    Ok(repo_root.join(selection))
}

fn find_git_repos(root: &PathBuf) -> Result<Vec<PathBuf>> {
    const MAX_DEPTH: usize = 3;

    let mut repos = Vec::new();

    fn visit_dirs(
        dir: &PathBuf,
        repos: &mut Vec<PathBuf>,
        depth: usize,
        max_depth: usize,
    ) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        if dir.join(".git").exists() {
            repos.push(dir.clone());
            return Ok(());
        }

        if depth >= max_depth {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, repos, depth + 1, max_depth)?;
            }
        }

        Ok(())
    }

    visit_dirs(root, &mut repos, 0, MAX_DEPTH)?;
    Ok(repos)
}

fn has_uncommitted_changes(repo: &Repository) -> bool {
    if let Ok(statuses) = repo.statuses(None) {
        !statuses.is_empty()
    } else {
        false
    }
}

fn has_unpushed_commits(repo: &Repository) -> Result<bool> {
    // Get current branch
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return Ok(false), // No HEAD, no unpushed commits
    };

    if !head.is_branch() {
        return Ok(false); // Detached HEAD
    }

    let branch_name = head.shorthand().unwrap_or("");

    // Try to find upstream branch
    let upstream_name = format!("refs/remotes/origin/{}", branch_name);
    let upstream_ref = match repo.find_reference(&upstream_name) {
        Ok(r) => r,
        Err(_) => return Ok(false), // No upstream, assume no unpushed commits
    };

    let local_commit = head.peel_to_commit()?;
    let upstream_commit = upstream_ref.peel_to_commit()?;

    // Check if local is ahead of upstream
    let (ahead, _behind) = repo.graph_ahead_behind(local_commit.id(), upstream_commit.id())?;

    Ok(ahead > 0)
}
