use anyhow::{Result, Context};
use console::{Key, measure_text_width, strip_ansi_codes, style};
use crate::presentation::cli::hop::app::HopApp;
use crate::presentation::cli::hop::ui;

pub fn handle_key(app: &mut HopApp) -> Result<bool> {
    let key = match app.term.read_key() {
        Ok(k) => k,
        Err(e) => {
            if e.to_string().contains("read interrupted") {
                if app.is_command_mode {
                    app.current_input.clear();
                    app.history_index = None;
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
            return Err(anyhow::Error::from(e)).context("Failed to read key");
        }
    };

    if !matches!(key, Key::Tab) && app.is_command_mode {
        app.tab_completion_base = None;
        app.suggestion_index = None;
    }

    match key {
        Key::ArrowLeft if app.is_command_mode => {
            if app.cursor_pos > 0 {
                app.cursor_pos -= 1;
            }
        }
        Key::ArrowRight if app.is_command_mode => {
            if app.cursor_pos < app.current_input.chars().count() {
                app.cursor_pos += 1;
            }
        }
        Key::ArrowUp => {
            if app.is_command_mode {
                if !app.state.history.is_empty() {
                    let new_index = match app.history_index {
                        None => Some(app.state.history.len() - 1),
                        Some(idx) if idx > 0 => Some(idx - 1),
                        Some(_) => Some(0),
                    };
                    if let Some(idx) = new_index {
                        app.history_index = Some(idx);
                        app.current_input = app.state.history[idx].clone();
                        app.cursor_pos = app.current_input.chars().count();
                    }
                }
            } else {
                if app.selected_index > 0 {
                    app.selected_index -= 1;
                } else {
                    app.selected_index = app.worktrees.len().saturating_sub(1);
                }
            }
        }
        Key::ArrowDown => {
            if app.is_command_mode {
                if let Some(idx) = app.history_index {
                    if idx < app.state.history.len() - 1 {
                        let next_idx = idx + 1;
                        app.history_index = Some(next_idx);
                        app.current_input = app.state.history[next_idx].clone();
                        app.cursor_pos = app.current_input.chars().count();
                    } else {
                        app.history_index = None;
                        app.current_input.clear();
                        app.cursor_pos = 0;
                    }
                }
            } else {
                if app.selected_index < app.worktrees.len().saturating_sub(1) {
                    app.selected_index += 1;
                } else {
                    app.selected_index = 0;
                }
            }
        }
        Key::Enter => {
            if app.is_command_mode {
                if !handle_command_execution(app)? {
                    return Ok(false);
                }
            } else {
                app.is_command_mode = true;
                app.cursor_pos = 0;
                app.history_index = None;
            }
        }
        Key::Char(c) if app.is_command_mode => {
            let byte_offset: usize = app.current_input.chars().take(app.cursor_pos).map(|c| c.len_utf8()).sum();
            app.current_input.insert(byte_offset, c);
            app.cursor_pos += 1;
            app.history_index = None;
        }
        Key::Tab if app.is_command_mode => {
            handle_tab_completion(app);
        }
        Key::Backspace if app.is_command_mode => {
            if app.cursor_pos > 0 {
                let byte_offset: usize = app.current_input.chars().take(app.cursor_pos - 1).map(|c| c.len_utf8()).sum();
                app.current_input.remove(byte_offset);
                app.cursor_pos -= 1;
            }
            app.history_index = None;
        }
        Key::Escape if app.is_command_mode => {
            app.is_command_mode = false;
            app.current_input.clear();
            app.cursor_pos = 0;
            app.history_index = None;
        }
        Key::Char('q') | Key::Escape if !app.is_command_mode => {
            return Ok(false);
        }
        _ => {}
    }

    Ok(true)
}

fn handle_command_execution(app: &mut HopApp) -> Result<bool> {
    if app.current_input.trim().is_empty() {
        app.current_input.clear();
        app.cursor_pos = 0;
        app.history_index = None;
        return Ok(true);
    }

    let parts: Vec<String> = app.current_input.split_whitespace().map(|s| s.to_string()).collect();
    if parts.is_empty() {
        return Ok(true);
    }

    let mut cmd_to_execute = app.current_input.clone();
    let mut parts = parts;
    let selected_worktree = app.worktrees[app.selected_index].clone();

    // Custom interception for AI Chat Mode entering
    if parts.len() == 2 && parts[0] == "ai" && parts[1] == "chat" {
        app.is_ai_chat_mode = true;
        app.current_input.clear();
        app.cursor_pos = 0;
        app.history_index = None;
        app.command_history.clear();
        let (_term_height, term_width) = app.term.size();
        let left_width = 30;
        let right_width = (term_width as usize).saturating_sub(left_width).saturating_sub(3);
        push_to_history(&mut app.command_history, &format!("{}", style("🐰 Entered AI Chat Mode. Type 'exit' to end.").cyan().bold()), right_width);
        return Ok(true);
    }

    if app.is_ai_chat_mode {
        let original_input = cmd_to_execute.trim();
        if original_input.eq_ignore_ascii_case("exit") || original_input.eq_ignore_ascii_case("quit") {
            app.is_ai_chat_mode = false;
            app.current_input.clear();
            app.cursor_pos = 0;
            app.history_index = None;
            app.command_history.clear();
            let (_term_height, term_width) = app.term.size();
            let left_width = 30;
            let right_width = (term_width as usize).saturating_sub(left_width).saturating_sub(3);
            push_to_history(&mut app.command_history, &format!("{}", style("AI chat session ended.").dim()), right_width);
            return Ok(true);
        }
        parts = vec!["ai".to_string(), "chat-turn".to_string(), original_input.to_string()];
    }

    // If cmd is a number, try to get from history
    if let Ok(index) = parts[0].parse::<usize>() {
        if index > 0 && index <= app.state.history.len() {
            cmd_to_execute = app.state.history[index - 1].clone();
            parts = cmd_to_execute.split_whitespace().map(|s| s.to_string()).collect();
        }
    }

    if parts.is_empty() {
        return Ok(true);
    }

    let cmd_name = parts[0].clone();
    let is_session_close = cmd_name == "session" && parts.get(1).map(|s| s.as_str()) == Some("close");
    
    // terminalビューの判定（組み込みコマンド以外はターミナルフォールバックされるため）
    let built_in_commands = ["close", "doctor", "history", "man", "session", "space"];
    app.is_terminal_view = cmd_name == "terminal" || !built_in_commands.contains(&cmd_name.as_str());

    let (_term_height, term_width) = app.term.size();
    let left_width = 30;
    let right_width = (term_width as usize).saturating_sub(left_width).saturating_sub(3);

    if cmd_name != "close" && !is_session_close {
        let prompt_sign = if app.is_ai_chat_mode { "(ai) >" } else if app.is_terminal_view { "$" } else { ">" };
        let prompt = format!("{} {} {}", style(&selected_worktree).cyan(), prompt_sign, cmd_to_execute);
        push_to_history(&mut app.command_history, &prompt, right_width);
    }

    let mut show_thinking = false;
    if app.is_ai_chat_mode && cmd_name == "ai" && parts.get(1).map(|s| s.as_str()) == Some("chat-turn") {
        push_to_history(&mut app.command_history, &format!("{}", style("🐰 Thinking...").dim().italic()), right_width);
        show_thinking = true;
    }

    // Clear input early before rendering
    let backup_input = app.current_input.clone();
    let backup_cursor = app.cursor_pos;
    app.current_input.clear();
    app.cursor_pos = 0;

    ui::render(app)?;
    let _ = app.term.flush();

    let result: Result<String> = if let Some(command) = app.commands.iter().find(|c| c.name() == cmd_name) {
        if cmd_name == "space" {
            app.is_command_mode = false;
            if parts.len() == 1 {
                parts.push(app.worktrees[app.selected_index].clone());
            }
        }
        command.run(parts, &app.project_path, &selected_worktree, &app.term)
    } else {
        // Fallback to terminal command
        if let Some(terminal_command) = app.commands.iter().find(|c| c.name() == "terminal") {
            let mut terminal_args = vec!["terminal".to_string()];
            terminal_args.extend(parts);
            terminal_command.run(terminal_args, &app.project_path, &selected_worktree, &app.term)
        } else {
            let available: Vec<String> = app.commands.iter().map(|c| c.name().to_string()).collect();
            Ok(format!("Unknown command: {}\nAvailable commands: {}", cmd_name, available.join(", ")))
        }
    };

    // Re-enter alternate screen and hide cursor to ensure TUI state
    let _ = app.term.write_str("\x1b[?1049h");
    let _ = app.term.hide_cursor();
    let _ = app.term.flush();

    let (term_height, term_width) = app.term.size();
    let left_width = 30;
    let right_width = (term_width as usize).saturating_sub(left_width).saturating_sub(3);

    if cmd_name == "close" || is_session_close {
        if result.is_ok() {
            app.is_command_mode = false;
            app.refresh_state()?;
            
            if let Some(idx) = app.worktrees.iter().position(|wt| wt == &selected_worktree) {
                app.selected_index = idx;
            } else if let Some(current_wt) = &app.state.current_worktree {
                if let Some(idx) = app.worktrees.iter().position(|wt| wt == current_wt) {
                    app.selected_index = idx;
                }
            } else {
                app.selected_index = 0;
            }
        }
    } else {
        if show_thinking {
            app.command_history.pop();
        }
        match result {
            Ok(output) => {
                push_to_history(&mut app.command_history, &output, right_width);
            }
            Err(e) => {
                push_to_history(&mut app.command_history, &e.to_string(), right_width);
                // Restore input on error so they don't have to re-type
                app.current_input = backup_input;
                app.cursor_pos = backup_cursor;
            }
        }

        // 状態を更新し（save_history内でrefresh_stateを呼ぶ）、履歴を保存
        app.save_history(&cmd_to_execute)?;
        
        let mut updated = false;
        if cmd_name == "space" {
            if let Some(current_wt) = &app.state.current_worktree {
                if let Some(idx) = app.worktrees.iter().position(|wt| wt == current_wt) {
                    app.selected_index = idx;
                    updated = true;
                }
            }
        }

        if !updated {
            if let Some(idx) = app.worktrees.iter().position(|wt| wt == &selected_worktree) {
                app.selected_index = idx;
            } else if let Some(current_wt) = &app.state.current_worktree {
                if let Some(idx) = app.worktrees.iter().position(|wt| wt == current_wt) {
                    app.selected_index = idx;
                }
            } else {
                app.selected_index = 0;
            }
        }
    }

    while app.command_history.len() > (term_height as usize - 7).max(1) {
        app.command_history.remove(0);
    }
    app.history_index = None;

    Ok(true)
}

fn handle_tab_completion(app: &mut HopApp) {
    if app.tab_completion_base.is_none() {
        app.tab_completion_base = Some(app.current_input.clone());
        app.suggestion_index = None;
    }

    let input = app.tab_completion_base.as_ref().unwrap().clone();
    let parts: Vec<&str> = input.split_whitespace().collect();
    let mut suggestions: Vec<String> = Vec::new();

    if !input.contains(' ') {
        // コマンド名の補完
        suggestions = app.commands.iter()
            .map(|c| c.name().to_string())
            .filter(|name| name.starts_with(&input))
            .collect();
    } else if !parts.is_empty() {
        // 引数の補完
        let cmd_name = parts[0];
        if let Some(command) = app.commands.iter().find(|c| c.name() == cmd_name) {
            let last_part = if input.ends_with(' ') { "" } else { parts.last().unwrap_or(&"") };
            suggestions = command.subcommands().into_iter()
                .filter(|(name, _)| name.starts_with(last_part))
                .map(|(name, _)| name.clone())
                .collect();
        }
    }

    if suggestions.is_empty() {
        return;
    }

    let next_idx = match app.suggestion_index {
        Some(idx) => (idx + 1) % suggestions.len(),
        None => 0,
    };
    app.suggestion_index = Some(next_idx);

    let selected = &suggestions[next_idx];

    if !input.contains(' ') {
        app.current_input = selected.clone();
    } else {
        let head = if input.ends_with(' ') {
            &input
        } else {
            input.rsplit_once(' ').map(|(h, _)| h).unwrap_or("")
        };
        if head.is_empty() || head.ends_with(' ') {
            app.current_input = format!("{}{}", head, selected);
        } else {
            app.current_input = format!("{} {}", head, selected);
        }
    }
    app.cursor_pos = app.current_input.chars().count();
}

fn push_to_history(history: &mut Vec<String>, text: &str, max_width: usize) {
    if text.is_empty() {
        return;
    }
    for line in text.lines() {
        let mut current = line.to_string();
        if current.is_empty() {
            history.push(" ".to_string());
            continue;
        }
        while measure_text_width(&strip_ansi_codes(&current)) > max_width && max_width > 0 {
            let mut split_idx = 0;
            let mut width = 0;
            let mut in_escape = false;
            
            for (i, c) in current.char_indices() {
                if c == '\x1b' {
                    in_escape = true;
                } else if in_escape {
                    if c >= '@' && c <= '~' {
                        in_escape = false;
                    }
                } else {
                    let c_width = measure_text_width(&c.to_string());
                    if width + c_width > max_width {
                        break;
                    }
                    width += c_width;
                }
                split_idx = i + c.len_utf8();
            }
            
            if split_idx == 0 || split_idx == current.len() {
                break;
            }
            
            let (head, tail) = current.split_at(split_idx);
            history.push(head.to_string());
            current = tail.to_string();
        }
        history.push(current);
    }
}
