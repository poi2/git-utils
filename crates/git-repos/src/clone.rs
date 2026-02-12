use anyhow::Result;
use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks};

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

    // Create parent directories
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    println!("Cloning {} to {}...", url, target_path.display());

    // Setup SSH authentication callbacks
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, allowed_types| {
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        } else if allowed_types.contains(git2::CredentialType::USERNAME) {
            Cred::username(username_from_url.unwrap_or("git"))
        } else {
            Err(git2::Error::from_str(
                "No supported authentication methods available",
            ))
        }
    });

    // Setup fetch options with callbacks
    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    if shallow {
        fetch_opts.depth(1);
    }

    // Setup clone options
    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_opts);

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
