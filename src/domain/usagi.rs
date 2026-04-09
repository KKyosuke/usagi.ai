use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// Core entity representing the list of registered usagi repositories.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Repositories {
    pub repositories: Vec<PathBuf>,
}
