use anyhow::Result;
use crate::presentation::tui::open as tui_open;
use crate::presentation::cli::hop;

/// Handles the `usagi open` CLI command.
///
/// Shows the workspace-selector TUI and, if a project is chosen, opens the
/// hop terminal for that project.
pub fn run() -> Result<()> {
    if let Some((project_path, worktree)) = tui_open::run_terminal_ui()? {
        return hop::run(project_path, worktree);
    }
    Ok(())
}
