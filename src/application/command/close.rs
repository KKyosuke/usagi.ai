use anyhow::Result;
use std::path::Path;
use super::Command;

pub struct CloseCommand;

const NAME: &str = "close";
const DESCRIPTION: &str = "Close the session";
const HELP: &str = "Closes the session and returns to the directory selection screen.";

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

    fn run(&self, _args: Vec<String>, _project_path: &Path) -> Result<String> {
        Ok("close".to_string())
    }
}

pub fn run(args: Vec<String>, project_path: &Path) -> Result<String> {
    CloseCommand.run(args, project_path)
}
