use clap::{Parser, Subcommand};
use anyhow::Result;
use usagi::presentation::commands::Command;
use usagi::presentation;

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
    /// Check system dependencies
    Doctor,
    /// AWS related commands
    Aws {
        #[command(subcommand)]
        command: AwsCommands,
    },
}

#[derive(Subcommand)]
pub enum AwsCommands {
    /// Login to AWS SSO
    Login {
        /// Optional profile name
        profile: Option<String>,
    },
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
        Some(Commands::Doctor) => {
            let doctor = presentation::commands::doctor::DoctorCommand;
            let current_dir = std::env::current_dir()?;
            let term = console::Term::stdout();
            println!("{}", doctor.run(vec![], &current_dir, "", &term)?);
        }
        Some(Commands::Aws { command }) => {
            match command {
                AwsCommands::Login { profile } => {
                    presentation::cli::aws::run_login(profile.clone())?;
                }
            }
        }
        Some(Commands::Open) | None => {
            presentation::cli::open::run()?;
        }
    }

    Ok(())
}
