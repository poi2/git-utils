use anyhow::{anyhow, Context, Result};
use clap::Parser;
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Parser)]
#[command(name = "git-pr-merged")]
#[command(about = "List merged pull requests in a revision range", long_about = None)]
struct Cli {
    /// Revision range (e.g., v1.0.0..v1.1.0, HEAD~10..HEAD)
    /// If not specified, uses latest tag..HEAD
    revision_range: Option<String>,

    /// Number of commits to check (alternative to revision range)
    #[arg(short, long, conflicts_with = "revision_range")]
    count: Option<usize>,

    /// Open PR list in web browser
    #[arg(short, long)]
    web: bool,

    /// Output format: text (default), json, markdown, plain
    #[arg(long, default_value = "text")]
    format: OutputFormat,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum OutputFormat {
    /// OSC 8 terminal links (default)
    Text,
    /// JSON format
    Json,
    /// Markdown format
    Markdown,
    /// Plain text without OSC 8
    Plain,
}

#[derive(Debug, Serialize, Deserialize)]
struct PullRequest {
    number: u32,
    title: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    merged_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
}

#[derive(Debug, Serialize)]
struct Output {
    range: String,
    platform: String,
    pulls: Vec<PullRequest>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let repo = Repository::discover(".")
        .context("Not a git repository. Run this command from within a git repository.")?;

    // Determine revision range
    let revision_range = if let Some(range) = cli.revision_range {
        range
    } else if let Some(count) = cli.count {
        format!("HEAD~{}..HEAD", count)
    } else {
        // Use latest tag..HEAD
        get_latest_tag(&repo)
            .map(|tag| format!("{}..HEAD", tag))
            .unwrap_or_else(|| "HEAD~10..HEAD".to_string())
    };

    // Check if gh command is available
    if !is_gh_available() {
        return Err(anyhow!(
            "gh command not found. Please install GitHub CLI: https://cli.github.com/"
        ));
    }

    // Get repository info (owner/repo)
    let repo_info = get_repo_info(&repo)?;

    // Extract PR numbers from git log
    let pr_numbers = extract_pr_numbers(&repo, &revision_range)?;

    if pr_numbers.is_empty() {
        println!("No merged pull requests found in range: {}", revision_range);
        return Ok(());
    }

    if cli.web {
        open_in_browser(&repo_info, &pr_numbers)?;
        return Ok(());
    }

    // Fetch PR details using gh command
    let pulls = fetch_pr_details(&repo_info, &pr_numbers)?;

    // Output results
    let output = Output {
        range: revision_range,
        platform: "github".to_string(),
        pulls,
    };

    match cli.format {
        OutputFormat::Text => print_text(&output, true),
        OutputFormat::Plain => print_text(&output, false),
        OutputFormat::Json => print_json(&output)?,
        OutputFormat::Markdown => print_markdown(&output),
    }

    Ok(())
}

fn get_latest_tag(repo: &Repository) -> Option<String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .current_dir(repo.path().parent()?)
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

fn is_gh_available() -> bool {
    Command::new("gh")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn get_repo_info(repo: &Repository) -> Result<String> {
    let remote = repo
        .find_remote("origin")
        .context("No 'origin' remote found")?;

    let url = remote.url().context("Invalid remote URL")?;

    // Parse GitHub URL (either SSH or HTTPS)
    // SSH: git@github.com:owner/repo.git
    // HTTPS: https://github.com/owner/repo.git
    let repo_path = if url.starts_with("git@github.com:") {
        url.trim_start_matches("git@github.com:")
            .trim_end_matches(".git")
    } else if url.contains("github.com/") {
        url.split("github.com/")
            .nth(1)
            .context("Invalid GitHub URL")?
            .trim_end_matches(".git")
    } else {
        return Err(anyhow!("Not a GitHub repository"));
    };

    Ok(repo_path.to_string())
}

fn extract_pr_numbers(repo: &Repository, range: &str) -> Result<Vec<u32>> {
    let output = Command::new("git")
        .args(["log", "--format=%s", range])
        .current_dir(repo.path().parent().context("Invalid repo path")?)
        .output()
        .context("Failed to run git log")?;

    if !output.status.success() {
        return Err(anyhow!("Invalid revision range: {}", range));
    }

    let log = String::from_utf8(output.stdout)?;
    let mut pr_numbers = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let pr_regex = regex::Regex::new(r"#(\d+)").unwrap();

    for line in log.lines() {
        // Look for patterns like "#123" or "(#123)" in commit messages
        for cap in pr_regex.captures_iter(line) {
            if let Some(num_str) = cap.get(1) {
                if let Ok(num) = num_str.as_str().parse::<u32>() {
                    if !seen.contains(&num) {
                        seen.insert(num);
                        pr_numbers.push(num);
                    }
                }
            }
        }
    }

    Ok(pr_numbers)
}

fn fetch_pr_details(repo_info: &str, pr_numbers: &[u32]) -> Result<Vec<PullRequest>> {
    let mut pulls = Vec::new();

    for &number in pr_numbers {
        let output = Command::new("gh")
            .args([
                "pr",
                "view",
                &number.to_string(),
                "--repo",
                repo_info,
                "--json",
                "number,title,url,mergedAt,author",
            ])
            .output()
            .context("Failed to run gh command")?;

        if output.status.success() {
            let pr: serde_json::Value = serde_json::from_slice(&output.stdout)?;

            pulls.push(PullRequest {
                number,
                title: pr["title"].as_str().unwrap_or("").to_string(),
                url: pr["url"].as_str().unwrap_or("").to_string(),
                merged_at: pr["mergedAt"].as_str().map(|s| s.to_string()),
                author: pr["author"]["login"].as_str().map(|s| s.to_string()),
            });
        }
    }

    Ok(pulls)
}

fn print_text(output: &Output, with_links: bool) {
    for pr in &output.pulls {
        if with_links {
            // OSC 8 format: \x1b]8;;URL\x1b\\TEXT\x1b]8;;\x1b\\
            print!("\x1b]8;;{}\x1b\\", pr.url);
            print!("#{}", pr.number);
            print!("\x1b]8;;\x1b\\");
            println!();
        } else {
            println!("#{}", pr.number);
        }
    }
}

fn print_json(output: &Output) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(output)?);
    Ok(())
}

fn print_markdown(output: &Output) {
    println!("## Merged PRs ({})", output.range);
    println!();
    for pr in &output.pulls {
        print!("- [#{}]({}) {}", pr.number, pr.url, pr.title);
        if let Some(author) = &pr.author {
            print!(" (@{})", author);
        }
        println!();
    }
}

fn open_in_browser(repo_info: &str, pr_numbers: &[u32]) -> Result<()> {
    let base_url = format!("https://github.com/{}/pulls", repo_info);
    let query = pr_numbers
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join("+");
    let url = format!("{}?q=is:pr+is:merged+{}", base_url, query);

    // Use gh to open browser
    let output = Command::new("gh")
        .args(["pr", "list", "--web", "--repo", repo_info])
        .output()
        .context("Failed to open browser")?;

    if !output.status.success() {
        return Err(anyhow!("Failed to open browser"));
    }

    println!("Opened in browser: {}", url);
    Ok(())
}
