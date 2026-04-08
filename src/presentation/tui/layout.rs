use console::{Term, style};

/// A single entry in the side-menu.
pub struct MenuItem {
    pub icon: String,
    pub label: String,
    pub key: String,
    pub last_updated: Option<String>,
}

/// Renders the usagi ASCII-art mascot centred in the terminal.
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

    let total_height = character_lines.len() + message_lines.len() + 2;
    let top_padding = if height > total_height + 10 {
        (height - total_height) / 4
    } else {
        1
    };

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

/// Renders the side-menu items, highlighting the selected entry.
pub fn render_side_menu(term: &Term, items: &[MenuItem], selected_index: usize) {
    let (_, width) = term.size();
    let width = width as usize;

    let _ = term.write_line("");

    for (i, item) in items.iter().enumerate() {
        let is_selected = i == selected_index;
        let prefix = if is_selected {
            style(&item.icon).red().bold().to_string()
        } else {
            style(&item.icon).yellow().to_string()
        };

        let label = &item.label;
        let key = &item.key;

        let menu_width = 30;
        let left_padding = if width > menu_width { (width - menu_width) / 2 } else { 0 };

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

        let line = format!(
            "{}{} {} {} {}",
            " ".repeat(left_padding),
            prefix,
            cursor,
            label_display,
            key_display
        );
        let _ = term.write_line(&line);
        if let Some(time) = &item.last_updated {
            let time_line = format!(
                "{}       {}",
                " ".repeat(left_padding),
                style(format!("Last update: {}", time)).dim()
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
    let width = width as usize;

    let footer_len = footer.chars().count();
    let left_padding = if width > footer_len { (width - footer_len) / 2 } else { 0 };

    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}{}",
        " ".repeat(left_padding),
        style(footer).dim()
    ));
}
