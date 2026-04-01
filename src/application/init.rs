use anyhow::{Result, Context, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use directories::ProjectDirs;
use console::{Term, Key, style};
use inquire::Text;
use crate::application::layout;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectState {
    pub initialized: bool,
    pub worktrees: Vec<String>,
    pub current_worktree: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Repositories {
    pub repositories: Vec<PathBuf>,
}

pub fn get_repositories() -> Result<Vec<PathBuf>> {
    let proj_dirs = ProjectDirs::from("", "", "usagi")
        .ok_or_else(|| anyhow!("Could not determine home directory"))?;
    let data_dir = proj_dirs.data_dir();
    let repo_json_path = data_dir.join("repositories.json");

    if repo_json_path.exists() {
        let content = fs::read_to_string(&repo_json_path).context("Failed to read repositories.json")?;
        let repos: Repositories = serde_json::from_str(&content).context("Failed to parse repositories.json")?;
        Ok(repos.repositories)
    } else {
        Ok(vec![])
    }
}

pub fn save_repositories(repos: &[PathBuf]) -> Result<()> {
    let proj_dirs = ProjectDirs::from("", "", "usagi")
        .ok_or_else(|| anyhow!("Could not determine home directory"))?;
    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir).context("Failed to create data directory")?;

    let repo_json_path = data_dir.join("repositories.json");
    let repos_struct = Repositories {
        repositories: repos.to_vec(),
    };
    let content = serde_json::to_string_pretty(&repos_struct).context("Failed to serialize repositories")?;
    fs::write(repo_json_path, content).context("Failed to write repositories.json")?;

    Ok(())
}

pub fn get_project_state(project_path: &Path) -> Result<ProjectState> {
    let state_path = project_path.join(".usagi/state.json");
    if !state_path.exists() {
        return Err(anyhow!("Project state is missing in {}. Please ensure it's a valid usagi project.", project_path.display()));
    }

    let state_json = fs::read_to_string(state_path).context("Failed to read project state")?;
    let state: ProjectState = serde_json::from_str(&state_json).context("Failed to parse project state")?;
    Ok(state)
}

pub fn save_project_state(project_path: &Path, state: &ProjectState) -> Result<()> {
    let state_path = project_path.join(".usagi/state.json");
    let content = serde_json::to_string_pretty(state).context("Failed to serialize project state")?;
    fs::write(state_path, content).context("Failed to write project state")?;
    Ok(())
}

pub fn run_terminal_ui() -> Result<Option<(PathBuf, Option<String>)>> {
    layout::show_rabbit();

    let mut repos = get_repositories()?;
    let mut selected_index = 0;
    let term = Term::stdout();

    loop {
        // Side menu items (Projects list)
        let mut project_items: Vec<String> = repos.iter().map(|path| {
            let config_path = path.join("usagi.config");
            let status = if !path.exists() {
                style("(Missing)").red()
            } else if config_path.exists() {
                style("(Active)").green()
            } else {
                style("(No config)").red()
            };
            format!("{} {}", path.display(), status)
        }).collect();
        project_items.push(format!("+ {}", style("New project").yellow().bold()));

        layout::render_side_menu(
            &project_items,
            selected_index,
        );

        let key = term.read_key().context("Failed to read key")?;
        
        // Clear lines: side menu + rabbit + header
        let lines_to_clear = project_items.len() + 3;
        term.clear_last_lines(lines_to_clear).context("Failed to clear lines")?;

        match key {
            Key::ArrowUp => {
                if selected_index > 0 { selected_index -= 1; }
                else { selected_index = project_items.len() - 1; }
            }
            Key::ArrowDown => {
                if selected_index < project_items.len() - 1 { selected_index += 1; }
                else { selected_index = 0; }
            }
            Key::Enter => {
                if selected_index < repos.len() {
                    let selected_path = &repos[selected_index];
                    let config_path = selected_path.join("usagi.config");
                    
                    if !selected_path.exists() || !config_path.exists() {
                        if show_delete_modal(selected_path)? {
                            repos.remove(selected_index);
                            save_repositories(&repos)?;
                            if selected_index >= repos.len() && !repos.is_empty() {
                                selected_index = repos.len() - 1;
                            }
                        }
                        continue;
                    }

                    return Ok(Some((selected_path.to_path_buf(), None)));
                } else {
                    // New project
                    let repo_url = Text::new("Repository URL:").prompt()?;
                    let directory = Text::new("Directory (optional):").prompt()?;
                    let branch = Text::new("Branch (optional, leave empty for default):").prompt()?;

                    let directory = if directory.is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(directory))
                    };
                    let branch = if branch.is_empty() {
                        None
                    } else {
                        Some(branch)
                    };

                    return crate::command::init::run(&repo_url, directory, branch).map(|_| None);
                }
            }
            Key::Char('q') | Key::Escape => {
                println!("Quit.");
                return Ok(None);
            }
            _ => {}
        }
    }
}

fn show_delete_modal(path: &Path) -> Result<bool> {
    let term = Term::stdout();
    let mut delete_selected = true;

    loop {
        println!("Project config is missing or directory not found: {}", style(path.display()).yellow());
        println!("Do you want to delete this project from list or keep it?");
        
        let delete_btn = if delete_selected {
            style("[ Delete ]").cyan().bold()
        } else {
            style("[ Delete ]").white()
        };

        let keep_btn = if !delete_selected {
            style("[ Keep ]").cyan().bold()
        } else {
            style("[ Keep ]").white()
        };

        println!("  {}     {}", delete_btn, keep_btn);

        let key = term.read_key().context("Failed to read key")?;
        term.clear_last_lines(3).context("Failed to clear lines")?;

        match key {
            Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => {
                delete_selected = !delete_selected;
            }
            Key::Enter => {
                return Ok(delete_selected);
            }
            Key::Escape | Key::Char('q') => {
                return Ok(false);
            }
            _ => {}
        }
    }
}
