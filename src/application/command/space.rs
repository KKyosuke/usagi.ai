use anyhow::{Result, anyhow};
use clap::Parser;
use std::path::Path;
use crate::application::init::{get_project_state, save_project_state};
use super::Command;

pub struct SpaceCommand;

impl Command for SpaceCommand {
    fn name(&self) -> &str {
        "space"
    }

    fn description(&self) -> &str {
        "作業スペースを切り替える"
    }

    fn help(&self) -> &str {
        "作業スペースを切り替える。
使用法: space <worktree_name>
指定したワークツリーに現在の作業ディレクトリを切り替えます。
'main' を指定するとメインのワークツリーに戻ります。"
    }

    fn run(&self, args: Vec<String>, project_path: &Path) -> Result<String> {
        let cli = match SpaceCli::try_parse_from(args) {
            Ok(cli) => cli,
            Err(e) => {
                return Err(anyhow!("{}", e));
            }
        };

        if let Some(worktree) = cli.worktree {
            let mut state = get_project_state(project_path)?;
            if worktree == "main" {
                state.current_worktree = None;
            } else {
                if !state.worktrees.contains(&worktree) {
                    return Err(anyhow!("Worktree '{}' does not exist.", worktree));
                }
                state.current_worktree = Some(worktree.clone());
            }
            save_project_state(project_path, &state)?;
            Ok(format!("Switched to workspace '{}'", worktree))
        } else {
            Ok("".to_string())
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "space")]
pub struct SpaceCli {
    pub worktree: Option<String>,
}

pub fn run(args: Vec<String>, project_path: &Path) -> Result<String> {
    SpaceCommand.run(args, project_path)
}
