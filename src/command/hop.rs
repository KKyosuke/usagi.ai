use anyhow::{Result, Context, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use console::{Term, Key, style};
use crate::command::init::ProjectState;

pub fn run(project_path: PathBuf, initial_worktree: Option<String>) -> Result<()> {
    // 0. プロジェクトディレクトリへ移動
    if !project_path.exists() {
        return Err(anyhow!("Project path does not exist: {}", project_path.display()));
    }
    std::env::set_current_dir(&project_path).context(format!("Failed to change directory to {}", project_path.display()))?;

    // 1. 初期化チェック
    let usagi_dir = Path::new(".usagi");
    let state_path = usagi_dir.join("state.json");
    if !usagi_dir.exists() || !state_path.exists() {
        return Err(anyhow!("Error: Not an initialized directory. Please run `usagi init` first."));
    }

    // 2. ProjectState の読み込み
    let state_json = fs::read_to_string(&state_path).context("Failed to read project state")?;
    let state: ProjectState = serde_json::from_str(&state_json).context("Failed to parse project state")?;

    // 3. ワークツリー一覧の作成 (main + state.worktrees)
    let mut worktrees = vec!["main".to_string()];
    worktrees.extend(state.worktrees.clone());

    // 4. TUI 画面の表示
    let term = Term::stdout();
    let mut selected_index = 0;

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
        term.write_line(&format!("{}", style("----- USAGI TERMINAL -----").magenta().bold()))?;

        // 左右分割描画
        for i in 0..(height as usize - 5) {
            let left_content = if i < worktrees.len() {
                let wt = &worktrees[i];
                if i == selected_index {
                    format!("> {}", style(wt).cyan().bold())
                } else {
                    format!("  {}", wt)
                }
            } else {
                "".to_string()
            };

            // 左側の幅を調整
            let left_display = format!("{:<width$}", left_content, width = left_width);
            
            // 右側の表示内容 (履歴)
            let right_content = if i == 0 {
                format!("Welcome to usagi terminal! (Workspace: {})", worktrees[selected_index])
            } else {
                let history_idx = i.saturating_sub(1);
                if history_idx < command_history.len() {
                    format!("$ {}", command_history[history_idx])
                } else {
                    "".to_string()
                }
            };
            
            let right_display = format!("{:<width$}", right_content, width = right_width);

            term.write_line(&format!("{} | {}", left_display, right_display))?;
        }

        // 入力欄
        term.move_cursor_to(0, height as usize - 4)?;
        term.write_line(&format!("{:-<width$}", "", width = width as usize))?;
        term.write_line(&format!("{:<left_width$} | $ ", "COMMAND", left_width = left_width))?;

        // 下部ヘルプ
        term.move_cursor_to(0, height as usize - 2)?;
        term.write_line(&format!("{}", style("Use Up/Down to select, Enter to type command, 'q' to quit.").dim()))?;

        let key = term.read_key().context("Failed to read key")?;

        match key {
            Key::ArrowUp => {
                if selected_index > 0 {
                    selected_index -= 1;
                } else {
                    selected_index = worktrees.len().saturating_sub(1);
                }
            }
            Key::ArrowDown => {
                if selected_index < worktrees.len().saturating_sub(1) {
                    selected_index += 1;
                } else {
                    selected_index = 0;
                }
            }
            Key::Enter => {
                // コマンド入力モード (簡易実装)
                term.move_cursor_to(left_width as usize + 5, height as usize - 3)?;
                term.show_cursor()?;
                let mut input = String::new();
                loop {
                    let k = term.read_key()?;
                    match k {
                        Key::Enter => break,
                        Key::Char(c) => {
                            input.push(c);
                            print!("{}", c);
                        }
                        Key::Backspace => {
                            if !input.is_empty() {
                                input.pop();
                                print!("\x08 \x08");
                            }
                        }
                        _ => {}
                    }
                }
                term.hide_cursor()?;
                if !input.is_empty() {
                    command_history.push(input);
                    if command_history.len() > (height as usize - 7) {
                        command_history.remove(0);
                    }
                }
            }
            Key::Char('q') | Key::Escape => {
                term.clear_screen()?;
                println!("Hop exited.");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
