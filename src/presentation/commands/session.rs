use anyhow::{Result, anyhow};
use std::path::Path;
use clap::{Parser, Subcommand, CommandFactory};
use crate::infrastructure::project_state::{get_project_state, save_project_state};
use crate::infrastructure::git;
use crate::presentation::commands::Command;

pub struct SessionCommand;

const NAME: &str = "session";
const DESCRIPTION: &str = "Manage sessions";
const HELP: &str = "Manages sessions (new working branches and worktrees).
Usage: session start <branch_name> [--base <base_branch>]
       session close <branch_name> [--delete-branch]
Creates a new branch and sets up a corresponding Git worktree, or closes and removes an existing one.";

impl Command for SessionCommand {
    fn name(&self) -> &str {
        NAME
    }

    fn description(&self) -> &str {
        DESCRIPTION
    }

    fn help(&self) -> &str {
        HELP
    }

    fn run(&self, args: Vec<String>, project_path: &Path) -> Result<String> {
        let cli = match SessionCli::try_parse_from(args) {
            Ok(cli) => cli,
            Err(e) => {
                return Err(anyhow!("{}", e));
            }
        };

        match cli.command {
            Some(SessionCommands::Start { branch, base }) => {
                start_session(&branch, base, project_path)
            }
            Some(SessionCommands::Close { branch, delete_branch }) => {
                close_session(&branch, delete_branch, project_path)
            }
            None => {
                let mut cmd = SessionCli::command();
                let help = cmd.render_help().to_string();
                return Err(anyhow!("Usage:\n{}", help));
            }
        }
    }

    fn subcommands(&self) -> Vec<(String, String)> {
        let cmd = SessionCli::command();
        cmd.get_subcommands()
            .map(|sub| {
                (
                    sub.get_name().to_string(),
                    sub.get_about().map(|a| a.to_string()).unwrap_or_default(),
                )
            })
            .collect()
    }

    fn usage(&self, args: &[&str]) -> Option<String> {
        let cmd = SessionCli::command();
        if args.len() > 1 {
            let sub_name = args[1];
            if let Some(sub) = cmd.get_subcommands().find(|s| s.get_name() == sub_name) {
                let usage = sub.clone().render_usage().to_string();
                let cleaned_usage = usage.replace("session-", "session ");
                return Some(cleaned_usage);
            }
        }
        Some("Usage: session [COMMAND]
Commands:
  start  Start a new session
  close  Close a session
  help   Print this message or the help of the given subcommand(s)".to_string())
    }
}

#[derive(Parser, Debug)]
#[command(name = "session")]
pub struct SessionCli {
    #[command(subcommand)]
    pub command: Option<SessionCommands>,
}

#[derive(Subcommand, Debug)]
pub enum SessionCommands {
    /// Start a new session
    Start {
        /// Branch name
        branch: String,
        /// Base branch (optional, default: origin default branch)
        #[arg(short, long)]
        base: Option<String>,
    },
    /// Close a session
    Close {
        /// Branch name
        branch: String,
        /// Delete the branch as well
        #[arg(short, long)]
        delete_branch: bool,
    },
}

fn start_session(branch: &str, base: Option<String>, project_path: &Path) -> Result<String> {
    if git::branch_exists(branch, project_path)? {
        return Err(anyhow!("Branch '{}' already exists.", branch));
    }

    let base_branch = match base {
        Some(b) => b,
        None => git::get_default_branch(project_path)?,
    };

    let worktree_path = project_path.join(branch);
    if worktree_path.exists() {
        return Err(anyhow!("Directory '{}' already exists.", worktree_path.display()));
    }

    git::create_worktree(project_path, branch, &worktree_path, &base_branch)?;

    let mut state = get_project_state(project_path)?;
    if !state.worktrees.contains(&branch.to_string()) {
        state.worktrees.push(branch.to_string());
    }
    state.current_worktree = Some(branch.to_string());
    save_project_state(project_path, &state)?;

    Ok(format!("Session started: branch '{}' in '{}'", branch, worktree_path.display()))
}

fn close_session(branch: &str, delete_branch: bool, project_path: &Path) -> Result<String> {
    let mut state = get_project_state(project_path)?;

    if !state.worktrees.contains(&branch.to_string()) {
        return Err(anyhow!("Session '{}' not found in project state.", branch));
    }

    let worktree_path = project_path.join(branch);
    if worktree_path.exists() {
        git::remove_worktree(project_path, &worktree_path)?;
    }

    if delete_branch {
        git::delete_branch(project_path, branch)?;
    }

    state.worktrees.retain(|w| w != branch);
    if state.current_worktree.as_deref() == Some(branch) {
        state.current_worktree = state.worktrees.first().cloned();
    }
    save_project_state(project_path, &state)?;

    let mut msg = format!("Session closed: branch '{}' removed", branch);
    if delete_branch {
        msg.push_str(" and deleted");
    }
    Ok(msg)
}
