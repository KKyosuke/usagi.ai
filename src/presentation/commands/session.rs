use anyhow::{Result, anyhow};
use std::path::Path;
use clap::{Parser, Subcommand, CommandFactory};
use crate::infrastructure::project_state::{get_project_state, save_project_state};
use crate::infrastructure::git;
use crate::domain::project::Worktree;
use crate::presentation::commands::Command;

pub struct SessionCommand;

const NAME: &str = "session";
const DESCRIPTION: &str = "Manage sessions";
const HELP: &str = "Manages sessions (new working branches and worktrees).
Usage: session start <branch_name> [--base <base_branch>]
       session close <branch_name>
       session update [--all] [--base <base_branch>]
Closes and removes an existing session (removes worktree and deletes the local branch).";

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
            Some(SessionCommands::Close { branch }) => {
                close_session(&branch, project_path)
            }
            Some(SessionCommands::Update { all, base }) => {
                update_sessions(all, base, project_path)
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
  start   Start a new session
  close   Close a session
  update  Update session(s)
  help    Print this message or the help of the given subcommand(s)".to_string())
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
    },
    /// Update session(s)
    Update {
        /// Update all sessions
        #[arg(short, long)]
        all: bool,
        /// Base branch to update from (optional, default: origin default branch)
        #[arg(short, long)]
        base: Option<String>,
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
    if !state.worktrees.iter().any(|w| w.branch == branch) {
        let mut worktree = Worktree {
            branch: branch.to_string(),
            directory: branch.to_string(),
            default: false,
            modified_at: "".to_string(),
        };
        worktree.update_modified_at();
        state.worktrees.push(worktree);
    }
    state.current_worktree = Some(branch.to_string());
    state.update_last_updated();
    save_project_state(project_path, &state)?;

    Ok(format!("Session started: branch '{}' in '{}'", branch, worktree_path.display()))
}

fn close_session(branch: &str, project_path: &Path) -> Result<String> {
    let mut state = get_project_state(project_path)?;

    if !state.worktrees.iter().any(|w| w.branch == branch) {
        return Err(anyhow!("Session '{}' not found in project state.", branch));
    }

    let worktree_path = project_path.join(branch);
    if worktree_path.exists() {
        git::remove_worktree(project_path, &worktree_path)?;
    }

    git::delete_branch(project_path, branch)?;

    state.worktrees.retain(|w| w.branch != branch);
    if state.current_worktree.as_deref() == Some(branch) {
        state.current_worktree = state.worktrees.first().map(|w| w.branch.clone());
    }
    state.update_last_updated();
    save_project_state(project_path, &state)?;

    Ok(format!("Session closed: branch '{}' removed and deleted", branch))
}

fn update_sessions(all: bool, base: Option<String>, project_path: &Path) -> Result<String> {
    let state = get_project_state(project_path)?;

    let base_branch = match base {
        Some(b) => b,
        None => git::get_default_branch(project_path)?,
    };

    if base_branch.contains('/') {
        if let Some(remote) = base_branch.split('/').next() {
            if !remote.is_empty() {
                println!("Fetching remote '{}'...", remote);
                git::fetch(project_path, remote)?;
            }
        }
    }

    let sessions_to_update = if all {
        if state.worktrees.is_empty() {
            return Ok("No sessions found to update.".to_string());
        }
        state.worktrees.iter().map(|w| w.branch.clone()).collect::<Vec<String>>()
    } else {
        match state.current_worktree {
            Some(ref w) => vec![w.clone()],
            None => return Err(anyhow!("No current session selected.")),
        }
    };

    let mut results = Vec::new();
    for session in sessions_to_update {
        let worktree_path = project_path.join(&session);
        if worktree_path.exists() {
            println!("Updating session '{}' with base '{}'...", session, base_branch);
            match git::rebase(&worktree_path, &base_branch) {
                Ok(_) => results.push(format!("Session '{}' updated successfully.", session)),
                Err(e) => results.push(format!("Failed to update session '{}': {}", session, e)),
            }
        } else {
            results.push(format!("Worktree for session '{}' does not exist at '{}'.", session, worktree_path.display()));
        }
    }

    Ok(results.join("\n"))
}
