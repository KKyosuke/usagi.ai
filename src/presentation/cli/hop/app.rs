use anyhow::{Result, anyhow, Context};
use std::path::PathBuf;
use console::Term;
use crate::domain::project::ProjectState;
use crate::infrastructure::project_state::get_project_state;
use crate::presentation::cli::hop::history_manager::HistoryManager;
use crate::presentation::commands::{self, Command};
use crate::presentation::tui::mode::AppMode;

pub struct SelectModal {
    pub title: String,
    pub items: Vec<String>,
    pub selected_index: usize,
    pub on_select: Box<dyn FnOnce(&mut HopApp, String) -> Result<()>>,
}

pub struct InputModal {
    pub title: String,
    pub value: String,
    pub on_submit: Box<dyn FnOnce(&mut HopApp, String) -> Result<()>>,
}

pub struct HopApp {
    pub project_path: PathBuf,
    pub state: ProjectState,
    pub history: HistoryManager,
    pub worktrees: Vec<String>,
    pub term: Term,
    pub selected_index: usize,
    pub current_input: String,
    pub cursor_pos: usize,
    pub is_command_mode: bool,
    pub commands: Vec<Box<dyn Command>>,
    pub is_terminal_view: bool,
    pub is_ai_chat_mode: bool,
    pub tab_completion_base: Option<String>,
    pub suggestion_index: Option<usize>,
    pub select_modal: Option<SelectModal>,
    pub input_modal: Option<InputModal>,
}

impl HopApp {
    pub fn new(project_path: PathBuf, initial_worktree: Option<String>) -> Result<Self> {
        let state = get_project_state(&project_path)
            .map_err(|_| anyhow!("Error: Not an initialized directory. Please run `usagi init` first."))?;

        let term = Term::stdout();
        let term_height = term.size().0 as usize;
        let max_history = term_height.saturating_sub(7).max(1);
        let history = HistoryManager::new(project_path.clone(), max_history)?;

        std::env::set_current_dir(&project_path).context(format!("Failed to change directory to {}", project_path.display()))?;

        let mut state = state;
        let mut changed = false;
        for worktree in &mut state.worktrees {
            let worktree_path = project_path.join(&worktree.directory);
            if worktree_path.exists() {
                if let Ok(has_upstream) = crate::infrastructure::git::has_upstream(&worktree_path) {
                    if worktree.has_upstream != has_upstream {
                        worktree.has_upstream = has_upstream;
                        changed = true;
                    }
                }
            }
        }
        if changed {
            let _ = crate::infrastructure::project_state::save_project_state(&project_path, &state);
        }

        let worktrees: Vec<String> = state.worktrees.iter().map(|w| w.branch.clone()).collect();
        
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

        let commands = commands::get_commands();

        Ok(Self {
            project_path,
            state,
            history,
            worktrees,
            term,
            selected_index,
            current_input: String::new(),
            cursor_pos: 0,
            is_command_mode: false,
            commands,
            is_terminal_view: false,
            is_ai_chat_mode: false,
            tab_completion_base: None,
            suggestion_index: None,
            select_modal: None,
            input_modal: None,
        })
    }

    pub fn mode(&self) -> AppMode {
        if self.is_ai_chat_mode {
            AppMode::AiChat
        } else if self.is_command_mode {
            AppMode::Command
        } else {
            AppMode::SideMenu
        }
    }

    pub fn refresh_state(&mut self) -> Result<()> {
        if let Ok(mut new_state) = get_project_state(&self.project_path) {
            let mut changed = false;
            for worktree in &mut new_state.worktrees {
                let worktree_path = self.project_path.join(&worktree.directory);
                if worktree_path.exists() {
                    if let Ok(has_upstream) = crate::infrastructure::git::has_upstream(&worktree_path) {
                        if worktree.has_upstream != has_upstream {
                            worktree.has_upstream = has_upstream;
                            changed = true;
                        }
                    }
                }
            }
            if changed {
                let _ = crate::infrastructure::project_state::save_project_state(&self.project_path, &new_state);
            }
            self.state = new_state;
            self.worktrees = self.state.worktrees.iter().map(|w| w.branch.clone()).collect();
        }
        self.history.refresh()?;
        Ok(())
    }

    pub fn save_history(&mut self, cmd: &str) -> Result<()> {
        self.refresh_state()?;
        if !self.is_ai_chat_mode {
            self.history.save_input(cmd)?;
        }
        Ok(())
    }
}
