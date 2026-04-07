use console::{Term, style};
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};

static EXIT_MESSAGE_PRINTED: AtomicBool = AtomicBool::new(false);

pub struct AlternateScreenGuard {
    pub term: Term,
    pub is_active: bool,
}

impl AlternateScreenGuard {
    pub fn new(term: Term) -> Result<Self> {
        let _ = term.write_str("\x1b[?1049h");
        let _ = term.hide_cursor();

        // 常にフラグをリセットして、新しいガードがメッセージを出せるようにする
        EXIT_MESSAGE_PRINTED.store(false, Ordering::SeqCst);

        let t = term.clone();
        let _ = ctrlc::set_handler(move || {
            let _ = t.write_str("\x1b[?1049l");
            let _ = t.show_cursor();
            if !EXIT_MESSAGE_PRINTED.swap(true, Ordering::SeqCst) {
                let _ = t.write_line("USAGI run away ( ^-^)ノ");
            }
            std::process::exit(0);
        });

        Ok(Self { term, is_active: true })
    }

    pub fn dismiss(&mut self) {
        self.is_active = false;
    }
}

impl Drop for AlternateScreenGuard {
    fn drop(&mut self) {
        let _ = self.term.write_str("\x1b[?1049l");
        let _ = self.term.show_cursor();
        if self.is_active {
            if !EXIT_MESSAGE_PRINTED.swap(true, Ordering::SeqCst) {
                let _ = self.term.write_line("USAGI run away ( ^-^)ノ");
            }
        }
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
    let character_lines = [
        "  (\\(\\ ",
        " (='-') ",
        " o(_(\")(\")",
    ];

    let message_lines = [
        "USAGI AI",
    ];

    let (height, width) = term.size();
    let width = width as usize;
    let height = height as usize;

    let total_height = character_lines.len() + message_lines.len() + 2; // +2 for spacing
    let top_padding = if height > total_height + 10 { (height - total_height) / 4 } else { 1 };

    for _ in 0..top_padding {
        let _ = term.write_line("");
    }

    for line in character_lines {
        let line_len = line.chars().count();
        let left_padding = if width > line_len { (width - line_len) / 2 } else { 0 };
        let padded_line = format!("{}{}", " ".repeat(left_padding), line);
        let _ = term.write_line(&style(padded_line).magenta().bold().to_string());
    }

    let _ = term.write_line("");

    for line in message_lines {
        let line_len = line.chars().count();
        let left_padding = if width > line_len { (width - line_len) / 2 } else { 0 };
        let padded_line = format!("{}{}", " ".repeat(left_padding), line);
        let _ = term.write_line(&style(padded_line).green().bold().to_string());
    }
}

pub fn render_side_menu(
    term: &Term,
    items: &[MenuItem],
    selected_index: usize,
) {
    let (height, width) = term.size();
    let width = width as usize;
    
    let _ = term.write_line("");

    for (i, item) in items.iter().enumerate() {
        let prefix = if i == selected_index {
            style(&item.icon).red().bold().to_string()
        } else {
            style(&item.icon).yellow().to_string()
        };

        let label = &item.label;
        let key = &item.key;

        // メニューの各行を中央揃えにするための計算
        // アイコン(2) + スペース(1) + ラベル(10程度) + スペース(10) + キー(1)
        // ここでは固定幅のメニューを想定して中央寄せする
        let menu_width = 30; 
        let left_padding = if width > menu_width { (width - menu_width) / 2 } else { 0 };
        
        // アイコンとラベルの間にカーソル（赤い四角）を入れるNeovim風演出
        let cursor = if i == selected_index {
            style(" ").bg(console::Color::Red).to_string()
        } else {
            " ".to_string()
        };

        let line = format!("{}{} {} {:<10} {:>5}", 
            " ".repeat(left_padding),
            prefix,
            cursor,
            label,
            key
        );
        let _ = term.write_line(&line);
        let _ = term.write_line(""); // 行間の空き
    }
}

pub fn render_footer(term: &Term) {
    let version = env!("CARGO_PKG_VERSION");
    let footer = format!(" v{} ⚡ plugins 4/55 in 23.885ms", version);
    let (height, width) = term.size();
    let width = width as usize;
    let height = height as usize;

    let footer_len = footer.chars().count();
    let left_padding = if width > footer_len { (width - footer_len) / 2 } else { 0 };
    
    // 下部に配置するための調整（簡易的）
    let _ = term.write_line("");
    let _ = term.write_line(&format!("{}{}", " ".repeat(left_padding), style(footer).dim()));
}

pub struct MenuItem {
    pub icon: String,
    pub label: String,
    pub key: String,
}
