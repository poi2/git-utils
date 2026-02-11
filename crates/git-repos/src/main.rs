use anyhow::Result;
use clap::{Parser, Subcommand};

mod clone;
mod ls;
mod utils;

use clone::clone_repo;
use ls::list_repos;

#[derive(Parser)]
#[command(name = "git-repos")]
#[command(about = "Manage git repositories", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clone a repository to the managed location
    Clone {
        /// Repository URL
        url: String,

        /// Shallow clone with --depth=1
        #[arg(long)]
        shallow: bool,

        /// Clone as bare repository
        #[arg(long)]
        bare: bool,

        /// Checkout specific branch
        #[arg(short, long)]
        branch: Option<String>,
    },

    /// List all managed repositories
    Ls {
        /// Show detailed information
        #[arg(short, long)]
        long: bool,

        /// Show absolute paths
        #[arg(short, long)]
        absolute: bool,

        /// Show only dirty repositories
        #[arg(long)]
        dirty: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Clone {
            url,
            shallow,
            bare,
            branch,
        } => {
            clone_repo(&url, shallow, bare, branch.as_deref())?;
        }
        Commands::Ls {
            long,
            absolute,
            dirty,
            json,
        } => {
            list_repos(long, absolute, dirty, json)?;
        }
    }

    Ok(())
}
