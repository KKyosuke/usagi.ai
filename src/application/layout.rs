use console::{Term, style};
use anyhow::Result;

pub struct AlternateScreenGuard {
    pub term: Term,
}

impl AlternateScreenGuard {
    pub fn new(term: Term) -> Result<Self> {
        term.write_str("\x1b[?1049h")?;
        Ok(Self { term })
    }
}

impl Drop for AlternateScreenGuard {
    fn drop(&mut self) {
        let _ = self.term.write_str("\x1b[?1049l");
        let _ = self.term.show_cursor();
    }
}

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

pub fn show_rabbit(term: &Term) {
    let rabbit = r#"
　　　　 　/ \ / \
　　　　　(  o.o  )
　　　　　  > ^ <
    "#;
    let _ = term.write_line(&style(rabbit).magenta().to_string());
    let _ = term.write_line("---------- USAGI AI ----------");
}

pub fn render_side_menu(
    term: &Term,
    projects: &[String],
    selected_project: usize,
) {
    let _ = term.write_line("Use Up/Down to select, Enter to open, 'q' to quit.");
    let _ = term.write_line(&style("PROJECTS").bold().to_string());
    let _ = term.write_line(&format!("{:-<60}", ""));

    for i in 0..projects.len() {
        if i == selected_project {
            let _ = term.write_line(&format!("> {}", style(&projects[i]).cyan().bold()));
        } else {
            let _ = term.write_line(&format!("  {}", &projects[i]));
        }
    }
}
