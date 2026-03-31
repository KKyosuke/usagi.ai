use console::style;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum AppMode {
    Global,
    Menu,
    Command,
    Execution,
}

impl AppMode {
    pub fn label(&self) -> &str {
        match self {
            AppMode::Global => "全体モード",
            AppMode::Menu => "メニューモード",
            AppMode::Command => "コマンドモード",
            AppMode::Execution => "実行モード",
        }
    }
}

pub fn show_rabbit(mode: AppMode) {
    let rabbit = r#"
　　　　 　/ \ / \
　　　　　(  o.o  )
　　　　　  > ^ <
    "#;
    println!("{}", style(rabbit).magenta());
    println!("---------- USAGI AI ----------");
    println!("MODE: {}", style(mode.label()).bold().cyan());
}

pub fn render_menu(
    projects: &[String],
    selected_project: usize,
) {
    println!("Use Up/Down to select, Enter to open, 'q' to quit.");
    println!("{}", style("PROJECTS").bold());
    println!("{:-<60}", "");

    for i in 0..projects.len() {
        if i == selected_project {
            println!("> {}", style(&projects[i]).cyan().bold());
        } else {
            println!("  {}", &projects[i]);
        }
    }
}
