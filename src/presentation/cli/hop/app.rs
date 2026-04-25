use anyhow::{Result, anyhow, Context};
use std::path::PathBuf;
use std::sync::Arc;
use console::{Term, style};
use futures::future::BoxFuture;
use crate::domain::project::ProjectState;
use crate::infrastructure::project_state::get_project_state;
use crate::presentation::cli::hop::history_manager::HistoryManager;
use crate::presentation::commands::{self, Command};
use crate::presentation::tui::mode::AppMode;

pub struct SelectModal {
    pub title: String,
    pub items: Vec<String>,
    pub selected_index: usize,
    pub on_select: Box<dyn for<'a> FnOnce(&'a mut HopApp, String) -> BoxFuture<'a, Result<()>> + Send>,
}

pub struct InputModal {
    pub title: String,
    pub value: String,
    pub on_submit: Box<dyn for<'a> FnOnce(&'a mut HopApp, String) -> BoxFuture<'a, Result<()>> + Send>,
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
    pub commands: Vec<Arc<dyn Command>>,
    pub is_terminal_view: bool,
    pub active_interaction: Option<Arc<dyn Command>>,
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
            active_interaction: None,
            tab_completion_base: None,
            suggestion_index: None,
            select_modal: None,
            input_modal: None,
        })
    }

    pub fn mode(&self) -> AppMode {
        if self.active_interaction.is_some() {
            AppMode::Interaction
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

    pub fn is_ai_chat_mode(&self) -> bool {
        self.active_interaction.as_ref().map(|c| c.name() == "ai").unwrap_or(false)
    }

    pub fn save_history(&mut self, cmd: &str) -> Result<()> {
        self.refresh_state()?;
        if !self.is_ai_chat_mode() {
            self.history.save_input(cmd)?;
        }
        Ok(())
    }

    pub fn handle_command_result(&mut self, result: Result<String>, selected_worktree: &str, cmd_to_execute: &str, backup_input: &str, backup_cursor: usize) {
        let (term_height, term_width) = self.term.size();
        let left_width = 30;
        let right_width = (term_width as usize).saturating_sub(left_width).saturating_sub(3);

        let parts: Vec<&str> = cmd_to_execute.split_whitespace().collect();
        let cmd_name = parts.first().unwrap_or(&"");
        let is_session_close = *cmd_name == "session" && parts.get(1) == Some(&"close");

        if *cmd_name == "close" || is_session_close {
            if result.is_ok() {
                self.is_command_mode = false;
                let _ = self.refresh_state();
                
                if let Some(idx) = self.worktrees.iter().position(|wt| wt == selected_worktree) {
                    self.selected_index = idx;
                } else if let Some(current_wt) = &self.state.current_worktree {
                    if let Some(idx) = self.worktrees.iter().position(|wt| wt == current_wt) {
                        self.selected_index = idx;
                    }
                } else {
                    self.selected_index = 0;
                }
            }
        } else {
            match result {
                Ok(output) => {
                    self.history.push_output(&output, right_width);
                }
                Err(e) => {
                    self.history.push_output(&e.to_string(), right_width);
                    // Restore input on error so they don't have to re-type
                    self.current_input = backup_input.to_string();
                    self.cursor_pos = backup_cursor;
                }
            }

            // 状態を更新し（save_history内でrefresh_stateを呼ぶ）、履歴を保存
            let _ = self.save_history(cmd_to_execute);
            
            let mut updated = false;
            if *cmd_name == "space" {
                if let Some(current_wt) = &self.state.current_worktree {
                    if let Some(idx) = self.worktrees.iter().position(|wt| wt == current_wt) {
                        self.selected_index = idx;
                        updated = true;
                    }
                }
            }

            if !updated {
                if let Some(idx) = self.worktrees.iter().position(|wt| wt == selected_worktree) {
                    self.selected_index = idx;
                } else if let Some(current_wt) = &self.state.current_worktree {
                    if let Some(idx) = self.worktrees.iter().position(|wt| wt == current_wt) {
                        self.selected_index = idx;
                    }
                } else {
                    self.selected_index = 0;
                }
            }
        }
        
        self.history.limit_output((term_height as usize).saturating_sub(7).max(1));
    }

    pub fn prepare_command_execution(&mut self, parts: &[String], cmd_to_execute: &str) -> bool {
        let cmd_name = if parts.is_empty() { "" } else { &parts[0] };
        self.is_terminal_view = cmd_name == "terminal";

        let (_term_height, term_width) = self.term.size();
        let right_width = (term_width as usize).saturating_sub(30).saturating_sub(3);

        let is_session_close = cmd_name == "session" && cmd_to_execute.contains("close");
        let selected_worktree = self.worktrees[self.selected_index].clone();

        if cmd_name != "close" && !is_session_close && !cmd_name.is_empty() {
            let prompt_sign = if self.is_ai_chat_mode() { "(ai) >" } else if self.is_terminal_view { "$" } else { ">" };
            let prompt = format!("{} {} {}", style(&selected_worktree).cyan(), prompt_sign, cmd_to_execute);
            self.history.push_output(&prompt, right_width);
        }

        let mut show_thinking = false;
        if cmd_name == "ai" && (self.is_ai_chat_mode() || (parts.len() > 1 && parts[1] != "--help" && parts[1] != "-h")) {
            self.history.push_output(&format!("{}", style("🐰 usagi is thinking..").dim().italic()), right_width);
            show_thinking = true;
        }

        self.current_input.clear();
        self.cursor_pos = 0;

        let _ = crate::presentation::cli::hop::ui::render(self);
        let _ = self.term.flush();
        
        show_thinking
    }

    pub fn finalize_command_execution(&mut self, result: Result<String>, selected_worktree: &str, cmd_to_execute: &str, backup_input: &str, backup_cursor: usize, show_thinking: bool) {
        // Re-enter alternate screen and hide cursor to ensure TUI state
        let _ = self.term.write_str("\x1b[?1049h");
        let _ = self.term.hide_cursor();
        let _ = self.term.flush();

        if show_thinking {
            self.history.pop_output();
        }

        self.handle_command_result(result, selected_worktree, cmd_to_execute, backup_input, backup_cursor);
    }

    pub async fn run_command_with_parts(&mut self, parts: Vec<String>, cmd_to_execute: &str) -> (Result<String>, bool) {
        let show_thinking = self.prepare_command_execution(&parts, cmd_to_execute);
        let cmd_name = if parts.is_empty() { "".to_string() } else { parts[0].clone() };
        
        let selected_worktree = self.worktrees[self.selected_index].clone();

        let command = self.commands.iter()
            .find(|c| c.name() == cmd_name)
            .map(|c| Arc::clone(c));

        let parts = parts;
        let result: Result<String> = if let Some(cmd) = command {
            cmd.run(parts, &self.project_path, &selected_worktree, &self.term).await
        } else {
            if cmd_name.is_empty() {
                Ok("".to_string())
            } else {
                Ok(format!("no such command in usagi: {}", cmd_name))
            }
        };
        
        (result, show_thinking)
    }
}
