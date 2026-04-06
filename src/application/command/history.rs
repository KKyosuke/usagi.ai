
use anyhow::Result;
use std::path::Path;
use crate::application::init::get_project_state;

pub fn run(_args: Vec<String>, project_path: &Path) -> Result<String> {
    let state = get_project_state(project_path)?;
    let mut output = String::new();
    
    if state.history.is_empty() {
        output.push_str("No history found.");
    } else {
        for (i, entry) in state.history.iter().enumerate() {
            output.push_str(&format!("{:4} {}\n", i + 1, entry));
        }
    }
    
    Ok(output)
}
