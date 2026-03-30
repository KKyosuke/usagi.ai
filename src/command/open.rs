use anyhow::{Result, Context, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use console::{Term, Key, style};
use directories::ProjectDirs;
use crate::command::init::{ProjectState, Repositories};
use inquire::Text;

pub fn run() -> Result<()> {
    // 可愛いうさぎを表示する
    show_rabbit();

    let mut repos = get_repositories()?;
    
    let mut selected_index = 0;
    let term = Term::stdout();

    loop {
        let mut items: Vec<String> = repos.iter().map(|path| {
            let config_path = path.join("usagi.config");
            let status = if !path.exists() {
                style("(Missing)").red()
            } else if config_path.exists() {
                style("(Active)").green()
            } else {
                style("(No config)").red()
            };
            format!("{} {}", path.display(), status)
        }).collect();

        items.push(format!("+ {}", style("New project").yellow().bold()));

        println!("Use Up/Down to select project, Enter to open, 'q' to quit.");
        for (i, item) in items.iter().enumerate() {
            if i == selected_index {
                println!("> {}", style(item).cyan().bold());
            } else {
                println!("  {}", item);
            }
        }

        let key = term.read_key().context("Failed to read key")?;
        term.clear_last_lines(items.len() + 1).context("Failed to clear lines")?;

        match key {
            Key::ArrowUp => {
                if selected_index > 0 {
                    selected_index -= 1;
                } else {
                    selected_index = items.len() - 1;
                }
            }
            Key::ArrowDown => {
                if selected_index < items.len() - 1 {
                    selected_index += 1;
                } else {
                    selected_index = 0;
                }
            }
            Key::Enter => {
                if selected_index < repos.len() {
                    let selected_path = &repos[selected_index];
                    let config_path = selected_path.join("usagi.config");
                    
                    if !selected_path.exists() || !config_path.exists() {
                        if show_delete_modal(selected_path)? {
                            repos.remove(selected_index);
                            save_repositories(&repos)?;
                            if selected_index >= repos.len() && !repos.is_empty() {
                                selected_index = repos.len() - 1;
                            }
                        }
                        continue;
                    }
                    return show_worktree_menu(selected_path);
                } else {
                    // New project
                    let repo_url = Text::new("Repository URL:").prompt()?;
                    return crate::command::init::run(&repo_url);
                }
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

fn get_repositories() -> Result<Vec<PathBuf>> {
    let proj_dirs = ProjectDirs::from("", "", "usagi")
        .ok_or_else(|| anyhow!("Could not determine home directory"))?;
    let data_dir = proj_dirs.data_dir();
    let repo_json_path = data_dir.join("repositories.json");

    if repo_json_path.exists() {
        let content = fs::read_to_string(&repo_json_path).context("Failed to read repositories.json")?;
        let repos: Repositories = serde_json::from_str(&content).context("Failed to parse repositories.json")?;
        Ok(repos.repositories)
    } else {
        Ok(vec![])
    }
}

fn save_repositories(repos: &[PathBuf]) -> Result<()> {
    let proj_dirs = ProjectDirs::from("", "", "usagi")
        .ok_or_else(|| anyhow!("Could not determine home directory"))?;
    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir).context("Failed to create data directory")?;

    let repo_json_path = data_dir.join("repositories.json");
    let repos_struct = Repositories {
        repositories: repos.to_vec(),
    };
    let content = serde_json::to_string_pretty(&repos_struct).context("Failed to serialize repositories")?;
    fs::write(repo_json_path, content).context("Failed to write repositories.json")?;

    Ok(())
}

fn show_delete_modal(path: &Path) -> Result<bool> {
    let term = Term::stdout();
    let mut delete_selected = true;

    loop {
        println!("Project config is missing or directory not found: {}", style(path.display()).yellow());
        println!("Do you want to delete this project from list or keep it?");
        
        let delete_btn = if delete_selected {
            style("[ Delete ]").cyan().bold()
        } else {
            style("[ Delete ]").white()
        };

        let keep_btn = if !delete_selected {
            style("[ Keep ]").cyan().bold()
        } else {
            style("[ Keep ]").white()
        };

        println!("  {}     {}", delete_btn, keep_btn);

        let key = term.read_key().context("Failed to read key")?;
        term.clear_last_lines(3).context("Failed to clear lines")?;

        match key {
            Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => {
                delete_selected = !delete_selected;
            }
            Key::Enter => {
                return Ok(delete_selected);
            }
            Key::Escape | Key::Char('q') => {
                return Ok(false);
            }
            _ => {}
        }
    }
}

fn show_worktree_menu(project_path: &Path) -> Result<()> {
    let state_path = project_path.join(".usagi/state.json");
    if !state_path.exists() {
        println!("Project state is missing in {}. Please ensure it's a valid usagi project.", project_path.display());
        return Ok(());
    }

    let state_json = fs::read_to_string(state_path).context("Failed to read project state")?;
    let state: ProjectState = serde_json::from_str(&state_json).context("Failed to parse project state")?;

    let mut items = state.worktrees.clone();
    // main ディレクトリが存在し、リストに含まれていない場合は追加
    if project_path.join("main").exists() && !items.contains(&"main".to_string()) {
        items.insert(0, "main".to_string());
    }

    let term = Term::stdout();
    let mut selected_index = 0;

    loop {
        // ヘルプメッセージの表示
        if items.is_empty() {
            println!("No worktrees found. Press 'n' for new, 'q' to quit.");
        } else {
            println!("Project: {}", style(project_path.display()).green());
            println!("Use Up/Down to select, Enter to open, 'n' for new, 'q' to quit.");
        }

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
        term.clear_last_lines(items.len() + if items.is_empty() { 1 } else { 2 }).context("Failed to clear lines")?;

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
                    println!("Opening workspace: {} in {}", style(selected).green(), project_path.display());
                    break;
                }
            }
            Key::Char('n') => {
                println!("Creating new workspace...");
                // TODO: start コマンドのインタラクティブ版などを呼び出す
                println!("Please run: usagi start <new_branch_name> (in {})", project_path.display());
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
    println!("---------- USAGI AI ----------");
}
