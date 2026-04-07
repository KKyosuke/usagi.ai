
use anyhow::Result;
use std::path::Path;
use crate::application::init::get_project_state;
use super::Command;

pub struct HistoryCommand;

impl Command for HistoryCommand {
    fn name(&self) -> &str {
        "history"
    }

    fn description(&self) -> &str {
        "コマンド履歴を表示する"
    }

    fn help(&self) -> &str {
        "これまでに実行したコマンドの履歴を表示します。
各履歴の番号を指定してコマンドを再実行することも可能です。
例: '1' と入力すると履歴の1番目のコマンドを実行します。"
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

pub fn run(args: Vec<String>, project_path: &Path) -> Result<String> {
    HistoryCommand.run(args, project_path)
}
