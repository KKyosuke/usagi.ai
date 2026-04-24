use anyhow::{Result, Context};
use console::Key;
use crate::presentation::cli::hop::app::HopApp;
use crate::presentation::cli::hop::executor;
use crate::presentation::cli::hop::completion;

pub fn handle_key(app: &mut HopApp) -> Result<bool> {
    let key = match app.term.read_key() {
        Ok(k) => k,
        Err(e) => {
            if e.to_string().contains("read interrupted") {
                if app.is_command_mode {
                    app.current_input.clear();
                    app.history.reset_input_index();
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

    if app.is_input_modal_mode {
        match key {
            Key::Enter => {
                let value = app.input_modal_value.clone();
                app.is_input_modal_mode = false;
                app.input_modal_value.clear();
                if let Some(on_submit) = app.input_modal_on_submit.take() {
                    on_submit(app, value)?;
                }
                return Ok(true);
            }
            Key::Escape => {
                app.is_input_modal_mode = false;
                app.input_modal_value.clear();
                app.input_modal_on_submit = None;
                return Ok(true);
            }
            Key::Backspace => {
                app.input_modal_value.pop();
                return Ok(true);
            }
            Key::Char(c) => {
                app.input_modal_value.push(c);
                return Ok(true);
            }
            _ => return Ok(true),
        }
    }

    if app.is_modal_mode {
        match key {
            Key::ArrowUp => {
                if app.modal_selected_index > 0 {
                    app.modal_selected_index -= 1;
                } else if !app.modal_items.is_empty() {
                    app.modal_selected_index = app.modal_items.len() - 1;
                }
                return Ok(true);
            }
            Key::ArrowDown => {
                if app.modal_selected_index < app.modal_items.len().saturating_sub(1) {
                    app.modal_selected_index += 1;
                } else {
                    app.modal_selected_index = 0;
                }
                return Ok(true);
            }
            Key::Enter => {
                if !app.modal_items.is_empty() {
                    let selected = app.modal_items[app.modal_selected_index].clone();
                    app.is_modal_mode = false;
                    app.modal_items.clear();
                    if let Some(on_select) = app.modal_on_select.take() {
                        on_select(app, selected)?;
                    }
                } else {
                    app.is_modal_mode = false;
                    app.modal_items.clear();
                }
                return Ok(true);
            }
            Key::Escape | Key::Char('q') => {
                app.is_modal_mode = false;
                app.modal_items.clear();
                app.modal_on_select = None;
                return Ok(true);
            }
            _ => return Ok(true),
        }
    }

    if app.is_model_selection_mode {
        match key {
            Key::ArrowUp => {
                if app.model_selection_index > 0 {
                    app.model_selection_index -= 1;
                } else if !app.available_models.is_empty() {
                    app.model_selection_index = app.available_models.len() - 1;
                }
                return Ok(true);
            }
            Key::ArrowDown => {
                if app.model_selection_index < app.available_models.len().saturating_sub(1) {
                    app.model_selection_index += 1;
                } else {
                    app.model_selection_index = 0;
                }
                return Ok(true);
            }
            Key::Enter => {
                if !app.available_models.is_empty() {
                    let selected = app.available_models[app.model_selection_index].clone();
                    if let Some(user_dirs) = directories::UserDirs::new() {
                        let models_dir = user_dirs.home_dir().join(".usagi").join("models");
                        let full_path = models_dir.join(&selected).to_string_lossy().to_string();
                        app.state.ai_model = Some(full_path.clone());
                        let _ = crate::infrastructure::project_state::save_project_state(&app.project_path, &app.state);
                        
                        let (_term_height, term_width) = app.term.size();
                        let right_width = (term_width as usize).saturating_sub(30).saturating_sub(3);
                        app.history.push_output(&format!("{}", console::style(format!("Default AI model set to: {}", selected)).green()), right_width);
                    }
                }
                
                app.is_model_selection_mode = false;
                
                if app.enter_chat_on_selection && app.state.ai_model.is_some() {
                    app.is_ai_chat_mode = true;
                    app.history.clear_output();
                    let (_term_height, term_width) = app.term.size();
                    let right_width = (term_width as usize).saturating_sub(30).saturating_sub(3);
                    app.history.push_output(&format!("{}", console::style("🐰 Entered AI Chat Mode. Type 'exit' to end.").cyan().bold()), right_width);
                }
                
                return Ok(true);
            }
            Key::Escape | Key::Char('q') => {
                app.is_model_selection_mode = false;
                let (_term_height, term_width) = app.term.size();
                let right_width = (term_width as usize).saturating_sub(30).saturating_sub(3);
                app.history.push_output(&format!("{}", console::style("Model selection cancelled.").yellow()), right_width);
                return Ok(true);
            }
            _ => {
                return Ok(true); // Ignore other keys
            }
        }
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
                if let Some(cmd) = app.history.prev_input() {
                    app.current_input = cmd;
                    app.cursor_pos = app.current_input.chars().count();
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
                if let Some(cmd) = app.history.next_input() {
                    app.current_input = cmd;
                    app.cursor_pos = app.current_input.chars().count();
                } else if app.history.input_index.is_none() {
                    app.current_input.clear();
                    app.cursor_pos = 0;
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
                if !executor::execute_command(app)? {
                    return Ok(false);
                }
            } else {
                app.is_command_mode = true;
                app.cursor_pos = 0;
                app.history.reset_input_index();
            }
        }
        Key::Char(c) if app.is_command_mode => {
            let byte_offset: usize = app.current_input.chars().take(app.cursor_pos).map(|c| c.len_utf8()).sum();
            app.current_input.insert(byte_offset, c);
            app.cursor_pos += 1;
            app.history.reset_input_index();
        }
        Key::Tab if app.is_command_mode => {
            completion::handle_tab(app);
        }
        Key::Backspace if app.is_command_mode => {
            if app.cursor_pos > 0 {
                let byte_offset: usize = app.current_input.chars().take(app.cursor_pos - 1).map(|c| c.len_utf8()).sum();
                app.current_input.remove(byte_offset);
                app.cursor_pos -= 1;
            }
            app.history.reset_input_index();
        }
        Key::Escape if app.is_command_mode => {
            app.is_command_mode = false;
            app.current_input.clear();
            app.cursor_pos = 0;
            app.history.reset_input_index();
        }
        Key::Char('q') | Key::Escape if !app.is_command_mode => {
            return Ok(false);
        }
        _ => {}
    }

    Ok(true)
}
