use anyhow::Result;
use crate::presentation::tui::action::{Action, ActionResult};

pub struct ConfigAction;

#[async_trait::async_trait]
impl Action for ConfigAction {
    async fn execute(&self) -> Result<ActionResult> {
        let term = console::Term::stdout();
        term.clear_screen()?;
        term.write_line("Config feature is not implemented yet.")?;
        term.write_line("Press any key to continue...")?;
        term.read_key()?;
        
        Ok(ActionResult::Continue)
    }
}
