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

/// Removes a git worktree at `worktree_path`.
pub fn remove_worktree(project_path: &Path, worktree_path: &Path) -> Result<()> {
    let status = ProcessCommand::new("git")
        .arg("-C")
        .arg(project_path.join("main"))
        .arg("worktree")
        .arg("remove")
        .arg(worktree_path)
        .status()
        .context("Failed to execute git worktree remove")?;

    if !status.success() {
        return Err(anyhow!("git worktree remove failed."));
    }
    Ok(())
}

/// Deletes a git branch.
pub fn delete_branch(project_path: &Path, branch: &str) -> Result<()> {
    let status = ProcessCommand::new("git")
        .arg("-C")
        .arg(project_path.join("main"))
        .arg("branch")
        .arg("-D")
        .arg(branch)
        .status()
        .context("Failed to execute git branch -D")?;

    if !status.success() {
        return Err(anyhow!("git branch -D failed."));
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

/// Fetches the latest changes from the specified remote.
pub fn fetch(project_path: &Path, remote: &str) -> Result<()> {
    let status = ProcessCommand::new("git")
        .arg("-C")
        .arg(project_path.join("main"))
        .arg("fetch")
        .arg(remote)
        .status()
        .context(format!("Failed to execute git fetch {}", remote))?;

    if !status.success() {
        return Err(anyhow!("git fetch {} failed.", remote));
    }
    Ok(())
}

/// Rebases the current branch of the worktree at `worktree_path` onto `base_branch`.
pub fn rebase(worktree_path: &Path, base_branch: &str) -> Result<()> {
    let status = ProcessCommand::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("rebase")
        .arg(base_branch)
        .status()
        .context(format!("Failed to execute git rebase {}", base_branch))?;

    if !status.success() {
        return Err(anyhow!("git rebase {} failed in {}.", base_branch, worktree_path.display()));
    }
    Ok(())
}

/// Returns the current branch name of the repository at `repo_path`.
pub fn get_current_branch(repo_path: &Path) -> Result<String> {
    let repo = git2::Repository::open(repo_path)
        .context(format!("Failed to open repository at {}", repo_path.display()))?;
    let head = repo.head().context("Failed to get HEAD")?;
    let branch = head.shorthand()
        .ok_or_else(|| anyhow!("HEAD is not a branch"))?;
    Ok(branch.to_string())
}
