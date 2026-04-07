use anyhow::{Result, Context, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use directories::ProjectDirs;
use console::{Term, Key, style};
use inquire::Text;
use crate::application::layout::{self, AlternateScreenGuard};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectState {
    pub initialized: bool,
    pub worktrees: Vec<String>,
    pub current_worktree: Option<String>,
    #[serde(default)]
    pub history: Vec<String>,
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
    let mut repos = get_repositories()?;
    let mut selected_index = 0;
    let term = Term::stdout();
    let mut _guard = AlternateScreenGuard::new(term.clone())?;

    loop {
        term.move_cursor_to(0, 0)?;
        term.clear_screen()?;
        layout::show_rabbit(&term);
        
        let menu_items = vec![
            layout::MenuItem { icon: "".to_string(), label: "Open".to_string(), key: "o".to_string() },
            layout::MenuItem { icon: "".to_string(), label: "New".to_string(), key: "e".to_string() },
            layout::MenuItem { icon: "".to_string(), label: "Config".to_string(), key: "c".to_string() },
            layout::MenuItem { icon: "".to_string(), label: "Quit".to_string(), key: "q".to_string() },
        ];

        layout::render_side_menu(
            &term,
            &menu_items,
            selected_index,
        );

        layout::render_footer(&term);

        let key = match term.read_key() {
            Ok(k) => k,
            Err(e) => {
                if e.to_string().contains("read interrupted") {
                    drop(_guard);
                    println!("Quit.");
                    return Ok(None);
                }
                return Err(anyhow::Error::from(e)).context("Failed to read key");
            }
        };
        
        match key {
            Key::ArrowUp => {
                if selected_index > 0 { selected_index -= 1; }
                else { selected_index = menu_items.len() - 1; }
            }
            Key::ArrowDown => {
                if selected_index < menu_items.len() - 1 { selected_index += 1; }
                else { selected_index = 0; }
            }
            Key::Enter => {
                if selected_index == 0 { // Open
                    if let Some(selected_path) = show_project_list_modal(&term, &repos)? {
                        _guard.dismiss();
                        drop(_guard);
                        return Ok(Some((selected_path, None)));
                    }
                } else if selected_index == 3 { // Quit
                    drop(_guard);
                    println!("Quit.");
                    return Ok(None);
                }
            }
            Key::Char('q') | Key::Escape | Key::CtrlC => {
                // ここでは明示的な終了なのでメッセージを出す（dismissしない）
                drop(_guard);
                println!("Quit.");
                return Ok(None);
            }
            _ => {}
        }
    }
}

pub fn show_project_list_modal(term: &Term, repos: &[PathBuf]) -> Result<Option<PathBuf>> {
    let mut selected_index = 0;
    if repos.is_empty() {
        return Ok(None);
    }

    loop {
        term.clear_screen()?;
        let (height, width) = term.size();
        let width = width as usize;
        
        let title = "--- Select Project ---";
        let title_len = title.chars().count();
        let left_padding = if width > title_len { (width - title_len) / 2 } else { 0 };
        term.write_line(&format!("{}{}", " ".repeat(left_padding), style(title).bold().yellow()))?;
        term.write_line("")?;

        for (i, repo) in repos.iter().enumerate() {
            let label = repo.display().to_string();
            let label_len = label.chars().count() + 2;
            let left_padding = if width > label_len { (width - label_len) / 2 } else { 0 };
            
            if i == selected_index {
                term.write_line(&format!("{}> {}", " ".repeat(left_padding), style(label).cyan().bold()))?;
            } else {
                term.write_line(&format!("{}  {}", " ".repeat(left_padding), label))?;
            }
        }

        let key = match term.read_key() {
            Ok(k) => k,
            Err(e) => {
                if e.to_string().contains("read interrupted") {
                    return Ok(None);
                }
                return Err(anyhow::Error::from(e)).context("Failed to read key");
            }
        };
        match key {
            Key::ArrowUp => {
                if selected_index > 0 { selected_index -= 1; }
                else { selected_index = repos.len() - 1; }
            }
            Key::ArrowDown => {
                if selected_index < repos.len() - 1 { selected_index += 1; }
                else { selected_index = 0; }
            }
            Key::Enter => {
                return Ok(Some(repos[selected_index].clone()));
            }
            Key::Escape | Key::Char('q') | Key::CtrlC => {
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

        let key = match term.read_key() {
            Ok(k) => k,
            Err(e) => {
                if e.to_string().contains("read interrupted") {
                    return Ok(false);
                }
                return Err(anyhow::Error::from(e)).context("Failed to read key");
            }
        };
        term.clear_last_lines(3).context("Failed to clear lines")?;

        match key {
            Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => {
                delete_selected = !delete_selected;
            }
            Key::Enter => {
                return Ok(delete_selected);
            }
            Key::Escape | Key::Char('q') | Key::CtrlC => {
                return Ok(false);
            }
            _ => {}
        }
    }
}
