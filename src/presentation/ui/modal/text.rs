use anyhow::Result;
use console::{style, Term};
use crate::presentation::ui::modal::{draw_box_top, draw_box_title, draw_box_divider, draw_box_bottom};

pub struct InputModal<'a> {
    pub title: &'a str,
    pub input: &'a str,
    pub cursor_pos: usize,
}

impl<'a> InputModal<'a> {
    pub fn new(title: &'a str, input: &'a str, cursor_pos: usize) -> Self {
        Self {
            title,
            input,
            cursor_pos,
        }
    }

    pub fn render(&self, term: &Term, height: usize, width: usize, x_offset: usize) -> Result<()> {
        let popup_x = x_offset + 4;
        let popup_width = width.saturating_sub(popup_x).saturating_sub(2);

        // タイトル行 + 区切り線 + 入力行
        let box_height = 3; 
        let mut offset = 4 + box_height;

        // 上枠
        draw_box_top(term, popup_x, height.saturating_sub(offset), popup_width)?;
        offset -= 1;
        
        // タイトル
        draw_box_title(term, popup_x, height.saturating_sub(offset), popup_width, self.title)?;
        offset -= 1;

        // 区切り線
        draw_box_divider(term, popup_x, height.saturating_sub(offset), popup_width)?;
        offset -= 1;

        // 入力フィールド
        term.move_cursor_to(popup_x, height.saturating_sub(offset))?;
        
        let max_input_width = popup_width.saturating_sub(4);
        let display_input = if self.input.len() > max_input_width {
             let start = self.input.len().saturating_sub(max_input_width);
             &self.input[start..]
        } else {
            self.input
        };

        let content = format!("│  {:<width$} │", display_input, width = max_input_width);
        term.write_str(&style(content).cyan().to_string())?;
        
        offset -= 1;

        // 下枠
        draw_box_bottom(term, popup_x, height.saturating_sub(offset), popup_width)?;

        // カーソルを実際の入力位置に移動させる（呼び出し側で制御しやすいように、描画の最後にカーソルを合わせる）
        let cursor_x = popup_x + 3 + self.cursor_pos.min(max_input_width);
        let cursor_y = height.saturating_sub(offset + 1); // 入力行のY座標
        term.move_cursor_to(cursor_x, cursor_y)?;
        term.show_cursor()?;
        term.flush()?;

        Ok(())
    }
}
