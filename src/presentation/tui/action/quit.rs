use anyhow::Result;
use crate::presentation::tui::action::{Action, ActionResult};

pub struct QuitAction;

#[async_trait::async_trait]
impl Action for QuitAction {
    async fn execute(&self) -> Result<ActionResult> {
        Ok(ActionResult::Quit)
    }
}
