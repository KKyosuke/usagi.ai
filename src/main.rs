use clap::{Parser, Subcommand};
use anyhow::Result;

mod command;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a repository
    Init {
        /// Repository URL
        repository_url: String,
    },
    /// Create a new workspace for a branch
    Start {
        /// New branch name
        new_branch_name: String,
        /// Origin branch name (optional)
        origin_branch_name: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { repository_url } => {
            command::init::run(repository_url)?;
        }
        Commands::Start { new_branch_name, origin_branch_name } => {
            println!("Starting new workspace: {} from {:?}", new_branch_name, origin_branch_name);
            // TODO: Implement start logic
        }
    }

    Ok(())
}
