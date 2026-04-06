use console::{Term, style};
use anyhow::Result;

pub struct AlternateScreenGuard {
    pub term: Term,
}

impl AlternateScreenGuard {
    pub fn new(term: Term) -> Result<Self> {
        term.write_str("\x1b[?1049h")?;
        Ok(Self { term })
    }
}

impl Drop for AlternateScreenGuard {
    fn drop(&mut self) {
        let _ = self.term.write_str("\x1b[?1049l");
        let _ = self.term.show_cursor();
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum AppMode {
    Global,
    SideMenu,
    Command,
    Execution,
}

impl AppMode {
    pub fn label(&self) -> &str {
        match self {
            AppMode::Global => "全体モード",
            AppMode::SideMenu => "サイドメニューモード",
            AppMode::Command => "コマンドモード",
            AppMode::Execution => "実行モード",
        }
    }
}

pub fn show_rabbit(term: &Term) {
    let rabbit_lines = [
        "  (\\(\\ ",
        " (='-') ",
        " o(_(\")(\")",
    ];

    let (height, width) = term.size();
    let width = width as usize;
    let height = height as usize;

    let rabbit_height = rabbit_lines.len();
    // ターミナルの高さの半分より少し上くらいに配置（下部にメニューなどがあるため）
    let top_padding = if height > rabbit_height + 5 { (height - rabbit_height) / 4 } else { 1 };

    for _ in 0..top_padding {
        let _ = term.write_line("");
    }

    for line in rabbit_lines {
        let line_len = line.chars().count();
        let left_padding = if width > line_len { (width - line_len) / 2 } else { 0 };
        let padded_line = format!("{}{}", " ".repeat(left_padding), line);
        let _ = term.write_line(&style(padded_line).magenta().bold().to_string());
    }

    let _ = term.write_line("");
    let footer = "---------- USAGI AI ----------";
    let footer_len = footer.chars().count();
    let footer_padding = if width > footer_len { (width - footer_len) / 2 } else { 0 };
    let _ = term.write_line(&format!("{}{}", " ".repeat(footer_padding), footer));
}

pub fn render_side_menu(
    term: &Term,
    projects: &[String],
    selected_project: usize,
) {
    let _ = term.write_line("Use Up/Down to select, Enter to open, 'q' to quit.");
    let _ = term.write_line(&style("PROJECTS").bold().to_string());
    let _ = term.write_line(&format!("{:-<60}", ""));

    for i in 0..projects.len() {
        if i == selected_project {
            let _ = term.write_line(&format!("> {}", style(&projects[i]).cyan().bold()));
        } else {
            let _ = term.write_line(&format!("  {}", &projects[i]));
        }
    }
}
