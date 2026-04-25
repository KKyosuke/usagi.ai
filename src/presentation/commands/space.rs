use anyhow::{Result, anyhow};
use clap::Parser;
use std::path::Path;
use async_trait::async_trait;
use console::Term;
use crate::infrastructure::project_state::{get_project_state, save_project_state};
use crate::presentation::commands::{Command, CommandContext, CommandAction};

pub struct SpaceCommand;

const NAME: &str = "space";
const DESCRIPTION: &str = "Switch workspace";
const HELP: &str = "Switches the workspace.
Usage: space <worktree_name>
Switches the current working directory to the specified worktree.
Specifying 'main' returns to the main worktree.";

#[async_trait]
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

    fn should_sync_selection(&self, _parts: &[String]) -> bool {
        true
    }

    async fn execute(&self, context: CommandContext) -> Result<CommandAction> {
        let mut parts = context.parts;
        let cmd_to_execute = parts.join(" ");

        if parts.len() == 1 {
            parts.push(context.worktrees[context.selected_index].clone());
        }

        Ok(CommandAction::RunCommand {
            parts,
            cmd_to_execute,
            close_after: true,
        })
    }

    async fn run(&self, args: Vec<String>, project_path: &Path, _current_worktree: &str, _term: &Term) -> Result<String> {
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
                if let Some(w) = state.worktrees.iter_mut().find(|w| w.branch == worktree) {
                    w.update_modified_at();
                    state.current_worktree = Some(worktree.clone());
                } else {
                    return Err(anyhow!("Worktree '{}' does not exist.", worktree));
                }
            }
            state.update_last_updated();
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
