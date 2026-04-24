
use anyhow::Result;
use std::path::Path;
use crate::infrastructure::project_history::get_project_history;
use crate::presentation::cli::hop::app::HopApp;
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

    fn is_match(&self, _app: &HopApp, parts: &[String]) -> bool {
        if !parts.is_empty() {
            if parts[0].parse::<usize>().is_ok() {
                return true;
            }
        }
        parts.get(0).map_or(false, |name| name == self.name())
    }

    fn execute(&self, app: &mut HopApp, parts: Vec<String>) -> Result<bool> {
        let mut parts = parts;
        let mut cmd_to_execute = parts.join(" ");
        let selected_worktree = app.worktrees[app.selected_index].clone();

        if let Ok(index) = parts[0].parse::<usize>() {
            if index > 0 && index <= app.history.input_history.history.len() {
                cmd_to_execute = app.history.input_history.history[index - 1].clone();
                parts = cmd_to_execute.split_whitespace().map(|s| s.to_string()).collect();
            } else {
                let (_term_height, term_width) = app.term.size();
                let right_width = (term_width as usize).saturating_sub(30).saturating_sub(3);
                app.history.push_output(&format!("history index {} out of range", index), right_width);
                app.current_input.clear();
                app.cursor_pos = 0;
                return Ok(true);
            }
        }

        let show_thinking = app.prepare_command_execution(self.name(), &cmd_to_execute);
        let (result, _) = app.run_command_with_parts(parts, &cmd_to_execute);
        app.finalize_command_execution(result, &selected_worktree, &cmd_to_execute, &cmd_to_execute, 0, show_thinking);
        Ok(true)
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
