use anyhow::Result;
use std::path::Path;
use super::Command;

pub struct AiCommand;

const NAME: &str = "ai";
const DESCRIPTION: &str = "Call the AI";
const HELP: &str = "Calls the AI.
Usage: ai <message>
You can ask questions or give instructions to the AI based on the current context.";

impl Command for AiCommand {
    fn name(&self) -> &str {
        NAME
    }

    fn description(&self) -> &str {
        DESCRIPTION
    }

    fn help(&self) -> &str {
        HELP
    }

    fn run(&self, _args: Vec<String>, _project_path: &Path) -> Result<String> {
        Ok("ai".to_string())
    }
}

pub fn run(args: Vec<String>, project_path: &Path) -> Result<String> {
    AiCommand.run(args, project_path)
}
