pub mod app;
pub mod ui;
pub mod event;
pub mod executor;
pub mod completion;
pub mod history_manager;

use anyhow::Result;
use std::path::PathBuf;
use crate::presentation::tui::screen::AlternateScreenGuard;
use self::app::HopApp;

pub async fn run(project_path: PathBuf, initial_worktree: Option<String>) -> Result<()> {
    let mut app = HopApp::new(project_path, initial_worktree)?;
    
    // TUI 画面の表示
    let _guard = AlternateScreenGuard::new(app.term.clone())?;
    
    // 画面全体を一度クリア
    app.term.clear_screen()?;

    loop {
        ui::render(&app)?;
        
        if !event::handle_key(&mut app).await? {
            break;
        }
    }

    Ok(())
}
