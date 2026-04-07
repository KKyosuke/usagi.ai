use anyhow::{Result, Context, anyhow};
use std::path::PathBuf;
use console::{Term, Key, style, measure_text_width};
use crate::infrastructure::project_state::get_project_state;
use crate::presentation::tui::mode::AppMode;
use crate::presentation::tui::screen::AlternateScreenGuard;
use crate::presentation::commands::{self, Command};

pub fn run(project_path: PathBuf, initial_worktree: Option<String>) -> Result<()> {
    // 1 & 2. ProjectState の読み込みと初期化チェック
    let mut state = get_project_state(&project_path)
        .map_err(|_| anyhow!("Error: Not an initialized directory. Please run `usagi init` first."))?;

    std::env::set_current_dir(&project_path).context(format!("Failed to change directory to {}", project_path.display()))?;

    // 3. ワークツリー一覧の作成 (main + state.worktrees)
    let mut worktrees = vec!["main".to_string()];
    worktrees.extend(state.worktrees.clone());

    // 4. TUI 画面の表示
    let term = Term::stdout();
    let _guard = AlternateScreenGuard::new(term.clone())?;
    let mut selected_index = 0;
    let mut current_input = String::new();
    let mut cursor_pos = 0;
    let mut is_command_mode = false;
    let mut history_index: Option<usize> = None;
    let commands = commands::get_commands();

    // 初期選択のワークツリーがあれば設定
    if let Some(initial_wt) = initial_worktree {
        if let Some(idx) = worktrees.iter().position(|wt| wt == &initial_wt) {
            selected_index = idx;
        }
    } else if let Some(current_wt) = &state.current_worktree {
        if let Some(idx) = worktrees.iter().position(|wt| wt == current_wt) {
            selected_index = idx;
        }
    }

    let mut command_history: Vec<String> = state.history.clone();
    
    // 画面全体を一度クリア
    term.clear_screen()?;

    loop {
        let (height, width) = term.size();
        let left_width = 25; 
        let right_width = (width as usize).saturating_sub(left_width).saturating_sub(3); // 3 for separators
        
        // ヘッダー表示
        term.move_cursor_to(0, 0)?;
        term.clear_screen()?;
        let mode = if is_command_mode {
            AppMode::Command
        } else {
            AppMode::SideMenu
        };
        term.write_line(&format!("{}", style("----- USAGI TERMINAL -----").magenta().bold()))?;
        term.write_line(&format!("MODE: {}", style(mode.label()).bold().cyan()))?;

        // 左右分割描画
        let current_state = get_project_state(&project_path).ok();
        for i in 0..(height as usize - 6) {
            let left_content = if i == 0 {
                style("workspace").bold().to_string()
            } else if i - 1 < worktrees.len() {
                let wt_idx = i - 1;
                let wt = &worktrees[wt_idx];
                
                let is_active = if let Some(ref s) = current_state {
                    s.current_worktree.as_deref() == Some(wt) || (wt == "main" && s.current_worktree.is_none())
                } else {
                    false
                };

                let mark_char = "●";
                let mark_width = measure_text_width(mark_char);
                let cursor = if wt_idx == selected_index && !is_command_mode { ">" } else { " " };
                let is_selected = wt_idx == selected_index;
                let mark = if is_selected {
                    style(mark_char).green().to_string()
                } else if is_active {
                    style(mark_char).dim().to_string()
                } else {
                    " ".repeat(mark_width)
                };
                
                if wt_idx == selected_index {
                    format!("{} {}  {}", cursor, mark, style(wt).cyan().bold())
                } else {
                    format!("{} {}  {}", cursor, mark, wt)
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
        term.move_cursor_to(0, (height as usize).saturating_sub(1))?;
        let help_text = if is_command_mode {
            style("Enter: execute, Escape: cancel, Tab: select suggestion, Type to command...").dim().to_string()
        } else {
            style("Use Up/Down to select, Enter to type command, 'q' to quit.").dim().to_string()
        };
        // 最終行でのスクロールを避けるため write_str を使用
        let help_display = format!("{:width$}", help_text, width = width as usize);
        term.write_str(&help_display)?;

        // コマンドモードのポップアップ表示
        if is_command_mode {
            let parts: Vec<&str> = current_input.split_whitespace().collect();
            let mut suggestions: Vec<(String, String)> = Vec::new();
            let mut usage_text: Option<String> = None;

            if !current_input.contains(' ') {
                // コマンド名のサジェスト
                let mut current_suggestions: Vec<(String, String)> = commands.iter()
                    .filter(|c| c.name().starts_with(&current_input))
                    .map(|c| (c.name().to_string(), c.description().to_string()))
                    .collect();

                // サジェストが1つだけの場合、その詳細な使用法を表示する
                if current_suggestions.len() == 1 {
                    let name = current_suggestions[0].0.clone();
                    if let Some(command) = commands.iter().find(|c| c.name() == name) {
                        usage_text = command.usage(&[name.as_str()]);
                        if name == current_input {
                            current_suggestions.clear();
                        }
                    }
                }
                suggestions = current_suggestions;
            } else if !parts.is_empty() {
                // コマンドの引数/サブコマンドのサジェスト
                let cmd_name = parts[0];
                if let Some(command) = commands.iter().find(|c| c.name() == cmd_name) {
                    let last_part = if current_input.ends_with(' ') { "" } else { parts.last().unwrap_or(&"") };
                    let mut current_suggestions: Vec<(String, String)> = command.subcommands()
                        .into_iter()
                        .filter(|(name, _)| name.starts_with(last_part))
                        .collect();

                    // サジェストが1つだけの場合、その詳細な使用法を表示する
                    if current_suggestions.len() == 1 {
                        let name = current_suggestions[0].0.clone();
                        let is_perfect_match = name == last_part;

                        let mut check_parts = parts.clone();
                        if !current_input.ends_with(' ') {
                            if let Some(last) = check_parts.last_mut() {
                                *last = name.as_str();
                            }
                        } else {
                            // すでにそのコマンドが入力済みの場合は、そのコマンド自体のサジェストは不要
                            if parts.iter().any(|&p| p == name) {
                                current_suggestions.clear();
                            }
                            check_parts.push(name.as_str());
                        }

                        if let Some(detail_usage) = command.usage(&check_parts) {
                            usage_text = Some(detail_usage);
                        } else {
                            usage_text = command.usage(&parts);
                        }

                        if is_perfect_match {
                            current_suggestions.clear();
                        }
                    } else {
                        usage_text = command.usage(&parts);
                    }
                    suggestions = current_suggestions;
                }
            }
            
            let mut offset = 5;
            let popup_x = left_width + 3; // | の右側
            let popup_width = (width as usize).saturating_sub(popup_x);

            if let Some(usage) = &usage_text {
                let lines: Vec<&str> = usage.lines().collect();
                for (i, line) in lines.iter().rev().enumerate() {
                    let y = (height as usize).saturating_sub(offset + i);
                    term.move_cursor_to(popup_x, y)?;
                    let display_line = format!("{:<width$}", line, width = popup_width);
                    term.write_str(&style(display_line).black().on_white().to_string())?;
                }
                offset += lines.len();
            }

            if !suggestions.is_empty() && usage_text.is_none() {
                let max_suggestions = 10;
                let display_count = suggestions.len().min(max_suggestions);
                
                for (idx, (name, desc)) in suggestions.iter().take(display_count).enumerate() {
                    let y = (height as usize).saturating_sub(offset + (display_count - 1 - idx));
                    term.move_cursor_to(popup_x, y)?;
                    let content = format!("{:<10} | {:<width$}", name, desc, width = popup_width.saturating_sub(13));
                    term.write_str(&style(content).black().on_white().to_string())?;
                }
            }
        }

        if is_command_mode {
            let input_prefix: String = current_input.chars().take(cursor_pos).collect();
            let cursor_x = left_width + 3 + measure_text_width(&input_prefix);
            term.move_cursor_to(cursor_x, height as usize - 3)?;
            term.show_cursor()?;
        } else {
            term.hide_cursor()?;
        }

        let key = match term.read_key() {
            Ok(k) => k,
            Err(e) => {
                if e.to_string().contains("read interrupted") {
                    if is_command_mode {
                        current_input.clear();
                        history_index = None;
                        continue;
                    } else {
                        break;
                    }
                }
                return Err(anyhow::Error::from(e)).context("Failed to read key");
            }
        };

        match key {
            Key::ArrowLeft if is_command_mode => {
                if cursor_pos > 0 {
                    cursor_pos -= 1;
                }
            }
            Key::ArrowRight if is_command_mode => {
                if cursor_pos < current_input.chars().count() {
                    cursor_pos += 1;
                }
            }
            Key::ArrowUp => {
                if is_command_mode {
                    if !state.history.is_empty() {
                        let new_index = match history_index {
                            None => Some(state.history.len() - 1),
                            Some(idx) if idx > 0 => Some(idx - 1),
                            Some(_) => Some(0),
                        };
                        if let Some(idx) = new_index {
                            history_index = Some(idx);
                            current_input = state.history[idx].clone();
                            cursor_pos = current_input.chars().count();
                        }
                    }
                } else {
                    if selected_index > 0 {
                        selected_index -= 1;
                    } else {
                        selected_index = worktrees.len().saturating_sub(1);
                    }
                }
            }
            Key::ArrowDown => {
                if is_command_mode {
                    if let Some(idx) = history_index {
                        if idx < state.history.len() - 1 {
                            let next_idx = idx + 1;
                            history_index = Some(next_idx);
                            current_input = state.history[next_idx].clone();
                            cursor_pos = current_input.chars().count();
                        } else {
                            history_index = None;
                            current_input.clear();
                            cursor_pos = 0;
                        }
                    }
                } else {
                    if selected_index < worktrees.len().saturating_sub(1) {
                        selected_index += 1;
                    } else {
                        selected_index = 0;
                    }
                }
            }
            Key::Enter => {
                if is_command_mode {
                    if !current_input.is_empty() {
                        let parts: Vec<String> = current_input.split_whitespace().map(|s| s.to_string()).collect();
                        if !parts.is_empty() {
                            let mut cmd_to_execute = current_input.clone();
                            let mut parts = parts;

                            // If cmd is a number, try to get from history
                            if let Ok(index) = parts[0].parse::<usize>() {
                                if index > 0 && index <= state.history.len() {
                                    cmd_to_execute = state.history[index - 1].clone();
                                    parts = cmd_to_execute.split_whitespace().map(|s| s.to_string()).collect();
                                }
                            }

                            if parts.is_empty() {
                                continue;
                            }

                            let cmd_name = parts[0].clone();
                            let is_session_close = cmd_name == "session" && parts.get(1).map(|s| s.as_str()) == Some("close");
                            let result: Result<String> = if let Some(command) = commands.iter().find(|c| c.name() == cmd_name) {
                                if cmd_name == "space" {
                                    is_command_mode = false;
                                    if parts.len() == 1 {
                                        parts.push(worktrees[selected_index].clone());
                                    }
                                }
                                command.run(parts, &project_path)
                            } else {
                                let available: Vec<String> = commands.iter().map(|c| c.name().to_string()).collect();
                                command_history.push(format!("Unknown command: {}", cmd_name));
                                command_history.push(format!("Available commands: {}", available.join(", ")));
                                Ok("".to_string())
                            };

                            match result {
                                Err(e) => {
                                    for line in e.to_string().lines() {
                                        command_history.push(line.to_string());
                                    }
                                }
                                Ok(output) => {
                                    if cmd_name == "close" || is_session_close {
                                        is_command_mode = false;
                                        // 状態を再読み込み
                                        if let Ok(new_state) = get_project_state(&project_path) {
                                            state = new_state;
                                            worktrees = vec!["main".to_string()];
                                            worktrees.extend(state.worktrees.clone());
                                            if let Some(current_wt) = &state.current_worktree {
                                                if let Some(idx) = worktrees.iter().position(|wt| wt == current_wt) {
                                                    selected_index = idx;
                                                }
                                            } else {
                                                selected_index = 0; // main を選択
                                            }
                                        }
                                    } else {
                                        // コマンド実行に成功した場合のみ履歴に追加
                                        command_history.push(cmd_to_execute.clone());
                                        if !output.is_empty() {
                                            for line in output.lines() {
                                                command_history.push(line.to_string());
                                            }
                                        }
                                        
                                        // 状態が更新された可能性があるので再読み込みして表示を更新
                                        if let Ok(mut new_state) = get_project_state(&project_path) {
                                            // 履歴を永続化
                                            if !new_state.history.contains(&cmd_to_execute) {
                                                new_state.history.push(cmd_to_execute.clone());
                                                let _ = crate::infrastructure::project_state::save_project_state(&project_path, &new_state);
                                            } else {
                                                // 既に存在する場合でも最新として扱うために順序を入れ替える等の処理は
                                                // 今回はシンプルにするため行わないが、再取得は必要
                                            }

                                            // 状態をローカルの state にも反映
                                            state = new_state;

                                            worktrees = vec!["main".to_string()];
                                            worktrees.extend(state.worktrees.clone());
                                            if let Some(current_wt) = &state.current_worktree {
                                                if let Some(idx) = worktrees.iter().position(|wt| wt == current_wt) {
                                                    selected_index = idx;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if command_history.len() > (height as usize - 7) {
                            command_history.remove(0);
                        }
                        current_input.clear();
                        cursor_pos = 0;
                        history_index = None;
                    } else {
                        is_command_mode = false;
                        cursor_pos = 0;
                        history_index = None;
                    }
                } else {
                    is_command_mode = true;
                    cursor_pos = 0;
                    history_index = None;
                }
            }
            Key::Char(c) if is_command_mode => {
                let byte_offset: usize = current_input.chars().take(cursor_pos).map(|c| c.len_utf8()).sum();
                current_input.insert(byte_offset, c);
                cursor_pos += 1;
                history_index = None;
            }
            Key::Tab if is_command_mode => {
                let parts: Vec<&str> = current_input.split_whitespace().collect();
                if !current_input.contains(' ') {
                    // コマンド名の補完
                    let suggestions: Vec<&str> = commands.iter()
                        .map(|c| c.name())
                        .filter(|name| name.starts_with(&current_input))
                        .collect();
                    if let Some(first) = suggestions.first() {
                        current_input = first.to_string();
                        cursor_pos = current_input.chars().count();
                    }
                } else if !parts.is_empty() {
                    // 引数の補完
                    let cmd_name = parts[0];
                    if let Some(command) = commands.iter().find(|c| c.name() == cmd_name) {
                        let last_part = if current_input.ends_with(' ') { "" } else { parts.last().unwrap_or(&"") };
                        let suggestions = command.subcommands();
                        if let Some((name, _)) = suggestions.iter().find(|(name, _)| name.starts_with(last_part)) {
                            let head = if current_input.ends_with(' ') {
                                &current_input
                            } else {
                                current_input.rsplit_once(' ').map(|(h, _)| h).unwrap_or("")
                            };
                            if head.is_empty() || head.ends_with(' ') {
                                current_input = format!("{}{}", head, name);
                            } else {
                                current_input = format!("{} {}", head, name);
                            }
                            cursor_pos = current_input.chars().count();
                        }
                    }
                }
            }
            Key::Backspace if is_command_mode => {
                if cursor_pos > 0 {
                    let byte_offset: usize = current_input.chars().take(cursor_pos - 1).map(|c| c.len_utf8()).sum();
                    current_input.remove(byte_offset);
                    cursor_pos -= 1;
                }
                history_index = None;
            }
            Key::Escape if is_command_mode => {
                is_command_mode = false;
                current_input.clear();
                cursor_pos = 0;
                history_index = None;
            }
            Key::Char('q') | Key::Escape if !is_command_mode => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
