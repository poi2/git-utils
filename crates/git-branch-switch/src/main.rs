use anyhow::Result;
use clap::Parser;
use git_utils_core::git;
use inquire::Select;

#[derive(Parser)]
#[command(name = "git-branch-switch")]
#[command(about = "Interactive branch switcher", long_about = None)]
struct Cli {
    /// Branch name or pattern to filter
    branch_pattern: Option<String>,

    /// Show recently used branches
    #[arg(short, long)]
    recent: bool,

    /// Show only merged branches
    #[arg(short, long)]
    merged: bool,

    /// Show only unmerged branches
    #[arg(long)]
    no_merged: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let repo = git::open_repo()?;
    let current_branch = git::get_current_branch(&repo)?;

    // Get branches
    let mut branches = if cli.recent {
        git::get_recent_branches(&repo)?
    } else {
        git::get_local_branches(&repo)?
    };

    // Filter by pattern if provided
    if let Some(pattern) = &cli.branch_pattern {
        branches.retain(|b| b.contains(pattern));
    }

    // Filter by merge status
    if cli.merged || cli.no_merged {
        let base_branch = git::detect_base_branch(&repo)?;
        branches.retain(|b| {
            if let Ok(is_merged) = git::is_branch_merged(&repo, b, &base_branch) {
                if cli.merged {
                    is_merged
                } else {
                    !is_merged
                }
            } else {
                false
            }
        });
    }

    // Remove current branch from list
    branches.retain(|b| b != &current_branch);

    if branches.is_empty() {
        println!("No branches found");
        return Ok(());
    }

    // Add merge status annotations
    let base_branch = git::detect_base_branch(&repo).ok();
    let branch_labels: Vec<String> = branches
        .iter()
        .map(|b| {
            if let Some(base) = &base_branch {
                if let Ok(true) = git::is_branch_merged(&repo, b, base) {
                    format!("{} [merged]", b)
                } else {
                    b.clone()
                }
            } else {
                b.clone()
            }
        })
        .collect();

    // Interactive selection
    let selection = Select::new("Select a branch:", branch_labels)
        .with_help_message("Use arrow keys to navigate, type to filter")
        .prompt()?;

    // Extract branch name (remove [merged] suffix if present)
    let selected_branch = selection.split_whitespace().next().unwrap();

    // Switch branch
    git::switch_branch(&repo, selected_branch)?;
    println!("Switched to branch '{}'", selected_branch);

    Ok(())
}
