use serde::{Serialize, Deserialize};

/// Project configuration defined in `usagi.config`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectConfig {
    pub repository_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Worktree {
    pub branch: String,
    pub directory: String,
    pub default: bool,
    pub modified_at: String,
}

impl Worktree {
    pub fn update_modified_at(&mut self) {
        self.modified_at = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    }
}

/// Core entity representing the state of an initialized usagi project.
#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectState {
    pub initialized: bool,
    pub worktrees: Vec<Worktree>,
    pub current_worktree: Option<String>,
    #[serde(default)]
    pub history: Vec<String>,
    #[serde(default)]
    pub last_updated: Option<String>,
}

impl ProjectState {
    pub fn update_last_updated(&mut self) {
        self.last_updated = Some(chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_state_serialization() {
        let mut state = ProjectState {
            initialized: true,
            worktrees: vec![Worktree {
                branch: "main".to_string(),
                directory: "main".to_string(),
                default: true,
                modified_at: "".to_string(),
            }],
            current_worktree: Some("main".to_string()),
            history: vec!["test command".to_string()],
            last_updated: None,
        };
        state.update_last_updated();
        state.worktrees[0].update_modified_at();

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ProjectState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.initialized, true);
        assert_eq!(deserialized.worktrees.len(), 1);
        assert_eq!(deserialized.worktrees[0].branch, "main");
        assert_eq!(deserialized.worktrees[0].default, true);
        assert_eq!(deserialized.current_worktree, Some("main".to_string()));
    }
}

