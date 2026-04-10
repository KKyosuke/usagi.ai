use anyhow::Result;
use console::{style, measure_text_width, strip_ansi_codes};
use crate::presentation::cli::hop::app::HopApp;
use crate::presentation::tui::layout;

pub fn render(app: &HopApp) -> Result<()> {
    let term = &app.term;
    let (height, width) = term.size();
    let left_width = 30; 
    let right_width = (width as usize).saturating_sub(left_width).saturating_sub(3); // 3 for separators
    
    // ヘッダー表示
    term.move_cursor_to(0, 0)?;
    term.clear_screen()?;
    
    term.write_line(&format!("{}", style("----- USAGI TERMINAL -----").magenta().bold()))?;
    term.write_line(&format!("MODE: {}", style(app.mode().label()).bold().cyan()))?;

    // 左右分割描画
    for i in 0..(height as usize - 6) {
        let left_content = if i == 0 {
            style("workspace").bold().to_string()
        } else {
            let wt_idx = (i - 1) / 2;
            if wt_idx < app.state.worktrees.len() {
                let wt = &app.state.worktrees[wt_idx];
                let is_second_line = (i - 1) % 2 == 1;

                if !is_second_line {
                    let mark_char = "●";
                    let mark_width = measure_text_width(mark_char);
                    let cursor = if wt_idx == app.selected_index && !app.is_command_mode { ">" } else { " " };
                    let is_selected = wt_idx == app.selected_index;
                    let mark = if is_selected {
                        style(mark_char).green().to_string()
                    } else {
                        " ".repeat(mark_width)
                    };
                    
                    if wt_idx == app.selected_index {
                        format!("{} {}  {}", cursor, mark, style(&wt.branch).cyan().bold())
                    } else {
                        format!("{} {}  {}", cursor, mark, &wt.branch)
                    }
                } else {
                    format!("   {}", style(layout::format_modified_at(&wt.modified_at)).dim())
                }
            } else {
                "".to_string()
            }
        };

        // 左側の幅を調整
        let left_padding = left_width.saturating_sub(measure_text_width(&left_content));
        let left_display = format!("{}{}", left_content, " ".repeat(left_padding));
        
        // 右側の表示内容 (履歴)
        let right_content = if i == 0 {
            let label = if app.is_terminal_view {
                style("TERMINAL").bold().cyan().to_string()
            } else {
                style("Welcome to usagi terminal!").bold().to_string()
            };
            let wt = &app.state.worktrees[app.selected_index];
            format!("{} (Dir: {})", label, style(&wt.directory).dim())
        } else {
            let history_idx = i.saturating_sub(1);
            if history_idx < app.command_history.len() {
                app.command_history[history_idx].clone()
            } else {
                "".to_string()
            }
        };
        
        let right_padding = right_width.saturating_sub(measure_text_width(&strip_ansi_codes(&right_content)));
        let right_display = format!("{}{}", right_content, " ".repeat(right_padding));

        term.write_line(&format!("{} | {}", left_display, right_display))?;
    }

    // 入力欄
    term.move_cursor_to(0, height as usize - 4)?;
    term.write_line(&format!("{:-<width$}", "", width = width as usize))?;
    let command_padding = left_width.saturating_sub(measure_text_width("COMMAND"));
    let command_prompt = format!("COMMAND{:padding$} | {}", "", app.current_input, padding = command_padding);
    let command_display = format!("{:width$}", command_prompt, width = width as usize);
    term.write_line(&command_display)?;

    // 下部ヘルプ
    term.move_cursor_to(0, (height as usize).saturating_sub(1))?;
    let help_text = if app.is_command_mode {
        style("Enter: execute, Escape: cancel, Tab: select suggestion, Type to command...").dim().to_string()
    } else {
        style("Use Up/Down to select, Enter to type command, 'q' to quit.").dim().to_string()
    };
    // 最終行でのスクロールを避けるため write_str を使用
    let help_display = format!("{:width$}", help_text, width = width as usize);
    term.write_str(&help_display)?;

    // コマンドモードのポップアップ表示
    if app.is_command_mode {
        render_command_popup(app, height as usize, width as usize, left_width)?;
    }

    if app.is_command_mode {
        let input_prefix: String = app.current_input.chars().take(app.cursor_pos).collect();
        let cursor_x = left_width + 3 + measure_text_width(&input_prefix);
        term.move_cursor_to(cursor_x, height as usize - 3)?;
        term.show_cursor()?;
    } else {
        term.hide_cursor()?;
    }

    Ok(())
}

