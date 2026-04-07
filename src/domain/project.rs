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
}

/// Core entity representing the list of registered usagi repositories.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Repositories {
    pub repositories: Vec<PathBuf>,
}
