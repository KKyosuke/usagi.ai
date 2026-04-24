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
    track: bool,
) -> Result<()> {
    let mut command = ProcessCommand::new("git");
    command
        .arg("-C")
        .arg(project_path.join("main"))
        .arg("worktree")
        .arg("add");

    if !track {
        command.arg("--no-track");
    }

    let status = command
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
    let repo = git2::Repository::open_ext(
        repo_path,
        git2::RepositoryOpenFlags::NO_SEARCH,
        &[] as &[&std::ffi::OsStr],
    ).context(format!("Failed to open repository at {}", repo_path.display()))?;
    let head = repo.head().context("Failed to get HEAD")?;
    let branch = head.shorthand()
        .ok_or_else(|| anyhow!("HEAD is not a branch"))?;
    Ok(branch.to_string())
}

/// Returns `true` if the branch at `repo_path` has an upstream branch.
pub fn has_upstream(repo_path: &Path) -> Result<bool> {
    let repo = match git2::Repository::open_ext(
        repo_path,
        git2::RepositoryOpenFlags::NO_SEARCH,
        &[] as &[&std::ffi::OsStr],
    ) {
        Ok(r) => r,
        Err(_) => return Ok(false),
    };

    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return Ok(false),
    };

    if !head.is_branch() {
        return Ok(false);
    }

    let branch = git2::Branch::wrap(head);
    let has_upstream = branch.upstream().is_ok();
    Ok(has_upstream)
}

/// Returns a list of remote branches.
pub fn list_remote_branches(project_path: &Path) -> Result<Vec<String>> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(project_path.join("main"))
        .arg("branch")
        .arg("-r")
        .output()
        .context("Failed to execute git branch -r")?;

    if !output.status.success() {
        return Err(anyhow!("git branch -r failed."));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches = stdout
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.contains("->"))
        .map(|line| line.to_string())
        .collect();

    Ok(branches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_has_upstream_no_search_upwards() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("usagi_git_test_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        fs::create_dir_all(&temp_dir)?;

        // Create parent repo with upstream
        let parent_repo = temp_dir.join("parent");
        fs::create_dir_all(&parent_repo)?;
        std::process::Command::new("git").arg("init").arg(&parent_repo).output()?;
        std::process::Command::new("git").arg("-C").arg(&parent_repo).args(&["config", "user.email", "test@example.com"]).output()?;
        std::process::Command::new("git").arg("-C").arg(&parent_repo).args(&["config", "user.name", "Test"]).output()?;
        std::process::Command::new("git").arg("-C").arg(&parent_repo).args(&["commit", "--allow-empty", "-m", "init"]).output()?;
        
        let current_branch = get_current_branch(&parent_repo)?;
        
        std::process::Command::new("git").arg("-C").arg(&parent_repo).args(&["remote", "add", "origin", "https://github.com/example/repo.git"]).output()?;
        std::process::Command::new("git").arg("-C").arg(&parent_repo).args(&["config", &format!("branch.{}.remote", current_branch), "origin"]).output()?;
        std::process::Command::new("git").arg("-C").arg(&parent_repo).args(&["config", &format!("branch.{}.merge", current_branch), "refs/heads/main"]).output()?;
        
        // Create the remote reference
        std::process::Command::new("git").arg("-C").arg(&parent_repo).args(&["update-ref", "refs/remotes/origin/main", &current_branch]).output()?;

        // Verify parent has upstream
        assert!(has_upstream(&parent_repo)?);

        // Create child directory (not a git repo)
        let child_dir = parent_repo.join("child");
        fs::create_dir_all(&child_dir)?;

        // has_upstream should NOT search upwards and should return false
        assert!(!has_upstream(&child_dir)?);

        // Clean up
        fs::remove_dir_all(&temp_dir)?;
        Ok(())
    }

    #[test]
    fn test_create_worktree_tracking() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("usagi_git_wt_test_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        fs::create_dir_all(&temp_dir)?;

        // Create remote repo
        let remote_repo = temp_dir.join("remote");
        fs::create_dir_all(&remote_repo)?;
        std::process::Command::new("git").arg("init").arg(&remote_repo).output()?;
        std::process::Command::new("git").arg("-C").arg(&remote_repo).args(&["config", "user.email", "test@example.com"]).output()?;
        std::process::Command::new("git").arg("-C").arg(&remote_repo).args(&["config", "user.name", "Test"]).output()?;
        std::process::Command::new("git").arg("-C").arg(&remote_repo).args(&["commit", "--allow-empty", "-m", "init"]).output()?;

        // Create project structure: project_path/main
        let project_path = temp_dir.join("project");
        let main_repo = project_path.join("main");
        fs::create_dir_all(&main_repo)?;
        std::process::Command::new("git").arg("clone").arg(&remote_repo).arg(&main_repo).output()?;

        // Case 1: Track
        let wt_path_track = project_path.join("wt_track");
        create_worktree(&project_path, "branch_track", &wt_path_track, "origin/main", true)?;
        assert!(has_upstream(&wt_path_track)?);

        // Case 2: No Track
        let wt_path_no_track = project_path.join("wt_no_track");
        create_worktree(&project_path, "branch_no_track", &wt_path_no_track, "origin/main", false)?;
        assert!(!has_upstream(&wt_path_no_track)?);

        fs::remove_dir_all(&temp_dir)?;
        Ok(())
    }
}
