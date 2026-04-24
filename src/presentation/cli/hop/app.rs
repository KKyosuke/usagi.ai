use anyhow::{Result, anyhow, Context};
use std::path::PathBuf;
use console::Term;
use crate::domain::project::ProjectState;
use crate::infrastructure::project_state::get_project_state;
use crate::presentation::cli::hop::history_manager::HistoryManager;
use crate::presentation::commands::{self, Command};
use crate::presentation::tui::mode::AppMode;

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
    pub is_modal_mode: bool,
    pub modal_title: String,
    pub modal_items: Vec<String>,
    pub modal_selected_index: usize,
    pub modal_on_select: Option<Box<dyn FnOnce(&mut HopApp, String) -> Result<()>>>,
    pub is_input_modal_mode: bool,
    pub input_modal_title: String,
    pub input_modal_value: String,
    pub input_modal_on_submit: Option<Box<dyn FnOnce(&mut HopApp, String) -> Result<()>>>,
    pub is_model_selection_mode: bool,
    pub available_models: Vec<String>,
    pub model_selection_index: usize,
    pub enter_chat_on_selection: bool,
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
            is_modal_mode: false,
            modal_title: String::new(),
            modal_items: Vec::new(),
            modal_selected_index: 0,
            modal_on_select: None,
            is_input_modal_mode: false,
            input_modal_title: String::new(),
            input_modal_value: String::new(),
            input_modal_on_submit: None,
            is_model_selection_mode: false,
            available_models: Vec::new(),
            model_selection_index: 0,
            enter_chat_on_selection: false,
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
        if let Ok(new_state) = get_project_state(&self.project_path) {
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
