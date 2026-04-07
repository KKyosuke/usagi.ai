/// The current interaction mode of the TUI.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum AppMode {
    Global,
    SideMenu,
    Command,
    Execution,
}

impl AppMode {
    pub fn label(&self) -> &str {
        match self {
            AppMode::Global => "全体モード",
            AppMode::SideMenu => "サイドメニューモード",
            AppMode::Command => "コマンドモード",
            AppMode::Execution => "実行モード",
        }
    }
}
