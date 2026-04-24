use anyhow::Result;
use std::sync::Arc;
use crate::presentation::cli::hop::app::HopApp;

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

    // コマンドの特定と実行
    let matched_cmd = app.commands.iter()
        .find(|c| c.is_match(app, &parts))
        .map(|c| Arc::clone(c));

    let result = if let Some(cmd) = matched_cmd {
        cmd.execute(app, parts)
    } else {
        // デフォルト処理 (外部コマンド実行などのフォールバック)
        let cmd_to_execute = app.current_input.clone();
        let selected_worktree = app.worktrees[app.selected_index].clone();
        let backup_input = app.current_input.clone();
        let backup_cursor = app.cursor_pos;

        let (result, show_thinking) = app.run_command_with_parts(parts, &cmd_to_execute);
        app.finalize_command_execution(result, &selected_worktree, &cmd_to_execute, &backup_input, backup_cursor, show_thinking);
        Ok(true)
    };

    let (term_height, _term_width) = app.term.size();
    app.history.limit_output((term_height as usize).saturating_sub(7).max(1));
    app.history.reset_input_index();

    result
}


