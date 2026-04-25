use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use console::Term;
use crate::presentation::cli::hop::app::{SelectModal, InputModal};
use crate::domain::project::ProjectState;

pub mod close;
pub mod doctor;
pub mod history;
pub mod man;
pub mod session;
pub mod space;
pub mod terminal;
pub mod ai;

pub struct CommandContext<'a> {
    pub parts: Vec<String>,
    pub state: &'a ProjectState,
    pub worktrees: &'a [String],
    pub selected_index: usize,
    pub is_interaction_mode: bool,
    pub input_history: &'a [String],
    pub project_path: &'a Path,
}

pub enum CommandAction {
    None,
    Consumed,
    ClearInput,
    DisplayMessage(String),
    SetSelectModal(SelectModal),
    SetInputModal(InputModal),
    EnterInteraction(String),
    ExitInteraction(String),
    RunCommand {
        parts: Vec<String>,
        cmd_to_execute: String,
        close_after: bool,
    },
    Exit,
}

/// Interface that every TUI command must implement.
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn help(&self) -> &str;
    fn run(&self, args: Vec<String>, project_path: &Path, current_worktree: &str, term: &Term) -> Result<String>;
    fn is_match(&self, context: &CommandContext) -> bool {
        context.parts.get(0).map_or(false, |name| name == self.name())
    }
    fn execute(&self, _context: CommandContext) -> Result<CommandAction> {
        Ok(CommandAction::None)
    }
    fn interact(&self, _context: CommandContext) -> Result<CommandAction> {
        Ok(CommandAction::None)
    }
    fn subcommands(&self) -> Vec<(String, String)> {
        vec![]
    }
    fn usage(&self, _args: &[&str]) -> Option<String> {
        None
    }
}

/// Returns all built-in TUI commands.
pub fn get_commands() -> Vec<Arc<dyn Command>> {
    vec![
        Arc::new(close::CloseCommand),
        Arc::new(doctor::DoctorCommand),
        Arc::new(history::HistoryCommand),
        Arc::new(man::ManCommand),
        Arc::new(session::SessionCommand),
        Arc::new(space::SpaceCommand),
        Arc::new(terminal::TerminalCommand),
        Arc::new(ai::AiCommand),
    ]
}
