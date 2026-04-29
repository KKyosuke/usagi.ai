use anyhow::Result;
use crate::presentation::tui::home::{self, HomeAction};
use crate::presentation::tui::action::{Action, ActionResult};
use crate::presentation::tui::action::open::OpenAction;
use crate::presentation::tui::action::new::NewAction;
use crate::presentation::tui::action::config::ConfigAction;
use crate::presentation::tui::action::quit::QuitAction;

pub async fn run() -> Result<()> {
    loop {
        let action: Box<dyn Action> = match home::run()? {
            Some(HomeAction::Open) => Box::new(OpenAction),
            Some(HomeAction::New) => Box::new(NewAction),
            Some(HomeAction::Config) => Box::new(ConfigAction),
            Some(HomeAction::Quit) | None => Box::new(QuitAction),
        };

        if let ActionResult::Quit = action.execute().await? {
            break;
        }
    }
    
    Ok(())
}
