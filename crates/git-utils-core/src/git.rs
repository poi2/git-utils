use crate::{Error, Result};
use git2::{BranchType, Repository};
use std::path::Path;

/// Opens the git repository in the current directory or parent directories
pub fn open_repo() -> Result<Repository> {
    Repository::discover(".").map_err(|_| Error::NotGitRepository)
}

/// Get all local branch names
pub fn get_local_branches(repo: &Repository) -> Result<Vec<String>> {
    let mut branches = Vec::new();
    for branch in repo.branches(Some(BranchType::Local))? {
        let (branch, _) = branch?;
        if let Some(name) = branch.name()? {
            branches.push(name.to_string());
        }
    }
    Ok(branches)
}

/// Get the current branch name
pub fn get_current_branch(repo: &Repository) -> Result<String> {
    let head = repo.head()?;
    if let Some(name) = head.shorthand() {
        Ok(name.to_string())
    } else {
        Err(Error::Other("HEAD is detached".to_string()))
    }
}

/// Check if a branch is merged into the base branch
pub fn is_branch_merged(repo: &Repository, branch_name: &str, base_branch: &str) -> Result<bool> {
    let base_ref = repo.find_branch(base_branch, BranchType::Local)?;
    let base_commit = base_ref.get().peel_to_commit()?;

    let branch_ref = repo.find_branch(branch_name, BranchType::Local)?;
    let branch_commit = branch_ref.get().peel_to_commit()?;

    Ok(repo.graph_descendant_of(base_commit.id(), branch_commit.id())?)
}

/// Detect base branch (main, master, or develop)
pub fn detect_base_branch(repo: &Repository) -> Result<String> {
    // First, check git config
    if let Ok(config) = repo.config() {
        if let Ok(base) = config.get_string("git-branch-delete.base") {
            return Ok(base);
        }
    }

    // Try common base branch names
    for candidate in &["main", "master", "develop"] {
        if repo.find_branch(candidate, BranchType::Local).is_ok() {
            return Ok(candidate.to_string());
        }
    }

    Err(Error::BaseBranchNotFound)
}

/// Switch to a branch
pub fn switch_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    let obj = repo.revparse_single(&format!("refs/heads/{}", branch_name))?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&format!("refs/heads/{}", branch_name))?;
    Ok(())
}

/// Delete a branch
pub fn delete_branch(repo: &Repository, branch_name: &str, force: bool) -> Result<()> {
    let mut branch = repo.find_branch(branch_name, BranchType::Local)?;

    // Check if merged before deleting (unless force is true)
    if !force {
        let base_branch = detect_base_branch(repo)?;
        if !is_branch_merged(repo, branch_name, &base_branch)? {
            return Err(Error::Other(format!(
                "Branch '{}' is not merged into '{}'. Use --force to delete anyway.",
                branch_name, base_branch
            )));
        }
    }

    branch.delete()?;
    Ok(())
}

/// Get recent branches from reflog
pub fn get_recent_branches(repo: &Repository) -> Result<Vec<String>> {
    let mut branches = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Parse reflog to find branch switches
    let reflog = repo.reflog("HEAD")?;
    for entry in reflog.iter() {
        if let Some(msg) = entry.message() {
            if msg.starts_with("checkout: moving from") {
                // Extract branch name from message like "checkout: moving from main to feature"
                if let Some(to_branch) = msg.split_whitespace().last() {
                    if !seen.contains(to_branch) {
                        seen.insert(to_branch.to_string());
                        branches.push(to_branch.to_string());
                    }
                }
            }
        }
    }

    Ok(branches)
}

/// Get repository root path
pub fn get_repo_root(repo: &Repository) -> Result<&Path> {
    repo.workdir()
        .ok_or_else(|| Error::Other("Bare repository not supported".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_repo() {
        // This test will only work in a git repository
        // Skip if not in a git repo
        if let Ok(repo) = open_repo() {
            assert!(!repo.is_bare());
        }
    }
}
