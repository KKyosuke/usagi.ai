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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project::Worktree;
    use std::fs;

    #[test]
    fn test_save_and_get_project_state() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("usagi_test_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        let usagi_dir = temp_dir.join(".usagi");
        fs::create_dir_all(&usagi_dir)?;
        let worktree_path = std::path::PathBuf::from("main");

        let state = ProjectState {
            initialized: true,
            worktrees: vec![Worktree {
                branch: "main".to_string(),
                directory: worktree_path.to_string_lossy().to_string(),
                default: true,
                modified_at: chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
                status: crate::domain::project::SessionStatus::Todo,
            }],
            current_worktree: Some(worktree_path.to_string_lossy().to_string()),
            last_updated: None,
            ai_model: None,
        };

        save_project_state(&temp_dir, &state)?;
        let loaded_state = get_project_state(&temp_dir)?;

        assert_eq!(loaded_state.initialized, true);
        assert_eq!(loaded_state.worktrees.len(), 1);
        assert_eq!(loaded_state.worktrees[0].branch, "main");

        // Clean up
        fs::remove_dir_all(&temp_dir)?;
        Ok(())
    }

    #[test]
    fn test_get_project_state_missing() {
        let temp_dir = std::env::temp_dir().join(format!("usagi_test_missing_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        let result = get_project_state(&temp_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Project state is missing"));
    }
}
