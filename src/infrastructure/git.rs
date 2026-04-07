use anyhow::{Result, Context, anyhow};
use std::path::Path;
use std::process::Command as ProcessCommand;

/// Clones a repository into `target`, optionally checking out `branch`.
pub fn clone(url: &str, target: &Path, branch: Option<&str>) -> Result<()> {
    let mut builder = git2::build::RepoBuilder::new();
    if let Some(b) = branch {
        builder.branch(b);
    }
    builder.clone(url, target).context("Failed to clone repository")?;
    Ok(())
}

/// Creates a new git worktree at `worktree_path` based on `base_branch`.
pub fn create_worktree(
    project_path: &Path,
    branch: &str,
    worktree_path: &Path,
    base_branch: &str,
) -> Result<()> {
    let status = ProcessCommand::new("git")
        .arg("-C")
        .arg(project_path.join("main"))
        .arg("worktree")
        .arg("add")
        .arg("-b")
        .arg(branch)
        .arg(worktree_path)
        .arg(base_branch)
        .status()
        .context("Failed to execute git worktree add")?;

    if !status.success() {
        return Err(anyhow!("git worktree add failed."));
    }
    Ok(())
}

/// Returns `true` if `branch` already exists in the repository under `project_path/main`.
pub fn branch_exists(branch: &str, project_path: &Path) -> Result<bool> {
    let output = ProcessCommand::new("git")
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

/// Returns the default remote branch (e.g. `origin/main`) for the repo at `project_path/main`.
pub fn get_default_branch(project_path: &Path) -> Result<String> {
    let output = ProcessCommand::new("git")
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
