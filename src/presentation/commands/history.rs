
use anyhow::Result;
use std::path::Path;
use crate::infrastructure::persistence::get_project_state;
use crate::presentation::commands::Command;

pub struct HistoryCommand;

const NAME: &str = "history";
const DESCRIPTION: &str = "Show command history";
const HELP: &str = "Shows the history of commands executed so far.
You can also re-run a command by specifying its history number.
Example: Enter '1' to execute the first command in history.";

impl Command for HistoryCommand {
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
        Some("Usage: history".to_string())
    }

    fn run(&self, _args: Vec<String>, project_path: &Path) -> Result<String> {
        let state = get_project_state(project_path)?;
        let mut output = String::new();

        if state.history.is_empty() {
            output.push_str("No history found.");
        } else {
            for (i, entry) in state.history.iter().enumerate() {
                output.push_str(&format!("{:4} {}\n", i + 1, entry));
            }
        }

        Ok(output)
    }
}
