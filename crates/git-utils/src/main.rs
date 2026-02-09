use anyhow::Result;
use clap::{Parser, Subcommand};

mod setup;

use setup::Setup;

#[derive(Parser)]
#[command(name = "git-utils")]
#[command(about = "Git utilities setup and management", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Setup git-utils environment
    Setup(Setup),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup(setup) => setup.execute()?,
    }

    Ok(())
}
