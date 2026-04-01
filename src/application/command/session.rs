use anyhow::{Result, Context, anyhow};
use std::process::Command;
use std::path::Path;
use clap::{Parser, Subcommand, CommandFactory};
use crate::application::init::{get_project_state, save_project_state};

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
}

pub fn run(args: Vec<String>, project_path: &Path) -> Result<()> {
    let cli = match SessionCli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) => {
            return Err(anyhow!("{}", e));
        }
    };

    match cli.command {
        Some(SessionCommands::Start { branch, base }) => {
            start_session(&branch, base, project_path)?;
        }
        None => {
            let mut cmd = SessionCli::command();
            let help = cmd.render_help().to_string();
            return Err(anyhow!("Usage:\n{}", help));
        }
    }
    Ok(())
}

fn start_session(branch: &str, base: Option<String>, project_path: &Path) -> Result<()> {
    // 1. すでにあるブランチ名の時はエラーを表示
    if branch_exists(branch, project_path)? {
        return Err(anyhow!("Branch '{}' already exists.", branch));
    }

    // 2. base branch の決定
    let base_branch = match base {
        Some(b) => b,
        None => get_default_branch(project_path)?,
    };

    // 3. worktree の作成
    // path は branch 名にする
    let worktree_path = project_path.join(branch);
    if worktree_path.exists() {
        return Err(anyhow!("Directory '{}' already exists.", worktree_path.display()));
    }

    let status = Command::new("git")
        .arg("-C")
        .arg(project_path.join("main"))
        .arg("worktree")
        .arg("add")
        .arg("-b")
        .arg(branch)
        .arg(&worktree_path)
        .arg(base_branch)
        .status()
        .context("Failed to execute git worktree add")?;

    if !status.success() {
        return Err(anyhow!("git worktree add failed."));
    }

    // 4. state.json の更新
    let mut state = get_project_state(project_path)?;
    if !state.worktrees.contains(&branch.to_string()) {
        state.worktrees.push(branch.to_string());
    }
    state.current_worktree = Some(branch.to_string());
    save_project_state(project_path, &state)?;

    println!("Session started: branch '{}' in '{}'", branch, worktree_path.display());
    Ok(())
}

fn branch_exists(branch: &str, project_path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project_path.join("main"))
        .arg("branch")
        .arg("--list")
        .arg(branch)
        .output()
        .context("Failed to execute git branch")?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(!stdout.trim().is_empty())
}

fn get_default_branch(project_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project_path.join("main"))
        .arg("symbolic-ref")
        .arg("refs/remotes/origin/HEAD")
        .output()
        .context("Failed to get default branch")?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(branch) = stdout.trim().strip_prefix("refs/remotes/") {
            return Ok(branch.to_string());
        }
    }
    
    Ok("origin/main".to_string())
}
