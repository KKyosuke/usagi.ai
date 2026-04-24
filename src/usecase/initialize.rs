use anyhow::{Result, Context};
use std::fs;
use std::path::{Path, PathBuf};
use crate::domain::project::{ProjectState, Worktree};
use crate::infrastructure::{git, global_registry};

/// Initialises a new usagi project.
///
/// This use case:
/// 1. Optionally changes the working directory.
/// 2. Creates `.usagi/` and writes an initial `state.json`.
/// 3. Clones the repository into `main/`.
/// 4. Creates `usagi.config`.
/// 5. Updates (or creates) `.gitignore`.
/// 6. Registers the project in the global repository registry.
pub fn run(
    repository_url: &str,
    directory: Option<PathBuf>,
    branch: Option<String>,
) -> Result<()> {
    if let Some(dir) = directory {
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .context("Failed to create target directory")?;
        }
        std::env::set_current_dir(&dir).context(format!(
            "Failed to change directory to {}",
            dir.display()
        ))?;
    }

    println!("Initializing repository: {}", repository_url);

    let usagi_dir = Path::new(".usagi");
    if usagi_dir.exists() {
        println!(
            "Error: .usagi directory already exists. This project is already initialized."
        );
        return Ok(());
    }
    fs::create_dir_all(usagi_dir).context("Failed to create .usagi directory")?;

    let main_dir = Path::new("main");
    if !main_dir.exists() {
        println!("Cloning repository into main/...");
        git::clone(repository_url, main_dir, branch.as_deref())?;
    } else {
        println!("Warning: main/ directory already exists. Skipping clone.");
    }

    let current_branch = git::get_current_branch(main_dir)?;

    let mut state = ProjectState {
        initialized: true,
        worktrees: vec![Worktree {
            branch: current_branch.clone(),
            directory: "main".to_string(),
            default: true,
            modified_at: chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
            status: crate::domain::project::SessionStatus::Todo,
            has_upstream: git::has_upstream(main_dir).unwrap_or(false),
        }],
        current_worktree: Some(current_branch),
        last_updated: None,
        ai_model: None,
    };
    state.update_last_updated();
    let state_json =
        serde_json::to_string_pretty(&state).context("Failed to serialize project state")?;
    fs::write(usagi_dir.join("state.json"), state_json)
        .context("Failed to write state.json")?;

    let config_path = Path::new("usagi.config");
    if !config_path.exists() {
        let config_content = format!(
            "# usagi project configuration\nrepository_url = \"{}\"\n",
            repository_url
        );
        fs::write(config_path, config_content)
            .context("Failed to write usagi.config")?;
    }

    let gitignore_path = Path::new(".gitignore");
    let mut gitignore_content = String::new();
    if gitignore_path.exists() {
        gitignore_content = fs::read_to_string(gitignore_path)
            .context("Failed to read existing .gitignore")?;
    }
    if !gitignore_content.contains(".usagi/") {
        if !gitignore_content.is_empty() && !gitignore_content.ends_with('\n') {
            gitignore_content.push('\n');
        }
        gitignore_content.push_str(".usagi/\n");
        fs::write(gitignore_path, gitignore_content)
            .context("Failed to write .gitignore")?;
    }

    global_registry::register_project()?;

    println!("Project initialized successfully.");
    Ok(())
}
