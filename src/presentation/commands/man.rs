use anyhow::Result;
use std::path::Path;
use crate::presentation::commands::{Command, get_commands};

pub struct ManCommand;

const NAME: &str = "man";
const DESCRIPTION: &str = "Show manual";
const HELP: &str = "Shows a list of available commands or help for a specific command.
Usage: man [command]";

impl Command for ManCommand {
    fn name(&self) -> &str {
        NAME
    }

    fn description(&self) -> &str {
        DESCRIPTION
    }

    fn help(&self) -> &str {
        HELP
    }

    fn subcommands(&self) -> Vec<(String, String)> {
        get_commands()
            .into_iter()
            .map(|c| (c.name().to_string(), c.description().to_string()))
            .collect()
    }

    fn usage(&self, _args: &[&str]) -> Option<String> {
        Some("Usage: man [COMMAND]".to_string())
    }

    fn run(&self, args: Vec<String>, _project_path: &Path, _current_worktree: &str, _term: &console::Term) -> Result<String> {
        let commands = get_commands();
        if args.len() > 1 {
            let cmd_name = &args[1];
            if let Some(cmd) = commands.iter().find(|c| c.name() == cmd_name) {
                return Ok(format!(
                    "Command: {}\nDescription: {}\nHelp:\n{}",
                    cmd.name(),
                    cmd.description(),
                    cmd.help()
                ));
            } else {
                return Ok(format!("Command '{}' not found.", cmd_name));
            }
        }

        let mut output = String::from("Available commands:\n");
        for cmd in commands {
            output.push_str(&format!("  {:10} {}\n", cmd.name(), cmd.description()));
        }
        output.push_str("\nYou can show detailed help with 'man <command>'.");
        Ok(output)
    }
}
