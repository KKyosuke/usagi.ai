use clap::{Parser, Subcommand};
use anyhow::Result;

mod command;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a repository
    Init {
        /// Repository URL
        repository_url: String,
        /// Directory name (optional)
        #[arg(short, long)]
        directory: Option<String>,
        /// Branch name (optional)
        #[arg(short, long)]
        branch: Option<String>,
    },
    /// Create a new workspace for a branch
    Start {
        /// New branch name
        new_branch_name: String,
        /// Origin branch name (optional)
        origin_branch_name: Option<String>,
    },
    /// Open a workspace (interactive menu)
    Open,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { repository_url, directory, branch }) => {
            let directory = directory.as_ref().map(std::path::PathBuf::from);
            command::init::run(repository_url, directory, branch.clone())?;
        }
        Some(Commands::Start { new_branch_name, origin_branch_name }) => {
            println!("Starting new workspace: {} from {:?}", new_branch_name, origin_branch_name);
            // TODO: Implement start logic
        }
        Some(Commands::Open) | None => {
            command::open::run()?;
        }
    }

    Ok(())
}
