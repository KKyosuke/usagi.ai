use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use crate::domain::project::ProjectHistory;

/// Reads the project history from `<project_path>/.usagi/history.json`.
pub fn get_project_history(project_path: &Path) -> Result<ProjectHistory> {
    let history_path = project_path.join(".usagi/history.json");
    if !history_path.exists() {
        return Ok(ProjectHistory::default());
    }

    let history_json = fs::read_to_string(&history_path)
        .context("Failed to read project history")?;
    let history: ProjectHistory = serde_json::from_str(&history_json)
        .context("Failed to parse project history")?;
    Ok(history)
}

/// Persists the project history to `<project_path>/.usagi/history.json`.
pub fn save_project_history(project_path: &Path, history: &ProjectHistory) -> Result<()> {
    let history_path = project_path.join(".usagi/history.json");
    let content = serde_json::to_string_pretty(history)
        .context("Failed to serialize project history")?;
    fs::write(&history_path, content).context("Failed to write project history")?;
    Ok(())
}
