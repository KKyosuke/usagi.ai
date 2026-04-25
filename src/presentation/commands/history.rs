
use anyhow::Result;
use std::path::Path;
use crate::infrastructure::project_history::get_project_history;
use crate::presentation::commands::{Command, CommandContext, CommandAction};

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

    fn is_match(&self, context: &CommandContext) -> bool {
        let parts = &context.parts;
        if !parts.is_empty() {
            if parts[0].parse::<usize>().is_ok() {
                return true;
            }
        }
        parts.get(0).map_or(false, |name| name == self.name())
    }

    fn execute(&self, context: CommandContext) -> Result<CommandAction> {
        let mut parts = context.parts;
        let mut cmd_to_execute = parts.join(" ");

        if let Ok(index) = parts[0].parse::<usize>() {
            if index > 0 && index <= context.input_history.len() {
                cmd_to_execute = context.input_history[index - 1].clone();
                parts = cmd_to_execute.split_whitespace().map(|s| s.to_string()).collect();
            } else {
                return Ok(CommandAction::DisplayMessage(format!("history index {} out of range", index)));
            }
        }

        Ok(CommandAction::RunCommand {
            parts,
            cmd_to_execute,
            close_after: false,
        })
    }

    fn usage(&self, _args: &[&str]) -> Option<String> {
        Some("Usage: history".to_string())
    }

    fn run(&self, _args: Vec<String>, project_path: &Path, _current_worktree: &str, _term: &console::Term) -> Result<String> {
        let project_history = get_project_history(project_path)?;
        let mut output = String::new();

        if project_history.history.is_empty() {
            output.push_str("No history found.");
        } else {
            for (i, entry) in project_history.history.iter().enumerate() {
                output.push_str(&format!("{:4} {}\n", i + 1, entry));
            }
        }

        Ok(output)
    }
}
