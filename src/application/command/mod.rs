use anyhow::Result;
use std::path::Path;

pub mod ai;
pub mod close;
pub mod history;
pub mod man;
pub mod session;
pub mod space;

pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn help(&self) -> &str;
    fn run(&self, args: Vec<String>, project_path: &Path) -> Result<String>;
}

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
