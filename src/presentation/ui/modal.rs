use anyhow::Result;
use console::{style, Term, measure_text_width};

pub struct SelectionModal<'a> {
    pub title: &'a str,
    pub items: &'a [String],
    pub selected_index: usize,
}

impl<'a> SelectionModal<'a> {
    pub fn new(title: &'a str, items: &'a [String], selected_index: usize) -> Self {
        Self {
            title,
            items,
            selected_index,
        }
    }

    pub fn render(&self, term: &Term, height: usize, width: usize, x_offset: usize) -> Result<()> {
        let popup_x = x_offset + 4;
        let popup_width = width.saturating_sub(popup_x).saturating_sub(2);

        if self.items.is_empty() {
            return Ok(());
        }

        let display_count = self.items.len().min(10);
        let box_height = display_count + 2; 
        let mut offset = 4 + box_height;

        // 上枠
        term.move_cursor_to(popup_x, height.saturating_sub(offset))?;
        term.write_str(&style(format!("┌{:─<width$}┐", "", width = popup_width)).cyan().to_string())?;
        offset -= 1;
        
        // タイトル
        term.move_cursor_to(popup_x, height.saturating_sub(offset))?;
        let title_with_padding = format!(" {} ", self.title);
        let title_len = measure_text_width(&title_with_padding);
        let left_pad = popup_width.saturating_sub(title_len) / 2;
        let right_pad = popup_width.saturating_sub(left_pad).saturating_sub(title_len);
        let title_line = format!("│{:space_width$}{}{:<padding$}│", "", title_with_padding, "", space_width = left_pad, padding = right_pad);
        term.write_str(&style(title_line).cyan().bold().to_string())?;
        offset -= 1;

        // 区切り線
        term.move_cursor_to(popup_x, height.saturating_sub(offset))?;
        term.write_str(&style(format!("├{:─<width$}┤", "", width = popup_width)).cyan().to_string())?;
        offset -= 1;

        for (idx, name) in self.items.iter().take(display_count).enumerate() {
            term.move_cursor_to(popup_x, height.saturating_sub(offset))?;
            let prefix = if idx == self.selected_index { "> " } else { "  " };
            let content = format!("│ {}{:<width$}│", prefix, name, width = popup_width.saturating_sub(3));
            
            if idx == self.selected_index {
                term.write_str(&style(content).black().on_cyan().to_string())?;
            } else {
                term.write_str(&style(content).cyan().to_string())?;
            }
            offset -= 1;
        }

        // 下枠
        term.move_cursor_to(popup_x, height.saturating_sub(offset))?;
        term.write_str(&style(format!("└{:─<width$}┘", "", width = popup_width)).cyan().to_string())?;

        Ok(())
    }
}
