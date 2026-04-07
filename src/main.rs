use clap::{Parser, Subcommand};
use anyhow::Result;

mod domain;
mod infrastructure;
mod usecase;
mod presentation;

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
    /// Open a workspace (interactive side menu)
    Open,
    /// Hop into usagi terminal
    Hop,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { repository_url, directory, branch }) => {
            let directory = directory.as_ref().map(std::path::PathBuf::from);
            presentation::cli::init::run(repository_url, directory, branch.clone())?;
        }
        Some(Commands::Hop) => {
            let current_dir = std::env::current_dir()?;
            presentation::cli::hop::run(current_dir, None)?;
        }
        Some(Commands::Open) | None => {
            presentation::cli::open::run()?;
        }
    }

    Ok(())
}
