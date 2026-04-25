use anyhow::Result;
use crate::presentation::tui::home::{self, HomeAction};
use crate::presentation::tui::project;
use crate::infrastructure::global_registry;
use crate::presentation::cli;

pub async fn run() -> Result<()> {
    loop {
        match home::run()? {
            Some(HomeAction::Open) => {
                let repos = global_registry::get_repositories()?;
                if let Some(project_path) = project::run(&repos)? {
                    // Transition to Hop TUI
                    cli::hop::run(project_path, None).await?;
                }
                // If they press Escape in project_select, it simply returns None and the loop goes back to Home.
            }
            Some(HomeAction::New) => {
                let term = console::Term::stdout();
                term.clear_screen()?;
                term.write_line("New feature is not implemented yet. Please use 'usagi init'.")?;
                term.write_line("Press any key to continue...")?;
                term.read_key()?;
            }
            Some(HomeAction::Config) => {
                let term = console::Term::stdout();
                term.clear_screen()?;
                term.write_line("Config feature is not implemented yet.")?;
                term.write_line("Press any key to continue...")?;
                term.read_key()?;
            }
            Some(HomeAction::Quit) | None => {
                break;
            }
        }
    }
    
    Ok(())
}
