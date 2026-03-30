use anyhow::Result;
use crate::application::init;
use crate::command::hop;

pub fn run() -> Result<()> {
    if let Some((project_path, worktree)) = init::run_terminal_ui()? {
        return hop::run(project_path, worktree);
    }
    Ok(())
}
