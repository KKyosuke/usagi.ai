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
