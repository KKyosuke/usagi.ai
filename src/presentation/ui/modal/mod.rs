use anyhow::Result;
use console::{style, Term, measure_text_width};

pub mod select;
pub mod text;

pub use select::SelectionModal;
pub use text::InputModal;

pub(crate) fn draw_box_top(term: &Term, x: usize, y: usize, width: usize) -> Result<()> {
    term.move_cursor_to(x, y)?;
    term.write_str(&style(format!("┌{:─<width$}┐", "", width = width)).cyan().to_string())?;
    Ok(())
}

pub(crate) fn draw_box_title(term: &Term, x: usize, y: usize, width: usize, title: &str) -> Result<()> {
    term.move_cursor_to(x, y)?;
    let title_with_padding = format!(" {} ", title);
    let title_len = measure_text_width(&title_with_padding);
    let left_pad = width.saturating_sub(title_len) / 2;
    let right_pad = width.saturating_sub(left_pad).saturating_sub(title_len);
    let title_line = format!("│{:space_width$}{}{:<padding$}│", "", title_with_padding, "", space_width = left_pad, padding = right_pad);
    term.write_str(&style(title_line).cyan().bold().to_string())?;
    Ok(())
}

pub(crate) fn draw_box_divider(term: &Term, x: usize, y: usize, width: usize) -> Result<()> {
    term.move_cursor_to(x, y)?;
    term.write_str(&style(format!("├{:─<width$}┤", "", width = width)).cyan().to_string())?;
    Ok(())
}

pub(crate) fn draw_box_bottom(term: &Term, x: usize, y: usize, width: usize) -> Result<()> {
    term.move_cursor_to(x, y)?;
    term.write_str(&style(format!("└{:─<width$}┘", "", width = width)).cyan().to_string())?;
    Ok(())
}
