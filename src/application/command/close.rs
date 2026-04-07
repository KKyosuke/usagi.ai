use anyhow::Result;
use std::path::Path;
use super::Command;

pub struct CloseCommand;

impl Command for CloseCommand {
    fn name(&self) -> &str {
        "close"
    }

    fn description(&self) -> &str {
        "セッションを終了する"
    }

    fn help(&self) -> &str {
        "セッションを終了し、作業ディレクトリ選択画面に戻ります。"
    }

    fn run(&self, _args: Vec<String>, _project_path: &Path) -> Result<String> {
        Ok("close".to_string())
    }
}

pub fn run(args: Vec<String>, project_path: &Path) -> Result<String> {
    CloseCommand.run(args, project_path)
}
