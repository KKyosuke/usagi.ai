use anyhow::Result;
use console::style;
use crate::presentation::cli::hop::app::HopApp;
use crate::presentation::cli::hop::ui;

pub fn execute_command(app: &mut HopApp) -> Result<bool> {
    if app.current_input.trim().is_empty() {
        app.current_input.clear();
        app.cursor_pos = 0;
        app.history.reset_input_index();
        return Ok(true);
    }

    let parts: Vec<String> = app.current_input.split_whitespace().map(|s| s.to_string()).collect();
    if parts.is_empty() {
        return Ok(true);
    }

    let mut cmd_to_execute = app.current_input.clone();
    let mut parts = parts;
    let selected_worktree = app.worktrees[app.selected_index].clone();

    let is_ai_set_model = parts.len() == 2 && parts[0] == "ai" && parts[1] == "--set-model";
    let is_ai_chat = parts.len() == 2 && parts[0] == "ai" && parts[1] == "chat";

    if is_ai_set_model || (is_ai_chat && app.state.ai_model.is_none()) {
        let (_term_height, term_width) = app.term.size();
        let left_width = 30;
        let right_width = (term_width as usize).saturating_sub(left_width).saturating_sub(3);
        let prompt_text = format!("{} {} {}", style(&selected_worktree).cyan(), ">", cmd_to_execute);
        app.history.push_output(&prompt_text, right_width);

        if let Some(user_dirs) = directories::UserDirs::new() {
            let models_dir = user_dirs.home_dir().join(".usagi").join("models");
            let mut available_models = Vec::new();
            if models_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&models_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() && path.extension().map_or(false, |ext| ext == "gguf") {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                available_models.push(name.to_string());
                            }
                        }
                    }
                }
            }
            
            if available_models.is_empty() {
                app.history.push_output(&format!("{}", style("No models found in ~/.usagi/models/. Please run 'usagi ai install' first.").red()), right_width);
            } else {
                app.available_models = available_models;
                app.model_selection_index = 0;
                app.enter_chat_on_selection = is_ai_chat;
                app.is_model_selection_mode = true;
            }
        }

        app.current_input.clear();
        app.cursor_pos = 0;
        app.history.reset_input_index();
        return Ok(true);
    }

    let is_session_start_remote = parts.len() >= 2 && parts[0] == "session" && parts[1] == "start" && parts.iter().any(|p| p == "--remote" || p == "-r");
    if is_session_start_remote {
        let (_term_height, term_width) = app.term.size();
        let left_width = 30;
        let right_width = (term_width as usize).saturating_sub(left_width).saturating_sub(3);
        let prompt_text = format!("{} {} {}", style(&selected_worktree).cyan(), ">", cmd_to_execute);
        app.history.push_output(&prompt_text, right_width);

        app.history.push_output(&format!("{}", style("Fetching remote branches...").dim()), right_width);
        ui::render(app)?;
        let _ = app.term.flush();

        let _ = crate::infrastructure::git::fetch(&app.project_path, "origin");
        let remote_branches = crate::infrastructure::git::list_remote_branches(&app.project_path)?;
        app.history.pop_output(); // Remove "Fetching..."

        if remote_branches.is_empty() {
            app.history.push_output(&format!("{}", style("No remote branches found.").red()), right_width);
        } else {
            app.is_modal_mode = true;
            app.modal_title = "Select base branch from remote".to_string();
            app.modal_items = remote_branches;
            app.modal_selected_index = 0;
            
            let parts_clone = parts.clone();
            app.modal_on_select = Some(Box::new(move |app, selected| {
                let mut new_parts = parts_clone;
                // --remote / -r を削除
                new_parts.retain(|p| p != "--remote" && p != "-r");

                // ブランチ名が既に提供されているかチェック
                // new_parts は ["session", "start", "BRANCH"] の形式を想定
                let has_branch = new_parts.len() >= 3;

                // --base <selected> を追加
                new_parts.push("--base".to_string());
                new_parts.push(selected.clone());
                
                if has_branch {
                    app.current_input = new_parts.join(" ");
                    execute_command(app)?;
                } else {
                    // Ask for branch name
                    app.is_input_modal_mode = true;
                    app.input_modal_title = "Enter session branch name:".to_string();
                    app.input_modal_value = selected.split('/').last().unwrap_or(&selected).to_string();
                    app.input_modal_on_submit = Some(Box::new(move |app, branch_name| {
                        new_parts.push(branch_name);
                        app.current_input = new_parts.join(" ");
                        execute_command(app)?;
                        Ok(())
                    }));
                }
                Ok(())
            }));
        }

        app.current_input.clear();
        app.cursor_pos = 0;
        app.history.reset_input_index();
        return Ok(true);
    }

    if is_ai_chat && !app.state.ai_model.is_none() {
        app.is_ai_chat_mode = true;
        app.current_input.clear();
        app.cursor_pos = 0;
        app.history.reset_input_index();
        app.history.clear_output();
        let (_term_height, term_width) = app.term.size();
        let left_width = 30;
        let right_width = (term_width as usize).saturating_sub(left_width).saturating_sub(3);
        app.history.push_output(&format!("{}", style("🐰 Entered AI Chat Mode. Type 'exit' to end.").cyan().bold()), right_width);
        return Ok(true);
    }

    if app.is_ai_chat_mode {
        let original_input = cmd_to_execute.trim();
        if original_input.eq_ignore_ascii_case("exit") || original_input.eq_ignore_ascii_case("quit") {
            app.is_ai_chat_mode = false;
            app.is_command_mode = false;
            app.current_input.clear();
            app.cursor_pos = 0;
            app.history.reset_input_index();
            app.history.clear_output();
            let (_term_height, term_width) = app.term.size();
            let left_width = 30;
            let right_width = (term_width as usize).saturating_sub(left_width).saturating_sub(3);
            app.history.push_output(&format!("{}", style("AI chat session ended.").dim()), right_width);
            return Ok(true);
        }
        parts = vec!["ai".to_string(), "chat-turn".to_string(), original_input.to_string()];
    }

    if let Ok(index) = parts[0].parse::<usize>() {
        if index > 0 && index <= app.history.input_history.history.len() {
            cmd_to_execute = app.history.input_history.history[index - 1].clone();
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
        app.history.push_output(&prompt, right_width);
    }

    let mut show_thinking = false;
    if app.is_ai_chat_mode && cmd_name == "ai" && parts.get(1).map(|s| s.as_str()) == Some("chat-turn") {
        app.history.push_output(&format!("{}", style("🐰 Thinking...").dim().italic()), right_width);
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
            app.history.pop_output();
        }
        match result {
            Ok(output) => {
                app.history.push_output(&output, right_width);
            }
            Err(e) => {
                app.history.push_output(&e.to_string(), right_width);
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

    app.history.limit_output((term_height as usize).saturating_sub(7).max(1));
    app.history.reset_input_index();

    Ok(true)
}


