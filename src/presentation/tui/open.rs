use anyhow::{Result, Context};
use std::path::PathBuf;
use console::{Term, Key, style};
use crate::infrastructure::global_registry;
use crate::presentation::tui::{layout, screen::AlternateScreenGuard};

/// Runs the top-level workspace-selector TUI.
///
/// Returns the selected project path and an optional worktree name,
/// or `None` if the user quit without selecting.
pub fn run_terminal_ui() -> Result<Option<(PathBuf, Option<String>)>> {
    let mut repos = global_registry::get_repositories()?;
    let mut selected_index = 0;
    let term = Term::stdout();
    let mut _guard = AlternateScreenGuard::new(term.clone())?;

    loop {
        term.move_cursor_to(0, 0)?;
        term.clear_screen()?;
        layout::show_rabbit(&term);

        let menu_items = vec![
            layout::MenuItem { icon: "".to_string(), label: "Open".to_string(), key: "o".to_string() },
            layout::MenuItem { icon: "".to_string(), label: "New".to_string(), key: "e".to_string() },
            layout::MenuItem { icon: "".to_string(), label: "Config".to_string(), key: "c".to_string() },
            layout::MenuItem { icon: "".to_string(), label: "Quit".to_string(), key: "q".to_string() },
        ];

        layout::render_side_menu(&term, &menu_items, selected_index);
        layout::render_footer(&term);

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
            Key::Enter => {
                if selected_index == 0 {
                    if let Some(selected_path) = show_project_list_modal(&term, &repos)? {
                        _guard.dismiss();
                        drop(_guard);
                        return Ok(Some((selected_path, None)));
                    }
                } else if selected_index == 3 {
                    drop(_guard);
                    println!("Quit.");
                    return Ok(None);
                }
            }
            Key::Char('q') | Key::Escape | Key::CtrlC => {
                drop(_guard);
                println!("Quit.");
                return Ok(None);
            }
            _ => {}
        }
    }
}

/// Displays a modal that lets the user pick from the registered projects.
///
/// Paths that no longer exist on disk are offered for removal from the list.
pub fn show_project_list_modal(
    term: &Term,
    repos: &[PathBuf],
) -> Result<Option<PathBuf>> {
    let mut selected_index = 0;
    if repos.is_empty() {
        return Ok(None);
    }

    loop {
        term.clear_screen()?;
        let (_, width) = term.size();
        let width = width as usize;

        let title = "--- Select Project ---";
        let title_len = title.chars().count();
        let left_padding = if width > title_len { (width - title_len) / 2 } else { 0 };
        term.write_line(&format!(
            "{}{}",
            " ".repeat(left_padding),
            style(title).bold().yellow()
        ))?;
        term.write_line("")?;

        for (i, repo) in repos.iter().enumerate() {
            let label = repo.display().to_string();
            let label_len = label.chars().count() + 2;
            let left_padding = if width > label_len { (width - label_len) / 2 } else { 0 };

            if i == selected_index {
                term.write_line(&format!(
                    "{}> {}",
                    " ".repeat(left_padding),
                    style(label).cyan().bold()
                ))?;
            } else {
                term.write_line(&format!("{}  {}", " ".repeat(left_padding), label))?;
            }
        }

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
                return Ok(Some(repos[selected_index].clone()));
            }
            Key::Escape | Key::Char('q') | Key::CtrlC => {
                return Ok(None);
            }
            _ => {}
        }
    }
}
