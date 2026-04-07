use anyhow::{Result, Context, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use crate::domain::project::{ProjectState, Repositories};

/// Returns the path to the usagi application data directory.
fn data_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "", "usagi")
        .ok_or_else(|| anyhow!("Could not determine home directory"))?;
    Ok(proj_dirs.data_dir().to_path_buf())
}

/// Reads the list of registered repositories.
pub fn get_repositories() -> Result<Vec<PathBuf>> {
    let repo_json_path = data_dir()?.join("repositories.json");

    if repo_json_path.exists() {
        let content = fs::read_to_string(&repo_json_path)
            .context("Failed to read repositories.json")?;
        let repos: Repositories = serde_json::from_str(&content)
            .context("Failed to parse repositories.json")?;
        Ok(repos.repositories)
    } else {
        Ok(vec![])
    }
}

/// Persists the list of registered repositories.
pub fn save_repositories(repos: &[PathBuf]) -> Result<()> {
    let dir = data_dir()?;
    fs::create_dir_all(&dir).context("Failed to create data directory")?;

    let repo_json_path = dir.join("repositories.json");
    let repos_struct = Repositories {
        repositories: repos.to_vec(),
    };
    let content = serde_json::to_string_pretty(&repos_struct)
        .context("Failed to serialize repositories")?;
    fs::write(repo_json_path, content).context("Failed to write repositories.json")?;

    Ok(())
}

/// Adds the current directory to the repository registry if not already present.
pub fn register_project() -> Result<()> {
    let dir = data_dir()?;
    fs::create_dir_all(&dir).context("Failed to create data directory")?;

    let repo_json_path = dir.join("repositories.json");
    let mut repos = if repo_json_path.exists() {
        let content = fs::read_to_string(&repo_json_path)
            .context("Failed to read repositories.json")?;
        serde_json::from_str::<Repositories>(&content)
            .context("Failed to parse repositories.json")?
    } else {
        Repositories::default()
    };

    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    if !repos.repositories.contains(&current_dir) {
        repos.repositories.push(current_dir);
        let content = serde_json::to_string_pretty(&repos)
            .context("Failed to serialize repositories")?;
        fs::write(repo_json_path, content)
            .context("Failed to write repositories.json")?;
    }

    Ok(())
}

/// Reads the project state from `<project_path>/.usagi/state.json`.
pub fn get_project_state(project_path: &Path) -> Result<ProjectState> {
    let state_path = project_path.join(".usagi/state.json");
    if !state_path.exists() {
        return Err(anyhow!(
            "Project state is missing in {}. Please ensure it's a valid usagi project.",
            project_path.display()
        ));
    }

    let state_json = fs::read_to_string(state_path)
        .context("Failed to read project state")?;
    let state: ProjectState = serde_json::from_str(&state_json)
        .context("Failed to parse project state")?;
    Ok(state)
}

/// Persists the project state to `<project_path>/.usagi/state.json`.
pub fn save_project_state(project_path: &Path, state: &ProjectState) -> Result<()> {
    let state_path = project_path.join(".usagi/state.json");
    let content = serde_json::to_string_pretty(state)
        .context("Failed to serialize project state")?;
    fs::write(state_path, content).context("Failed to write project state")?;
    Ok(())
}
