use anyhow::{Result, Context};
use console::Key;
use crate::presentation::cli::hop::app::HopApp;
use crate::presentation::cli::hop::executor;
use crate::presentation::cli::hop::completion;

pub async fn handle_key(app: &mut HopApp) -> Result<bool> {
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

    if let Some(modal) = &mut app.input_modal {
        match key {
            Key::Enter => {
                let value = modal.value.clone();
                if let Some(modal) = app.input_modal.take() {
                    (modal.on_submit)(app, value).await?;
                }
                return Ok(true);
            }
            Key::Escape => {
                app.input_modal = None;
                return Ok(true);
            }
            Key::Backspace => {
                modal.value.pop();
                return Ok(true);
            }
            Key::Char(c) => {
                modal.value.push(c);
                return Ok(true);
            }
            _ => return Ok(true),
        }
    }

    if let Some(modal) = &mut app.select_modal {
        match key {
            Key::ArrowUp => {
                if modal.selected_index > 0 {
                    modal.selected_index -= 1;
                } else if !modal.items.is_empty() {
                    modal.selected_index = modal.items.len() - 1;
                }
                return Ok(true);
            }
            Key::ArrowDown => {
                if modal.selected_index < modal.items.len().saturating_sub(1) {
                    modal.selected_index += 1;
                } else {
                    modal.selected_index = 0;
                }
                return Ok(true);
            }
            Key::Enter => {
                if !modal.items.is_empty() {
                    let selected = modal.items[modal.selected_index].clone();
                    if let Some(modal) = app.select_modal.take() {
                        (modal.on_select)(app, selected).await?;
                    }
                } else {
                    app.select_modal = None;
                }
                return Ok(true);
            }
            Key::Escape | Key::Char('q') => {
                app.select_modal = None;
                return Ok(true);
            }
            _ => return Ok(true),
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
                if !executor::execute_command(app).await? {
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
