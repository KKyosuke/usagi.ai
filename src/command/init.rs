use anyhow::{Result, Context};
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct ProjectState {
    initialized: bool,
    worktrees: Vec<String>,
}

pub fn run(repository_url: &str) -> Result<()> {
    println!("Initializing repository: {}", repository_url);

    // 1. .usagi/ ディレクトリの作成
    let usagi_dir = Path::new(".usagi");
    if usagi_dir.exists() {
        println!("Error: .usagi directory already exists. This project is already initialized.");
        return Ok(());
    }
    fs::create_dir_all(usagi_dir).context("Failed to create .usagi directory")?;

    // 2. .usagi/state.json の作成
    let state = ProjectState {
        initialized: true,
        worktrees: vec![],
    };
    let state_json = serde_json::to_string_pretty(&state).context("Failed to serialize project state")?;
    fs::write(usagi_dir.join("state.json"), state_json).context("Failed to write state.json")?;

    // 3. main/ へのクローン
    let main_dir = Path::new("main");
    if !main_dir.exists() {
        println!("Cloning repository into main/...");
        git2::Repository::clone(repository_url, main_dir).context("Failed to clone repository")?;
    } else {
        println!("Warning: main/ directory already exists. Skipping clone.");
    }

    // 4. usagi.config の作成
    let config_path = Path::new("usagi.config");
    if !config_path.exists() {
        let config_content = format!(
            "# usagi project configuration\nrepository_url = \"{}\"\n",
            repository_url
        );
        fs::write(config_path, config_content).context("Failed to write usagi.config")?;
    }

    // 5. .gitignore の作成
    let gitignore_path = Path::new(".gitignore");
    let mut gitignore_content = String::new();
    if gitignore_path.exists() {
        gitignore_content = fs::read_to_string(gitignore_path).context("Failed to read existing .gitignore")?;
    }

    if !gitignore_content.contains(".usagi/") {
        if !gitignore_content.is_empty() && !gitignore_content.ends_with('\n') {
            gitignore_content.push('\n');
        }
        gitignore_content.push_str(".usagi/\n");
        fs::write(gitignore_path, gitignore_content).context("Failed to write .gitignore")?;
    }

    println!("Project initialized successfully.");
    Ok(())
}
