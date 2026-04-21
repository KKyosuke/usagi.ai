pub mod ui;

use anyhow::Result;
use std::path::PathBuf;
use console::{Term, Key};
use crate::presentation::tui::screen::AlternateScreenGuard;

/// Displays a modal that lets the user pick from the registered projects.
pub fn run(repos: &[PathBuf]) -> Result<Option<PathBuf>> {
    let term = Term::stdout();
    let mut _guard = AlternateScreenGuard::new(term.clone())?;
    
    let mut selected_index = 0;
    if repos.is_empty() {
        term.clear_screen()?;
        term.write_line("No registered projects found.")?;
        term.write_line("Please run 'usagi init <URL>' to register a project.")?;
        term.write_line("")?;
        term.write_line("Press any key to return to menu...")?;
        term.read_key()?;
        return Ok(None);
    }

    loop {
        ui::render(&term, repos, selected_index)?;

        let key = match term.read_key() {
            Ok(k) => k,
            Err(e) => {
                if e.to_string().contains("read interrupted") {
                    return Ok(None);
                }
                return Err(anyhow::Error::from(e).context("Failed to read key"));
            }
        };
        
        match key {
            Key::ArrowUp => {
                if selected_index > 0 { selected_index -= 1; }
                else { selected_index = repos.len() - 1; }
            }
            Key::ArrowDown => {
                if selected_index < repos.len() - 1 { selected_index += 1; }
                else { selected_index = 0; }
            }
            Key::Enter => {
                _guard.dismiss();
                drop(_guard);
                return Ok(Some(repos[selected_index].clone()));
            }
            Key::Escape | Key::Char('q') | Key::CtrlC => {
                return Ok(None);
            }
            _ => {}
        }
    }
}
