use anyhow::{Result, Context, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;

pub use crate::application::init::{ProjectState, Repositories};

pub fn run(repository_url: &str, directory: Option<PathBuf>, branch: Option<String>) -> Result<()> {
    if let Some(dir) = directory {
        if !dir.exists() {
            fs::create_dir_all(&dir).context("Failed to create target directory")?;
        }
        std::env::set_current_dir(&dir).context(format!("Failed to change directory to {}", dir.display()))?;
    }

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
        let mut builder = git2::build::RepoBuilder::new();
        if let Some(ref b) = branch {
            builder.branch(b);
        }
        builder.clone(repository_url, main_dir).context("Failed to clone repository")?;
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

    // 6. 共通データディレクトリへの登録
    register_project()?;

    println!("Project initialized successfully.");
    Ok(())
}

fn register_project() -> Result<()> {
    let proj_dirs = ProjectDirs::from("", "", "usagi")
        .ok_or_else(|| anyhow!("Could not determine home directory"))?;
    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir).context("Failed to create data directory")?;

    let repo_json_path = data_dir.join("repositories.json");
    let mut repos = if repo_json_path.exists() {
        let content = fs::read_to_string(&repo_json_path).context("Failed to read repositories.json")?;
        serde_json::from_str(&content).context("Failed to parse repositories.json")?
    } else {
        Repositories::default()
    };

    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    if !repos.repositories.contains(&current_dir) {
        repos.repositories.push(current_dir);
        let content = serde_json::to_string_pretty(&repos).context("Failed to serialize repositories")?;
        fs::write(repo_json_path, content).context("Failed to write repositories.json")?;
    }

    Ok(())
}
