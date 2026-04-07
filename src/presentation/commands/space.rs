use anyhow::{Result, anyhow};
use clap::Parser;
use std::path::Path;
use crate::infrastructure::persistence::{get_project_state, save_project_state};
use crate::presentation::commands::Command;

pub struct SpaceCommand;

const NAME: &str = "space";
const DESCRIPTION: &str = "Switch workspace";
const HELP: &str = "Switches the workspace.
Usage: space <worktree_name>
Switches the current working directory to the specified worktree.
Specifying 'main' returns to the main worktree.";

impl Command for SpaceCommand {
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
        Some("Usage: space <WORKTREE>".to_string())
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
