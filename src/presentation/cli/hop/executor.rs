use anyhow::Result;
use std::sync::Arc;
use futures::StreamExt;
use crate::presentation::cli::hop::app::HopApp;
use crate::presentation::commands::{CommandContext, CommandAction, CommandEvent, Command};

pub async fn execute_command(app: &mut HopApp) -> Result<bool> {
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
        state: app.state.clone(),
        worktrees: app.worktrees.clone(),
        selected_index: app.selected_index,
        input_history: app.history.input_history.history.clone(),
        project_path: app.project_path.clone(),
    };

    // コマンドの特定
    let (matched_cmd, is_interact) = if let Some(cmd) = &app.active_interaction {
        (Some(Arc::clone(cmd)), true)
    } else {
        let cmd = app.commands.iter()
            .find(|c| c.is_match(&parts))
            .map(|c| Arc::clone(c));
        (cmd, false)
    };

    let result = if let Some(cmd) = matched_cmd {
        if is_interact {
            let mut stream = cmd.interact(context).await?;
            let mut last_res = Ok(true);
            while let Some(event) = stream.next().await {
                match event? {
                    CommandEvent::DisplayMessage(msg) => {
                        let (_, term_width) = app.term.size();
                        let right_width = (term_width as usize).saturating_sub(30).saturating_sub(3);
                        app.history.push_output(&msg, right_width);
                        let _ = crate::presentation::cli::hop::ui::render(app);
                        let _ = app.term.flush();
                    }
                    CommandEvent::Action(action) => {
                        last_res = handle_action(app, action, &cmd, parts.clone()).await;
                    }
                }
            }
            last_res
        } else {
            let action = cmd.execute(context).await?;
            handle_action(app, action, &cmd, parts).await
        }
    } else {
        execute_fallback(app, parts).await
    };

    let (term_height, _term_width) = app.term.size();
    app.history.limit_output((term_height as usize).saturating_sub(7).max(1));
    app.history.reset_input_index();

    result
}

async fn handle_action(app: &mut HopApp, action: CommandAction, cmd: &Arc<dyn Command>, parts: Vec<String>) -> Result<bool> {
    match action {
        CommandAction::None => {
            execute_fallback(app, parts).await
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
            app.active_interaction = Some(Arc::clone(cmd));
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
            let (result, is_long_running, command) = app.run_command_with_parts(parts.clone(), &cmd_to_execute).await;
            app.finalize_command_execution(result, &selected_worktree, &cmd_to_execute, &cmd_to_execute, 0, is_long_running, &parts, command);
            Ok(true)
        }
        CommandAction::Exit => Ok(false),
    }
}

async fn execute_fallback(app: &mut HopApp, parts: Vec<String>) -> Result<bool> {
    // デフォルト処理 (外部コマンド実行などのフォールバック)
    let cmd_to_execute = app.current_input.clone();
    let selected_worktree = app.worktrees[app.selected_index].clone();
    let backup_input = app.current_input.clone();
    let backup_cursor = app.cursor_pos;

    let (result, is_long_running, command) = app.run_command_with_parts(parts.clone(), &cmd_to_execute).await;
    app.finalize_command_execution(result, &selected_worktree, &cmd_to_execute, &backup_input, backup_cursor, is_long_running, &parts, command);
    Ok(true)
}


