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

                let git_url = match inquire::Text::new("git url:").prompt() {
                    Ok(url) if url.is_empty() => {
                        term.write_line("Git URL is required.")?;
                        term.write_line("Press any key to continue...")?;
                        term.read_key()?;
                        continue;
                    }
                    Ok(url) => url,
                    Err(_) => continue,
                };

                let repo_name = git_url.split('/').last().unwrap_or("repository").trim_end_matches(".git");
                let default_dir = format!("./{}", repo_name);

                let directory_input = match inquire::Text::new("directory:")
                    .with_default(&default_dir)
                    .prompt() {
                    Ok(dir) => dir,
                    Err(_) => continue,
                };

                let branch_input = match inquire::Text::new("branch:")
                    .with_default("default")
                    .prompt() {
                    Ok(branch) => branch,
                    Err(_) => continue,
                };

                let directory = if directory_input.is_empty() { None } else { Some(std::path::PathBuf::from(directory_input)) };
                let branch = if branch_input == "default" || branch_input.is_empty() { None } else { Some(branch_input) };

                let current_dir = std::env::current_dir()?;

                term.clear_screen()?;
                match crate::presentation::cli::init::run(&git_url, directory, branch) {
                    Ok(_) => {}
                    Err(e) => {
                        term.write_line(&format!("Error: {}", e))?;
                    }
                }

                let _ = std::env::set_current_dir(current_dir);

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
