use anyhow::Result;
use std::path::Path;
use async_trait::async_trait;
use console::Term;
use crate::presentation::commands::Command;

pub struct CloseCommand;

const NAME: &str = "close";
const DESCRIPTION: &str = "Close the session";
const HELP: &str = "Closes the session and returns to the directory selection screen.";

#[async_trait]
impl Command for CloseCommand {
    fn name(&self) -> &str {
        NAME
    }

    fn description(&self) -> &str {
        DESCRIPTION
    }

    fn help(&self) -> &str {
        HELP
    }

    fn usage(&self, _args: &[&str]) -> Option<String> {
        Some("Usage: close".to_string())
    }

    fn should_close_command_mode(&self, _parts: &[String]) -> bool {
        true
    }

    async fn run(&self, _args: Vec<String>, _project_path: &Path, _current_worktree: &str, _term: &Term) -> Result<String> {
        Ok("close".to_string())
    }
}
