use anyhow::Result;
use git2::Repository;
use serde::Serialize;
use std::path::PathBuf;

use crate::utils::get_repo_root;

#[derive(Serialize)]
struct RepoEntry {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    absolute_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
}

pub fn list_repos(long: bool, absolute: bool, dirty: bool, json: bool) -> Result<()> {
    let repo_root = get_repo_root()?;

    if !repo_root.exists() {
        println!("Repository root does not exist: {}", repo_root.display());
        return Ok(());
    }

    let repos = find_git_repos(&repo_root)?;

    if repos.is_empty() {
        println!("No repositories found");
        return Ok(());
    }

    let mut entries: Vec<RepoEntry> = Vec::new();

    for repo_path in repos {
        let relative_path = repo_path
            .strip_prefix(&repo_root)
            .unwrap()
            .to_string_lossy()
            .to_string();

        // Check if dirty filter is enabled
        if dirty {
            if let Ok(repo) = Repository::open(&repo_path) {
                if is_repo_clean(&repo) {
                    continue;
                }
            }
        }

        let mut entry = RepoEntry {
            path: relative_path.clone(),
            absolute_path: None,
            branch: None,
            status: None,
        };

        if absolute {
            entry.absolute_path = Some(repo_path.to_string_lossy().to_string());
        }

        if long || json {
            if let Ok(repo) = Repository::open(&repo_path) {
                entry.branch = get_current_branch(&repo);
                entry.status = Some(get_repo_status(&repo));
            }
        }

        entries.push(entry);
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        for entry in entries {
            if long {
                println!(
                    "{:<50} {:<20} {}",
                    if absolute {
                        entry.absolute_path.as_ref().unwrap()
                    } else {
                        &entry.path
                    },
                    entry.branch.as_deref().unwrap_or(""),
                    entry.status.as_deref().unwrap_or("")
                );
            } else {
                println!(
                    "{}",
                    if absolute {
                        entry.absolute_path.as_ref().unwrap()
                    } else {
                        &entry.path
                    }
                );
            }
        }
    }

    Ok(())
}

fn find_git_repos(root: &PathBuf) -> Result<Vec<PathBuf>> {
    // Maximum depth for repository discovery
    // For <root>/<domain>/<user>/<repo> layout, we need depth of 3
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

        // Check if this is a git repository
        if dir.join(".git").exists() {
            repos.push(dir.clone());
            return Ok(()); // Don't recurse into subdirectories of a git repo
        }

        // Stop recursion if we've reached max depth
        if depth >= max_depth {
            return Ok(());
        }

        // Recurse into subdirectories
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

fn get_current_branch(repo: &Repository) -> Option<String> {
    repo.head().ok()?.shorthand().map(|s| s.to_string())
}

fn get_repo_status(repo: &Repository) -> String {
    if is_repo_clean(repo) {
        "[clean]".to_string()
    } else {
        "[dirty]".to_string()
    }
}

fn is_repo_clean(repo: &Repository) -> bool {
    if let Ok(statuses) = repo.statuses(None) {
        statuses.is_empty()
    } else {
        true
    }
}
