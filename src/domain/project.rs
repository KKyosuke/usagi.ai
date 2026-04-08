use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// Core entity representing the state of an initialized usagi project.
#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectState {
    pub initialized: bool,
    pub worktrees: Vec<String>,
    pub current_worktree: Option<String>,
    #[serde(default)]
    pub history: Vec<String>,
    #[serde(default)]
    pub last_updated: Option<String>,
}

impl ProjectState {
    pub fn update_last_updated(&mut self) {
        self.last_updated = Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string());
    }
}

/// Core entity representing the list of registered usagi repositories.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Repositories {
    pub repositories: Vec<PathBuf>,
}
