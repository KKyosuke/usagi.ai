use anyhow::Result;
use std::path::Path;
use super::Command;

pub struct AiCommand;

impl Command for AiCommand {
    fn name(&self) -> &str {
        "ai"
    }

    fn description(&self) -> &str {
        "AIを呼び出す"
    }

    fn help(&self) -> &str {
        "AIを呼び出す。
使用法: ai <message>
現在のコンテキストに基づいてAIに質問や指示を行うことができます。"
    }

    fn run(&self, _args: Vec<String>, _project_path: &Path) -> Result<String> {
        Ok("ai".to_string())
    }
}

pub fn run(args: Vec<String>, project_path: &Path) -> Result<String> {
    AiCommand.run(args, project_path)
}
