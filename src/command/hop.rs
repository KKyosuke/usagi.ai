use anyhow::{Result, Context, anyhow};
use std::path::PathBuf;
use console::{Term, Key, style, measure_text_width};
use crate::application::init::get_project_state;
use crate::application::layout::AppMode;

pub fn run(project_path: PathBuf, initial_worktree: Option<String>) -> Result<()> {
    // 1 & 2. ProjectState の読み込みと初期化チェック
    let state = get_project_state(&project_path)
        .map_err(|_| anyhow!("Error: Not an initialized directory. Please run `usagi init` first."))?;

    std::env::set_current_dir(&project_path).context(format!("Failed to change directory to {}", project_path.display()))?;

    // 3. ワークツリー一覧の作成 (main + state.worktrees)
    let mut worktrees = vec!["main".to_string()];
    worktrees.extend(state.worktrees.clone());

    // 4. TUI 画面の表示
    let term = Term::stdout();
    let mut selected_index = 0;
    let mut current_input = String::new();
    let mut is_command_mode = false;
    let available_commands = vec!["start", "ai", "close"];

    // 初期選択のワークツリーがあれば設定
    if let Some(initial_wt) = initial_worktree {
        if let Some(idx) = worktrees.iter().position(|wt| wt == &initial_wt) {
            selected_index = idx;
        }
    }

    let mut command_history: Vec<String> = vec![];
    
    // 画面全体を一度クリア
    term.clear_screen()?;

    loop {
        let (height, width) = term.size();
        let left_width = 25; 
        let right_width = (width as usize).saturating_sub(left_width).saturating_sub(3); // 3 for separators
        
        // ヘッダー表示
        term.move_cursor_to(0, 0)?;
        let mode = if is_command_mode {
            AppMode::Command
        } else {
            AppMode::Menu
        };
        term.write_line(&format!("{}", style("----- USAGI TERMINAL -----").magenta().bold()))?;
        term.write_line(&format!("MODE: {}", style(mode.label()).bold().cyan()))?;

        // 左右分割描画
        for i in 0..(height as usize - 6) {
            let left_content = if i == 0 {
                style("workspace").bold().to_string()
            } else if i - 1 < worktrees.len() {
                let wt_idx = i - 1;
                let wt = &worktrees[wt_idx];
                if wt_idx == selected_index {
                    format!("> {}", style(wt).cyan().bold())
                } else {
                    format!("  {}", wt)
                }
            } else {
                "".to_string()
            };

            // 左側の幅を調整
            let left_padding = left_width.saturating_sub(measure_text_width(&left_content));
            let left_display = format!("{}{:width$}", left_content, "", width = left_padding);
            
            // 右側の表示内容 (履歴)
            let right_content = if i == 0 {
                format!("Welcome to usagi terminal! (Workspace: {})", worktrees[selected_index])
            } else {
                let history_idx = i.saturating_sub(1);
                if history_idx < command_history.len() {
                    command_history[history_idx].clone()
                } else {
                    "".to_string()
                }
            };
            
            let right_padding = right_width.saturating_sub(measure_text_width(&right_content));
            let right_display = format!("{}{:width$}", right_content, "", width = right_padding);

            term.write_line(&format!("{} | {}", left_display, right_display))?;
        }

        // 入力欄
        term.move_cursor_to(0, height as usize - 4)?;
        term.write_line(&format!("{:-<width$}", "", width = width as usize))?;
        let command_padding = left_width.saturating_sub(measure_text_width("COMMAND"));
        let command_prompt = format!("COMMAND{:padding$} | {}", "", current_input, padding = command_padding);
        let command_display = format!("{:width$}", command_prompt, width = width as usize);
        term.write_line(&command_display)?;

        // 下部ヘルプ
        term.move_cursor_to(0, height as usize - 2)?;
        let help_text = if is_command_mode {
            let suggestions: Vec<&str> = available_commands.iter()
                .filter(|c| c.starts_with(&current_input))
                .cloned()
                .collect();
            let suggestion_text = if suggestions.is_empty() {
                "".to_string()
            } else {
                format!(" (Suggestions: {})", suggestions.join(", "))
            };
            format!("Enter: execute, Escape: cancel, Type to command...{}", style(suggestion_text).yellow())
        } else {
            style("Use Up/Down to select, Enter to type command, 'q' to quit.").dim().to_string()
        };
        term.write_line(&format!("{:width$}", help_text, width = width as usize))?;

        if is_command_mode {
            let cursor_x = left_width + 3 + measure_text_width(&current_input);
            term.move_cursor_to(cursor_x, height as usize - 3)?;
            term.show_cursor()?;
        } else {
            term.hide_cursor()?;
        }

        let key = term.read_key().context("Failed to read key")?;

        match key {
            Key::ArrowUp if !is_command_mode => {
                if selected_index > 0 {
                    selected_index -= 1;
                } else {
                    selected_index = worktrees.len().saturating_sub(1);
                }
            }
            Key::ArrowDown if !is_command_mode => {
                if selected_index < worktrees.len().saturating_sub(1) {
                    selected_index += 1;
                } else {
                    selected_index = 0;
                }
            }
            Key::Enter => {
                if is_command_mode {
                    if !current_input.is_empty() {
                        command_history.push(current_input.clone());
                        if command_history.len() > (height as usize - 7) {
                            command_history.remove(0);
                        }
                        current_input.clear();
                    }
                    is_command_mode = false;
                } else {
                    is_command_mode = true;
                }
            }
            Key::Char(c) if is_command_mode => {
                current_input.push(c);
            }
            Key::Backspace if is_command_mode => {
                current_input.pop();
            }
            Key::Escape if is_command_mode => {
                is_command_mode = false;
                current_input.clear();
            }
            Key::Char('q') | Key::Escape if !is_command_mode => {
                term.clear_screen()?;
                println!("Hop exited.");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
