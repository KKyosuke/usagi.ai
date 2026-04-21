use console::{Term, style};
use std::time::{Duration, Instant};
use std::thread;
use crate::presentation::tui::utils::format_modified_at;

/// A single entry in the side-menu.
pub struct MenuItem {
    pub icon: String,
    pub label: String,
    pub key: String,
    pub modified_at: Option<String>,
}

/// Renders the usagi ASCII-art mascot centred in the terminal.
pub fn show_rabbit(term: &Term, center_vertically: bool) {
    let character_lines = [
        "  (\\(\\ ",
        " (='-') ",
        " o(_(\")(\")",
    ];

    let message_lines = [
        "USAGI AI",
    ];

    let (height, width) = term.size();
    let width = if width == 0 { 80 } else { width as usize };
    let height = if height == 0 { 24 } else { height as usize };

    let total_height = character_lines.len() + message_lines.len() + 2;
    let top_padding = if center_vertically && height > total_height + 10 {
        (height - total_height) / 4
    } else {
        1
    };

    for _ in 0..top_padding {
        let _ = term.write_line("");
    }

    let max_char_width = character_lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let left_padding_rabbit = if width > max_char_width { (width - max_char_width) / 2 } else { 0 };

    for line in character_lines {
        let padded_line = format!("{}{}", " ".repeat(left_padding_rabbit), line);
        let _ = term.write_line(&style(padded_line).magenta().bold().to_string());
    }

    let _ = term.write_line("");

    let max_message_width = message_lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let left_padding_message = if width > max_message_width { (width - max_message_width) / 2 } else { 0 };

    for line in message_lines {
        let padded_line = format!("{}{}", " ".repeat(left_padding_message), line);
        let _ = term.write_line(&style(padded_line).green().bold().to_string());
    }
}

/// Renders the usagi ASCII-art mascot with a simple running animation.
pub fn animate_rabbit(term: &Term, duration_ms: u64, center_vertically: bool) {
    let frames = [
        [
            "  (\\(\\ ",
            " (='-') ",
            " o(_(\")(\")",
        ],
        [
            "  | |  ",
            " (='^') ",
            "  (_/)(_/)",
        ],
        [
            "  (\\(\\ ",
            " (='o') ",
            "  / / / /",
        ],
        [
            "  | |  ",
            " (='-') ",
            "  (_)(_)o",
        ],
    ];

    let y_offsets = [1, 0, 0, 1];

    let (height, width) = term.size();
    let width = if width == 0 { 80 } else { width as usize };
    let height = if height == 0 { 24 } else { height as usize };
    
    let total_height = 5;
    let top_padding = if center_vertically && height > total_height + 10 {
        (height - total_height) / 4
    } else {
        1
    };

    for _ in 0..top_padding {
        let _ = term.write_line("");
    }

    let start_time = Instant::now();
    let duration = Duration::from_millis(duration_ms);
    let frame_interval = Duration::from_millis(120);
    
    let mut frame_idx = 0;
    let mut x_pos = 0;
    let max_x = width.saturating_sub(15).max(1);

    while start_time.elapsed() < duration {
        let frame = &frames[frame_idx];
        let rabbit_padding = " ".repeat(x_pos);
        let y_offset = y_offsets[frame_idx];

        for _ in 0..y_offset {
            let _ = term.write_line("");
        }
        for line in frame {
            let _ = term.write_line(&style(format!("{}{}", rabbit_padding, line)).magenta().bold().to_string());
        }
        for _ in 0..(1 - y_offset) {
            let _ = term.write_line("");
        }
        let _ = term.write_line(&style(format!("{}  USAGI AI", rabbit_padding)).green().bold().to_string());
        
        thread::sleep(frame_interval);
        
        if start_time.elapsed() < duration {
            let _ = term.clear_last_lines(5);
            frame_idx = (frame_idx + 1) % frames.len();
            x_pos = (x_pos + 3) % max_x;
        } else if frame_idx != 0 {
            let _ = term.clear_last_lines(5);
            for _ in 0..y_offsets[0] {
                let _ = term.write_line("");
            }
            for line in &frames[0] {
                let _ = term.write_line(&style(format!("{}{}", rabbit_padding, line)).magenta().bold().to_string());
            }
            for _ in 0..(1 - y_offsets[0]) {
                let _ = term.write_line("");
            }
            let _ = term.write_line(&style(format!("{}  USAGI AI", rabbit_padding)).green().bold().to_string());
        }
    }
}

/// Renders the side-menu items, highlighting the selected entry.
pub fn render_side_menu(term: &Term, items: &[MenuItem], selected_index: usize) {
    let (_, width) = term.size();
    let width = if width == 0 { 80 } else { width as usize };

    let _ = term.write_line("");

    let mut max_menu_width = 0;
    for item in items {
        let icon_width = item.icon.chars().count();
        let item_width = if icon_width == 0 {
            18
        } else {
            icon_width + 19
        };
        if item_width > max_menu_width {
            max_menu_width = item_width;
        }
        if let Some(time) = &item.modified_at {
            let formatted_time = format_modified_at(time);
            let time_width = formatted_time.chars().count() + 3;
            if time_width > max_menu_width {
                max_menu_width = time_width;
            }
        }
    }

    let left_padding = if width > max_menu_width { (width - max_menu_width) / 2 } else { 0 };

    for (i, item) in items.iter().enumerate() {
        let is_selected = i == selected_index;
        let prefix = if is_selected {
            style(&item.icon).red().bold().to_string()
        } else {
            style(&item.icon).yellow().to_string()
        };

        let label = &item.label;
        let key = &item.key;

        let cursor = if is_selected {
            style(">").red().bold().to_string()
        } else {
            " ".to_string()
        };

        let label_display = if is_selected {
            style(format!("{:<10}", label)).cyan().bold().to_string()
        } else {
            format!("{:<10}", label)
        };

        let key_display = if is_selected {
            style(format!("{:>5}", key)).yellow().to_string()
        } else {
            format!("{:>5}", key)
        };

        let line = if item.icon.is_empty() {
            format!(
                "{}{} {} {}",
                " ".repeat(left_padding),
                cursor,
                label_display,
                key_display
            )
        } else {
            format!(
                "{}{} {} {} {}",
                " ".repeat(left_padding),
                prefix,
                cursor,
                label_display,
                key_display
            )
        };
        let _ = term.write_line(&line);
        if let Some(time) = &item.modified_at {
            let formatted_time = format_modified_at(time);
            let time_line = format!(
                "{}   {}",
                " ".repeat(left_padding),
                style(formatted_time).dim()
            );
            let _ = term.write_line(&time_line);
        }
        let _ = term.write_line("");
    }
}

/// Renders the status footer at the bottom of the terminal.
pub fn render_footer(term: &Term) {
    let version = env!("CARGO_PKG_VERSION");
    let footer = format!(" v{} ⚡ plugins 4/55 in 23.885ms", version);
    let (_, width) = term.size();
    let width = if width == 0 { 80 } else { width as usize };

    let footer_len = footer.chars().count();
    let left_padding = if width > footer_len { (width - footer_len) / 2 } else { 0 };

    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}{}",
        " ".repeat(left_padding),
        style(footer).dim()
    ));
}
