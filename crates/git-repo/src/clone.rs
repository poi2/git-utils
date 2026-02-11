use anyhow::{anyhow, Result};
use git2::{build::RepoBuilder, FetchOptions, Repository};
use inquire::Select;
use std::io::IsTerminal;
use std::path::Path;
use std::process::Command;

use crate::utils::{convert_url_if_needed, get_repo_root, parse_repo_url};

pub fn clone_repo(url: &str, shallow: bool, bare: bool, branch: Option<&str>) -> Result<()> {
    let repo_root = get_repo_root()?;
    let url = convert_url_if_needed(url);
    let info = parse_repo_url(&url)?;

    // Build target path: <root>/<domain>/<user>/<repo>
    let target_path = repo_root
        .join(&info.domain)
        .join(&info.user)
        .join(&info.repo);

    if target_path.exists() {
        println!("Directory already exists: {}", target_path.display());
        println!();

        let options = vec![
            "Skip (do nothing)".to_string(),
            "Update (git pull)".to_string(),
            "Replace (rm -rf && clone)".to_string(),
            format!("Rename (clone as {}-2)", info.repo),
        ];
        let selection = if std::io::stdin().is_terminal() {
            Select::new("Options:", options)
                .with_help_message("Select what to do")
                .prompt()?
        } else {
            // Non-interactive mode: default to skip
            println!("Non-interactive mode: skipping");
            "Skip (do nothing)".to_string()
        };

        match selection.as_str() {
            "Skip (do nothing)" => {
                println!("Skipped");
                return Ok(());
            }
            "Update (git pull)" => {
                return update_repo(&target_path);
            }
            "Replace (rm -rf && clone)" => {
                println!("Removing existing directory...");
                std::fs::remove_dir_all(&target_path)?;
                // Continue to clone
            }
            s if s.starts_with("Rename (clone as ") => {
                return clone_with_renamed_dir(&url, shallow, bare, branch, &repo_root, &info);
            }
            _ => unreachable!(),
        }
    }

    // Create parent directories
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    println!("Cloning {} to {}...", url, target_path.display());

    // Setup clone options
    let mut builder = RepoBuilder::new();

    if shallow {
        let mut fetch_opts = FetchOptions::new();
        fetch_opts.depth(1);
        builder.fetch_options(fetch_opts);
    }

    if bare {
        builder.bare(true);
    }

    if let Some(branch_name) = branch {
        builder.branch(branch_name);
    }

    // Clone the repository
    builder.clone(&url, &target_path)?;

    println!("Successfully cloned to {}", target_path.display());

    Ok(())
}

fn update_repo(repo_path: &std::path::PathBuf) -> Result<()> {
    println!("Updating repository...");

    // Open the repository
    let repo = Repository::open(repo_path)?;

    // Check if there are uncommitted changes
    if let Ok(statuses) = repo.statuses(None) {
        if !statuses.is_empty() {
            return Err(anyhow!("Cannot update: repository has uncommitted changes"));
        }
    }

    // Run git pull
    let output = Command::new("git")
        .args(["pull"])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to pull: {}", stderr));
    }

    println!("Successfully updated");
    Ok(())
}

fn clone_with_renamed_dir(
    url: &str,
    shallow: bool,
    bare: bool,
    branch: Option<&str>,
    repo_root: &Path,
    info: &crate::utils::RepoInfo,
) -> Result<()> {
    // Find available name with suffix
    let mut suffix = 2;
    let target_path = loop {
        let path = repo_root
            .join(&info.domain)
            .join(&info.user)
            .join(format!("{}-{}", info.repo, suffix));

        if !path.exists() {
            break path;
        }
        suffix += 1;
    };

    println!("Cloning to {}...", target_path.display());

    // Create parent directories
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Setup clone options
    let mut builder = RepoBuilder::new();

    if shallow {
        let mut fetch_opts = FetchOptions::new();
        fetch_opts.depth(1);
        builder.fetch_options(fetch_opts);
    }

    if bare {
        builder.bare(true);
    }

    if let Some(branch_name) = branch {
        builder.branch(branch_name);
    }

    // Clone the repository
    builder.clone(url, &target_path)?;

    println!("Successfully cloned to {}", target_path.display());

    Ok(())
}
