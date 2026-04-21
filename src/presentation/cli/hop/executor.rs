use anyhow::Result;
use console::{style, measure_text_width, strip_ansi_codes};
use crate::presentation::cli::hop::app::HopApp;
use crate::presentation::cli::hop::ui;

pub fn execute_command(app: &mut HopApp) -> Result<bool> {
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
            app.is_command_mode = false;
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
    
    // terminalビューの判定
    app.is_terminal_view = cmd_name == "terminal";

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
        Ok(format!("no such command in usagi: {}", cmd_name))
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

pub fn push_to_history(history: &mut Vec<String>, text: &str, max_width: usize) {
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
