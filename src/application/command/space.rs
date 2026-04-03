use anyhow::{Result, anyhow};
use clap::Parser;
use std::path::Path;
use crate::application::init::{get_project_state, save_project_state};

#[derive(Parser, Debug)]
#[command(name = "space")]
pub struct SpaceCli {
    pub worktree: Option<String>,
}

pub fn run(args: Vec<String>, project_path: &Path) -> Result<()> {
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
            state.current_worktree = Some(worktree);
        }
        save_project_state(project_path, &state)?;
    }

    Ok(())
}
