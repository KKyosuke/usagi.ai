use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use console::Term;
use crate::presentation::cli::hop::app::HopApp;

pub mod close;
pub mod doctor;
pub mod history;
pub mod man;
pub mod session;
pub mod space;
pub mod terminal;
pub mod ai;

/// Interface that every TUI command must implement.
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn help(&self) -> &str;
    fn run(&self, args: Vec<String>, project_path: &Path, current_worktree: &str, term: &Term) -> Result<String>;
    fn is_match(&self, _app: &HopApp, parts: &[String]) -> bool {
        parts.get(0).map_or(false, |name| name == self.name())
    }
    fn execute(&self, _app: &mut HopApp, _parts: Vec<String>) -> Result<bool> {
        Ok(false)
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
