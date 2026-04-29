use anyhow::Result;

pub mod open;
pub mod new;
pub mod config;
pub mod quit;

pub enum ActionResult {
    Continue,
    Quit,
}

#[async_trait::async_trait]
pub trait Action {
    async fn execute(&self) -> Result<ActionResult>;
}
