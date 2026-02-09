use anyhow::{anyhow, Result};
use git2::Config;
use std::path::PathBuf;
use url::Url;

/// Get the repository root from git config
pub fn get_repo_root() -> Result<PathBuf> {
    let config = Config::open_default()?;
    let root = config.get_string("git-repo.root").map_err(|_| {
        anyhow!("git-repo.root not configured. Run 'git config --global git-repo.root <path>'")
    })?;

    let expanded = shellexpand::tilde(&root);
    Ok(PathBuf::from(expanded.as_ref()))
}

/// Check if SSH is preferred from git config
pub fn prefer_ssh() -> bool {
    if let Ok(config) = Config::open_default() {
        if let Ok(prefer) = config.get_bool("git-repo.prefer-ssh") {
            return prefer;
        }
    }
    false
}

#[derive(Debug)]
pub struct RepoInfo {
    pub domain: String,
    pub user: String,
    pub repo: String,
}

/// Parse repository URL to extract domain, user, and repo name
pub fn parse_repo_url(url_str: &str) -> Result<RepoInfo> {
    // Handle SSH URLs like git@github.com:user/repo.git
    if url_str.starts_with("git@") {
        let parts: Vec<&str> = url_str.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid SSH URL format"));
        }

        let domain = parts[0].trim_start_matches("git@");
        let path = parts[1].trim_end_matches(".git");
        let path_parts: Vec<&str> = path.split('/').collect();

        if path_parts.len() < 2 {
            return Err(anyhow!("Invalid repository path"));
        }

        Ok(RepoInfo {
            domain: domain.to_string(),
            user: path_parts[0].to_string(),
            repo: path_parts[1].to_string(),
        })
    } else {
        // Handle HTTPS URLs
        let url = Url::parse(url_str)?;
        let domain = url.host_str().ok_or_else(|| anyhow!("No host in URL"))?;

        let path = url.path().trim_start_matches('/').trim_end_matches(".git");
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() < 2 {
            return Err(anyhow!("Invalid repository path"));
        }

        Ok(RepoInfo {
            domain: domain.to_string(),
            user: parts[0].to_string(),
            repo: parts[1].to_string(),
        })
    }
}

/// Convert HTTPS URL to SSH if needed
pub fn convert_url_if_needed(url: &str) -> String {
    if !prefer_ssh() || url.starts_with("git@") {
        return url.to_string();
    }

    // Convert HTTPS to SSH
    if let Ok(parsed) = Url::parse(url) {
        if let Some(host) = parsed.host_str() {
            let path = parsed.path().trim_start_matches('/');
            return format!("git@{}:{}", host, path);
        }
    }

    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_url() {
        let url = "git@github.com:poi2/git-utils.git";
        let info = parse_repo_url(url).unwrap();
        assert_eq!(info.domain, "github.com");
        assert_eq!(info.user, "poi2");
        assert_eq!(info.repo, "git-utils");
    }

    #[test]
    fn test_parse_https_url() {
        let url = "https://github.com/poi2/git-utils.git";
        let info = parse_repo_url(url).unwrap();
        assert_eq!(info.domain, "github.com");
        assert_eq!(info.user, "poi2");
        assert_eq!(info.repo, "git-utils");
    }
}
