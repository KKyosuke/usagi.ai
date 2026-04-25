use anyhow::Result;
use std::sync::Arc;
use crate::presentation::cli::hop::app::HopApp;
use crate::presentation::commands::{CommandContext, CommandAction};

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

    let context = CommandContext {
        parts: parts.clone(),
        state: &app.state,
        worktrees: &app.worktrees,
        selected_index: app.selected_index,
        is_interaction_mode: app.active_interaction.is_some(),
        input_history: &app.history.input_history.history,
        project_path: &app.project_path,
    };

    // コマンドの特定
    let (matched_cmd, is_interact) = if let Some(cmd) = &app.active_interaction {
        (Some(Arc::clone(cmd)), true)
    } else {
        let cmd = app.commands.iter()
            .find(|c| c.is_match(&context))
            .map(|c| Arc::clone(c));
        (cmd, false)
    };

    let result = if let Some(cmd) = matched_cmd {
        let action = if is_interact {
            cmd.interact(context)?
        } else {
            cmd.execute(context)?
        };
        match action {
            CommandAction::None => {
                execute_fallback(app, parts)
            }
            CommandAction::Consumed => Ok(true),
            CommandAction::ClearInput => {
                app.current_input.clear();
                app.cursor_pos = 0;
                app.history.reset_input_index();
                Ok(true)
            }
            CommandAction::DisplayMessage(msg) => {
                let (_, term_width) = app.term.size();
                let right_width = (term_width as usize).saturating_sub(30).saturating_sub(3);
                app.history.push_output(&msg, right_width);
                app.current_input.clear();
                app.cursor_pos = 0;
                Ok(true)
            }
            CommandAction::SetSelectModal(modal) => {
                app.select_modal = Some(modal);
                app.current_input.clear();
                app.cursor_pos = 0;
                Ok(true)
            }
            CommandAction::SetInputModal(modal) => {
                app.input_modal = Some(modal);
                app.current_input.clear();
                app.cursor_pos = 0;
                Ok(true)
            }
            CommandAction::EnterInteraction(msg) => {
                app.active_interaction = Some(Arc::clone(&cmd));
                app.current_input.clear();
                app.cursor_pos = 0;
                app.history.reset_input_index();
                app.history.clear_output();
                let (_, term_width) = app.term.size();
                let right_width = (term_width as usize).saturating_sub(30).saturating_sub(3);
                
                app.history.push_output(&msg, right_width);
                Ok(true)
            }
            CommandAction::ExitInteraction(msg) => {
                app.active_interaction = None;
                app.is_command_mode = false;
                app.current_input.clear();
                app.cursor_pos = 0;
                app.history.reset_input_index();
                app.history.clear_output();
                let (_, term_width) = app.term.size();
                let right_width = (term_width as usize).saturating_sub(30).saturating_sub(3);
                
                app.history.push_output(&msg, right_width);
                Ok(true)
            }
            CommandAction::RunCommand { parts, cmd_to_execute, close_after } => {
                if close_after {
                    app.is_command_mode = false;
                }
                let selected_worktree = app.worktrees[app.selected_index].clone();
                let (result, show_thinking) = app.run_command_with_parts(parts, &cmd_to_execute);
                app.finalize_command_execution(result, &selected_worktree, &cmd_to_execute, &cmd_to_execute, 0, show_thinking);
                Ok(true)
            }
            CommandAction::Exit => Ok(false),
        }
    } else {
        execute_fallback(app, parts)
    };

    let (term_height, _term_width) = app.term.size();
    app.history.limit_output((term_height as usize).saturating_sub(7).max(1));
    app.history.reset_input_index();

    result
}

fn execute_fallback(app: &mut HopApp, parts: Vec<String>) -> Result<bool> {
    // デフォルト処理 (外部コマンド実行などのフォールバック)
    let cmd_to_execute = app.current_input.clone();
    let selected_worktree = app.worktrees[app.selected_index].clone();
    let backup_input = app.current_input.clone();
    let backup_cursor = app.cursor_pos;

    let (result, show_thinking) = app.run_command_with_parts(parts, &cmd_to_execute);
    app.finalize_command_execution(result, &selected_worktree, &cmd_to_execute, &backup_input, backup_cursor, show_thinking);
    Ok(true)
}


