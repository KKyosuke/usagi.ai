use anyhow::{Result, Context};
use console::Key;
use crate::presentation::cli::hop::app::HopApp;

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
    
    let result: Result<String> = if let Some(command) = app.commands.iter().find(|c| c.name() == cmd_name) {
        if cmd_name == "space" {
            app.is_command_mode = false;
            if parts.len() == 1 {
                parts.push(app.worktrees[app.selected_index].clone());
            }
        }
        command.run(parts, &app.project_path)
    } else {
        let available: Vec<String> = app.commands.iter().map(|c| c.name().to_string()).collect();
        Ok(format!("Unknown command: {}\nAvailable commands: {}", cmd_name, available.join(", ")))
    };

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
        // 履歴に追加
        app.command_history.push(cmd_to_execute.clone());
        match result {
            Ok(output) => {
                if !output.is_empty() {
                    for line in output.lines() {
                        if !line.trim().is_empty() {
                            app.command_history.push(line.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                for line in e.to_string().lines() {
                    if !line.trim().is_empty() {
                        app.command_history.push(line.to_string());
                    }
                }
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

    let (_, height) = app.term.size();
    if app.command_history.len() > (height as usize - 7) {
        app.command_history.remove(0);
    }
    app.current_input.clear();
    app.cursor_pos = 0;
    app.history_index = None;

    Ok(true)
}

fn handle_tab_completion(app: &mut HopApp) {
    let parts: Vec<&str> = app.current_input.split_whitespace().collect();
    if !app.current_input.contains(' ') {
        // コマンド名の補完
        let suggestions: Vec<&str> = app.commands.iter()
            .map(|c| c.name())
            .filter(|name| name.starts_with(&app.current_input))
            .collect();
        if let Some(first) = suggestions.first() {
            app.current_input = first.to_string();
            app.cursor_pos = app.current_input.chars().count();
        }
    } else if !parts.is_empty() {
        // 引数の補完
        let cmd_name = parts[0];
        if let Some(command) = app.commands.iter().find(|c| c.name() == cmd_name) {
            let last_part = if app.current_input.ends_with(' ') { "" } else { parts.last().unwrap_or(&"") };
            let suggestions = command.subcommands();
            if let Some((name, _)) = suggestions.iter().find(|(name, _)| name.starts_with(last_part)) {
                let head = if app.current_input.ends_with(' ') {
                    &app.current_input
                } else {
                    app.current_input.rsplit_once(' ').map(|(h, _)| h).unwrap_or("")
                };
                if head.is_empty() || head.ends_with(' ') {
                    app.current_input = format!("{}{}", head, name);
                } else {
                    app.current_input = format!("{} {}", head, name);
                }
                app.cursor_pos = app.current_input.chars().count();
            }
        }
    }
}
