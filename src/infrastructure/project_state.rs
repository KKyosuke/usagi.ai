use anyhow::{Result, Context, anyhow};
use std::fs;
use std::path::Path;
use crate::domain::project::ProjectState;

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
