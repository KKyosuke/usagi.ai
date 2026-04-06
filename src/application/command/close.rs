use anyhow::Result;

pub fn run(_args: Vec<String>, _project_path: &std::path::Path) -> Result<String> {
    Ok("close".to_string())
}