fn render_command_popup(app: &HopApp, height: usize, width: usize, left_width: usize) -> Result<()> {
    if app.current_input.is_empty() {
        return Ok(());
    }
    let term = &app.term;
    let parts: Vec<&str> = app.current_input.split_whitespace().collect();
    let mut suggestions: Vec<(String, String)> = Vec::new();
    let mut usage_text: Option<String> = None;

    if !app.current_input.contains(' ') {
        // コマンド名のサジェスト
        let mut current_suggestions: Vec<(String, String)> = app.commands.iter()
            .filter(|c| c.name().starts_with(&app.current_input))
            .map(|c| (c.name().to_string(), c.description().to_string()))
            .collect();

        // サジェストが1つだけの場合、その詳細な使用法を表示する
        if current_suggestions.len() == 1 {
            let name = current_suggestions[0].0.clone();
            if let Some(command) = app.commands.iter().find(|c| c.name() == name) {
                usage_text = command.usage(&[name.as_str()]);
                if name == app.current_input {
                    current_suggestions.clear();
                }
            }
        }
        suggestions = current_suggestions;
    } else if !parts.is_empty() {
        // コマンドの引数/サブコマンドのサジェスト
        let cmd_name = parts[0];
        if let Some(command) = app.commands.iter().find(|c| c.name() == cmd_name) {
            let last_part = if app.current_input.ends_with(' ') { "" } else { parts.last().unwrap_or(&"") };
            let mut current_suggestions: Vec<(String, String)> = command.subcommands()
                .into_iter()
                .filter(|(name, _)| name.starts_with(last_part))
                .collect();

            // サジェストが1つだけの場合、その詳細な使用法を表示する
            if current_suggestions.len() == 1 {
                let name = current_suggestions[0].0.clone();
                let is_perfect_match = name == last_part;

                let mut check_parts = parts.clone();
                if !app.current_input.ends_with(' ') {
                    if let Some(last) = check_parts.last_mut() {
                        *last = name.as_str();
                    }
                } else {
                    // すでにそのコマンドが入力済みの場合は、そのコマンド自体のサジェストは不要
                    if parts.iter().any(|&p| p == name) {
                        current_suggestions.clear();
                    }
                    check_parts.push(name.as_str());
                }

                if let Some(detail_usage) = command.usage(&check_parts) {
                    usage_text = Some(detail_usage);
                } else {
                    usage_text = command.usage(&parts);
                }

                if is_perfect_match {
                    current_suggestions.clear();
                }
            } else {
                usage_text = command.usage(&parts);
            }
            suggestions = current_suggestions;
        }
    }
    
    let mut offset = 5;
    let popup_x = left_width + 3; // | の右側
    let popup_width = width.saturating_sub(popup_x);

    if let Some(usage) = &usage_text {
        let lines: Vec<&str> = usage.lines().collect();
        for (i, line) in lines.iter().rev().enumerate() {
            let y = height.saturating_sub(offset + i);
            term.move_cursor_to(popup_x, y)?;
            let display_line = format!("{:<width$}", line, width = popup_width);
            term.write_str(&style(display_line).black().on_white().to_string())?;
        }
        offset += lines.len();
    }

    if !suggestions.is_empty() && usage_text.is_none() {
        let max_suggestions = 10;
        let display_count = suggestions.len().min(max_suggestions);
        
        for (idx, (name, desc)) in suggestions.iter().take(display_count).enumerate() {
            let y = height.saturating_sub(offset + (display_count - 1 - idx));
            term.move_cursor_to(popup_x, y)?;
            let content = format!("{:<10} | {:<width$}", name, desc, width = popup_width.saturating_sub(13));
            term.write_str(&style(content).black().on_white().to_string())?;
        }
    }

    Ok(())
}
