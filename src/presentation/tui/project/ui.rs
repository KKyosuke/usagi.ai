use console::{Term, style};
use std::path::PathBuf;
use crate::presentation::tui::utils::format_modified_at;

pub fn render(term: &Term, repos: &[PathBuf], selected_index: usize) -> anyhow::Result<()> {
    term.clear_screen()?;
    let (_, width) = term.size();
    let width = if width == 0 { 80 } else { width as usize };

    let mut max_repo_width = 0;
    for repo in repos {
        let label = repo.display().to_string();
        let mut current_width = label.chars().count() + 2; // "> " or "  "
        if let Ok(state) = crate::infrastructure::project_state::get_project_state(repo) {
            if let Some(time) = state.last_updated {
                let formatted_time = format_modified_at(&time);
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
            let formatted_time = format_modified_at(&time);

            term.write_line(&format! (
                "{}   {}",
                " ".repeat(left_padding),
                style(formatted_time).dim()
            ))?;
        }
    }

    Ok(())
}
