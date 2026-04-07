use anyhow::Result;
use std::path::Path;

pub mod ai;
pub mod close;
pub mod history;
pub mod man;
pub mod session;
pub mod space;

/// Interface that every TUI command must implement.
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn help(&self) -> &str;
    fn run(&self, args: Vec<String>, project_path: &Path) -> Result<String>;
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
        Box::new(ai::AiCommand),
        Box::new(close::CloseCommand),
        Box::new(history::HistoryCommand),
        Box::new(man::ManCommand),
        Box::new(session::SessionCommand),
        Box::new(space::SpaceCommand),
    ]
}
