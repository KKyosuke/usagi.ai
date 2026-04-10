use anyhow::{Result, anyhow, Context};
use std::path::PathBuf;
use console::Term;
use crate::domain::project::ProjectState;
use crate::infrastructure::project_state::{get_project_state, save_project_state};
use crate::presentation::commands::{self, Command};
use crate::presentation::tui::mode::AppMode;

pub struct HopApp {
    pub project_path: PathBuf,
    pub state: ProjectState,
    pub worktrees: Vec<String>,
    pub term: Term,
    pub selected_index: usize,
    pub current_input: String,
    pub cursor_pos: usize,
    pub is_command_mode: bool,
    pub history_index: Option<usize>,
    pub commands: Vec<Box<dyn Command>>,
    pub command_history: Vec<String>,
    pub is_terminal_view: bool,
}

impl HopApp {
    pub fn new(project_path: PathBuf, initial_worktree: Option<String>) -> Result<Self> {
        let state = get_project_state(&project_path)
            .map_err(|_| anyhow!("Error: Not an initialized directory. Please run `usagi init` first."))?;

        std::env::set_current_dir(&project_path).context(format!("Failed to change directory to {}", project_path.display()))?;

        let worktrees: Vec<String> = state.worktrees.iter().map(|w| w.branch.clone()).collect();
        let term = Term::stdout();
        
        let mut selected_index = 0;
        // 初期選択のワークツリーがあれば設定
        if let Some(initial_wt) = initial_worktree {
            if let Some(idx) = worktrees.iter().position(|wt| wt == &initial_wt) {
                selected_index = idx;
            }
        } else if let Some(current_wt) = &state.current_worktree {
            if let Some(idx) = worktrees.iter().position(|wt| wt == current_wt) {
                selected_index = idx;
            }
        }

        let command_history: Vec<String> = state.history.iter().filter(|s| !s.trim().is_empty()).cloned().collect();
        let commands = commands::get_commands();

        Ok(Self {
            project_path,
            state,
            worktrees,
            term,
            selected_index,
            current_input: String::new(),
            cursor_pos: 0,
            is_command_mode: false,
            history_index: None,
            commands,
            command_history,
            is_terminal_view: false,
        })
    }

    pub fn mode(&self) -> AppMode {
        if self.is_command_mode {
            AppMode::Command
        } else {
            AppMode::SideMenu
        }
    }

    pub fn refresh_state(&mut self) -> Result<()> {
        if let Ok(new_state) = get_project_state(&self.project_path) {
            self.state = new_state;
            self.worktrees = self.state.worktrees.iter().map(|w| w.branch.clone()).collect();
        }
        Ok(())
    }

    pub fn save_history(&mut self, cmd: &str) -> Result<()> {
        // Refresh state before saving history to avoid overwriting changes made by commands
        self.refresh_state()?;

        if !self.state.history.contains(&cmd.to_string()) {
            self.state.history.push(cmd.to_string());
            self.state.update_last_updated();
            save_project_state(&self.project_path, &self.state)?;
        }
        Ok(())
    }
}
