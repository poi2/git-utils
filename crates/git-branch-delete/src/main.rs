use anyhow::Result;
use clap::Parser;
use git_utils_core::git;
use inquire::Confirm;

#[derive(Parser)]
#[command(name = "git-branch-delete")]
#[command(about = "Delete git branches interactively", long_about = None)]
struct Cli {
    /// Delete all branches except base and current
    #[arg(short, long, conflicts_with_all = ["merged", "select"])]
    all: bool,

    /// Delete only merged branches (default)
    #[arg(short, long)]
    merged: bool,

    /// Select branches one by one
    #[arg(short, long, conflicts_with = "all")]
    select: bool,

    /// Force delete (use -D instead of -d)
    #[arg(short, long, conflicts_with = "merged")]
    force: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let repo = git::open_repo()?;
    let current_branch = git::get_current_branch(&repo)?;
    let base_branch = git::detect_base_branch(&repo)?;

    println!("Base branch: {}", base_branch);
    println!("Current branch: {}", current_branch);

    // Get all local branches
    let mut branches = git::get_local_branches(&repo)?;

    // Remove current and base branches
    branches.retain(|b| b != &current_branch && b != &base_branch);

    // Filter by merge status (default is merged unless --force)
    if !cli.all && !cli.force {
        branches.retain(|b| git::is_branch_merged(&repo, b, &base_branch).unwrap_or_default());
    }

    if branches.is_empty() {
        println!("No branches to delete");
        return Ok(());
    }

    // Select mode
    let branches_to_delete = if cli.select {
        let mut selected = Vec::new();
        for branch in &branches {
            let is_merged = git::is_branch_merged(&repo, branch, &base_branch).unwrap_or(false);
            let label = if is_merged {
                format!("{} [merged]", branch)
            } else {
                branch.clone()
            };

            let answer = Confirm::new(&format!("Delete branch '{}'?", label))
                .with_default(false)
                .prompt()?;

            if answer {
                selected.push(branch.clone());
            }
        }
        selected
    } else {
        // Show branches to be deleted
        println!("\nBranches to be deleted:");
        for branch in &branches {
            let is_merged = git::is_branch_merged(&repo, branch, &base_branch).unwrap_or(false);
            if is_merged {
                println!("  {} [merged]", branch);
            } else {
                println!("  {}", branch);
            }
        }

        let answer = Confirm::new(&format!("\nDelete {} branches?", branches.len()))
            .with_default(false)
            .prompt()?;

        if answer {
            branches
        } else {
            Vec::new()
        }
    };

    // Delete branches
    if branches_to_delete.is_empty() {
        println!("No branches deleted");
        return Ok(());
    }

    let mut deleted_count = 0;
    let mut skipped_count = 0;

    for branch in &branches_to_delete {
        match git::delete_branch(&repo, branch, cli.force) {
            Ok(_) => {
                println!("Deleted local branch '{}'", branch);
                deleted_count += 1;
            }
            Err(e) => {
                eprintln!("Skipped local branch '{}': {}", branch, e);
                skipped_count += 1;
            }
        }
    }

    if skipped_count > 0 {
        println!(
            "\nDeleted {} local branches ({} skipped)",
            deleted_count, skipped_count
        );
    } else {
        println!("\nDeleted {} local branches", deleted_count);
    }

    Ok(())
}
