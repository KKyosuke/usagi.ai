use anyhow::{Result, Context};
use std::fs;
use std::path::Path;
use console::{Term, Key, style};
use crate::command::init::ProjectState;

pub fn run() -> Result<()> {
    // 可愛いうさぎを表示する
    show_rabbit();

    // プロジェクトの状態を読み込む
    let state_path = Path::new(".usagi/state.json");
    if !state_path.exists() {
        println!("Project is not initialized. Please run `usagi init` first.");
        return Ok(());
    }

    let state_json = fs::read_to_string(state_path).context("Failed to read .usagi/state.json")?;
    let state: ProjectState = serde_json::from_str(&state_json).context("Failed to parse .usagi/state.json")?;

    let mut items = state.worktrees.clone();
    // main ディレクトリが存在し、リストに含まれていない場合は追加
    if Path::new("main").exists() && !items.contains(&"main".to_string()) {
        items.insert(0, "main".to_string());
    }

    if items.is_empty() {
        println!("No worktrees found. Press 'n' to create a new one, or 'q' to quit.");
    } else {
        println!("Use Up/Down to select, Enter to open, 'n' for new, 'q' to quit.");
    }

    let term = Term::stdout();
    let mut selected_index = 0;

    loop {
        // メニューの描画
        for (i, item) in items.iter().enumerate() {
            if i == selected_index {
                println!("> {}", style(item).cyan().bold());
            } else {
                println!("  {}", item);
            }
        }

        // キー入力待ち
        let key = term.read_key().context("Failed to read key")?;

        // 描画した行をクリア
        term.clear_last_lines(items.len()).context("Failed to clear lines")?;

        match key {
            Key::ArrowUp => {
                if selected_index > 0 {
                    selected_index -= 1;
                } else if !items.is_empty() {
                    selected_index = items.len() - 1;
                }
            }
            Key::ArrowDown => {
                if !items.is_empty() {
                    if selected_index < items.len() - 1 {
                        selected_index += 1;
                    } else {
                        selected_index = 0;
                    }
                }
            }
            Key::Enter => {
                if !items.is_empty() {
                    let selected = &items[selected_index];
                    println!("Opening workspace: {}", style(selected).green());
                    // 実際にはディレクトリ移動などを行う必要があるが、
                    // シェルコマンド経由でないとカレントディレクトリは変更できない。
                    // ここではメッセージを表示するにとどめる。
                    break;
                }
            }
            Key::Char('n') => {
                println!("Creating new workspace...");
                // TODO: start コマンドのインタラクティブ版などを呼び出す
                println!("Please run: usagi start <new_branch_name>");
                break;
            }
            Key::Char('q') | Key::Escape => {
                println!("Quit.");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

fn show_rabbit() {
    let rabbit = r#"
　　　　 　/ \ / \
　　　　　(  o.o  )
　　　　　 > ^ <
    "#;
    println!("{}", style(rabbit).magenta());
    println!("--- USAGI AI ---");
}
