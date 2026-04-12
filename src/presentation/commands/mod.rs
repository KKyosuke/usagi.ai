use anyhow::Result;
use std::path::Path;
use console::Term;

pub mod close;
pub mod doctor;
pub mod history;
pub mod man;
pub mod session;
pub mod space;
pub mod terminal;

/// Interface that every TUI command must implement.
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn help(&self) -> &str;
    fn run(&self, args: Vec<String>, project_path: &Path, current_worktree: &str, term: &Term) -> Result<String>;
    fn subcommands(&self) -> Vec<(String, String)> {
        vec![]
    }
    fn usage(&self, _args: &[&str]) -> Option<String> {
        None
    }
}

/// Returns all built-in TUI commands.
pub fn get_commands() -> Vec<Box<dyn Command>> {
    vec![
        Box::new(close::CloseCommand),
        Box::new(doctor::DoctorCommand),
        Box::new(history::HistoryCommand),
        Box::new(man::ManCommand),
        Box::new(session::SessionCommand),
        Box::new(space::SpaceCommand),
        Box::new(terminal::TerminalCommand),
    ]
}
