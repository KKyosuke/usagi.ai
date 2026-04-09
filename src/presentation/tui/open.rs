use anyhow::Result;
use std::path::PathBuf;
use console::{Term, Key, style};
use crate::infrastructure::global_registry;
use crate::presentation::tui::{layout, screen::AlternateScreenGuard};

/// Runs the top-level workspace-selector TUI.
///
/// Returns the selected project path and an optional worktree name,
/// or `None` if the user quit without selecting.
pub fn run_terminal_ui() -> Result<Option<(PathBuf, Option<String>)>> {
    let repos = global_registry::get_repositories()?;
    let mut selected_index = 0;
    let term = Term::stdout();
    let mut _guard = AlternateScreenGuard::new(term.clone())?;

    loop {
        term.move_cursor_to(0, 0)?;
        term.clear_screen()?;
        layout::show_rabbit(&term);

        let menu_items = vec![
            layout::MenuItem { icon: "".to_string(), label: "Open".to_string(), key: "o".to_string(), modified_at: None },
            layout::MenuItem { icon: "".to_string(), label: "New".to_string(), key: "e".to_string(), modified_at: None },
            layout::MenuItem { icon: "".to_string(), label: "Config".to_string(), key: "c".to_string(), modified_at: None },
            layout::MenuItem { icon: "".to_string(), label: "Quit".to_string(), key: "q".to_string(), modified_at: None },
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
            Key::Char('o') => {
                selected_index = 0;
                if let Some(selected_path) = show_project_list_modal(&term, &repos)? {
                    _guard.dismiss();
                    drop(_guard);
                    return Ok(Some((selected_path, None)));
                }
            }
            Key::Char('e') => {
                selected_index = 1;
                term.clear_screen()?;
                term.write_line("New feature is not implemented yet. Please use 'usagi init'.")?;
                term.write_line("Press any key to continue...")?;
                term.read_key()?;
            }
            Key::Char('c') => {
                selected_index = 2;
                term.clear_screen()?;
                term.write_line("Config feature is not implemented yet.")?;
                term.write_line("Press any key to continue...")?;
                term.read_key()?;
            }
            Key::Enter => {
                if selected_index == 0 {
                    if let Some(selected_path) = show_project_list_modal(&term, &repos)? {
                        _guard.dismiss();
                        drop(_guard);
                        return Ok(Some((selected_path, None)));
                    }
                } else if selected_index == 1 {
                    term.clear_screen()?;
                    term.write_line("New feature is not implemented yet. Please use 'usagi init'.")?;
                    term.write_line("Press any key to continue...")?;
                    term.read_key()?;
                } else if selected_index == 2 {
                    term.clear_screen()?;
                    term.write_line("Config feature is not implemented yet.")?;
                    term.write_line("Press any key to continue...")?;
                    term.read_key()?;
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
        term.clear_screen()?;
        term.write_line("No registered projects found.")?;
        term.write_line("Please run 'usagi init <URL>' to register a project.")?;
        term.write_line("")?;
        term.write_line("Press any key to return to menu...")?;
        term.read_key()?;
        return Ok(None);
    }

    loop {
        term.clear_screen()?;
        let (_, width) = term.size();
        let width = if width == 0 { 80 } else { width as usize };

        let mut max_repo_width = 0;
        for repo in repos {
            let label = repo.display().to_string();
            let mut current_width = label.chars().count() + 2; // "> " or "  "
            if let Ok(state) = crate::infrastructure::project_state::get_project_state(repo) {
                if let Some(time) = state.last_updated {
                    let formatted_time = layout::format_modified_at(&time);
                    let time_width = formatted_time.chars().count() + 3;
                    if time_width > current_width {
                        current_width = time_width;
                    }
                }
            }
            if current_width > max_repo_width {
                max_repo_width = current_width;
            }
        }
        let left_padding = if width > max_repo_width { (width - max_repo_width) / 2 } else { 0 };

        let title = "--- Select Project ---";
        let title_len = title.chars().count();
        let title_padding = if width > title_len { (width - title_len) / 2 } else { 0 };
        term.write_line(&format!(
            "{}{}",
            " ".repeat(title_padding),
            style(title).bold().yellow()
        ))?;
        term.write_line("")?;

        for (i, repo) in repos.iter().enumerate() {
            let mut last_updated = None;
            if let Ok(state) = crate::infrastructure::project_state::get_project_state(repo) {
                last_updated = state.last_updated;
            }

            let label = repo.display().to_string();

            if i == selected_index {
                term.write_line(&format!(
                    "{}> {}",
                    " ".repeat(left_padding),
                    style(label).cyan().bold()
                ))?;
            } else {
                term.write_line(&format!("{}  {}", " ".repeat(left_padding), label))?;
            }

            if let Some(time) = last_updated {
                let formatted_time = layout::format_modified_at(&time);

                term.write_line(&format! (
                    "{}   {}",
                    " ".repeat(left_padding),
                    style(formatted_time).dim()
                ))?;
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
