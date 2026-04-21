pub mod ui;

use anyhow::Result;
use console::{Term, Key};
use crate::presentation::tui::screen::AlternateScreenGuard;

pub enum HomeAction {
    Open,
    New,
    Config,
    Quit,
}

/// Displays the home menu and waits for the user to select an action.
pub fn run() -> Result<Option<HomeAction>> {
    let mut selected_index = 0;
    let term = Term::stdout();
    let mut _guard = AlternateScreenGuard::new(term.clone())?;

    loop {
        term.move_cursor_to(0, 0)?;
        term.clear_screen()?;
        ui::show_rabbit(&term, true);

        let menu_items = vec![
            ui::MenuItem { icon: "".to_string(), label: "Open".to_string(), key: "o".to_string(), modified_at: None },
            ui::MenuItem { icon: "".to_string(), label: "New".to_string(), key: "e".to_string(), modified_at: None },
            ui::MenuItem { icon: "".to_string(), label: "Config".to_string(), key: "c".to_string(), modified_at: None },
            ui::MenuItem { icon: "".to_string(), label: "Quit".to_string(), key: "q".to_string(), modified_at: None },
        ];

        ui::render_side_menu(&term, &menu_items, selected_index);
        ui::render_footer(&term);

        let key = match term.read_key() {
            Ok(k) => k,
            Err(e) => {
                if e.to_string().contains("read interrupted") {
                    drop(_guard);
                    println!("Quit.");
                    return Ok(None);
                }
                return Err(anyhow::Error::from(e).context("Failed to read key"));
            }
        };

        match key {
            Key::ArrowUp => {
                if selected_index > 0 { selected_index -= 1; }
                else { selected_index = menu_items.len() - 1; }
            }
            Key::ArrowDown => {
                if selected_index < menu_items.len() - 1 { selected_index += 1; }
                else { selected_index = 0; }
            }
            Key::Char('o') => {
                _guard.dismiss();
                drop(_guard);
                return Ok(Some(HomeAction::Open));
            }
            Key::Char('e') => {
                _guard.dismiss();
                drop(_guard);
                return Ok(Some(HomeAction::New));
            }
            Key::Char('c') => {
                _guard.dismiss();
                drop(_guard);
                return Ok(Some(HomeAction::Config));
            }
            Key::Enter => {
                if selected_index == 0 {
                    _guard.dismiss();
                    drop(_guard);
                    return Ok(Some(HomeAction::Open));
                } else if selected_index == 1 {
                    _guard.dismiss();
                    drop(_guard);
                    return Ok(Some(HomeAction::New));
                } else if selected_index == 2 {
                    _guard.dismiss();
                    drop(_guard);
                    return Ok(Some(HomeAction::Config));
                } else if selected_index == 3 {
                    drop(_guard);
                    println!("Quit.");
                    return Ok(Some(HomeAction::Quit));
                }
            }
            Key::Char('q') | Key::Escape | Key::CtrlC => {
                drop(_guard);
                println!("Quit.");
                return Ok(Some(HomeAction::Quit));
            }
            _ => {}
        }
    }
}
