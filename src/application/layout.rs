use console::style;

pub fn show_rabbit() {
    let rabbit = r#"
　　　　 　/ \ / \
　　　　　(  o.o  )
　　　　　  > ^ <
    "#;
    println!("{}", style(rabbit).magenta());
    println!("---------- USAGI AI ----------");
}

pub fn render_menu(
    projects: &[String],
    selected_project: usize,
    worktrees: &[String],
    selected_worktree: Option<usize>,
    active_column: usize,
) {
    println!("Use Up/Down to select, Left/Right to switch columns, Enter to open, 'q' to quit.");
    println!("{:<60} | {}", style("PROJECTS").bold(), style("WORKTREES").bold());
    println!("{:-<60}-+-{:-<20}", "", "");

    let max_len = projects.len().max(worktrees.len());
    for i in 0..max_len {
        let proj = if i < projects.len() {
            if i == selected_project && active_column == 0 {
                format!("> {}", style(&projects[i]).cyan().bold())
            } else if i == selected_project {
                format!("  {}", style(&projects[i]).cyan())
            } else {
                format!("  {}", &projects[i])
            }
        } else {
            "".to_string()
        };

        let wt = if i < worktrees.len() {
            if Some(i) == selected_worktree && active_column == 1 {
                format!("> {}", style(&worktrees[i]).cyan().bold())
            } else if Some(i) == selected_worktree {
                format!("  {}", style(&worktrees[i]).cyan())
            } else {
                format!("  {}", &worktrees[i])
            }
        } else if worktrees.is_empty() && i == 0 {
            style("(No worktrees found. Press 'n' to create one.)").dim().to_string()
        } else {
            "".to_string()
        };

        // Note: alignment might be off due to ANSI codes, but we'll try our best.
        // A better way would be to calculate visible length, but console doesn't seem to provide it directly.
        println!("{:<60} | {}", proj, wt);
    }
}
