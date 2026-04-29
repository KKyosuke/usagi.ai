use anyhow::Result;
use crate::presentation::tui::action::{Action, ActionResult};
use crate::presentation::tui::project;
use crate::infrastructure::global_registry;
use crate::presentation::cli;

pub struct OpenAction;

#[async_trait::async_trait]
impl Action for OpenAction {
    async fn execute(&self) -> Result<ActionResult> {
        let repos = global_registry::get_repositories()?;
        if let Some(project_path) = project::run(&repos)? {
            // Transition to Hop TUI
            cli::hop::run(project_path, None).await?;
        }
        // If they press Escape in project_select, it simply returns None and the loop goes back to Home.
        Ok(ActionResult::Continue)
    }
}
