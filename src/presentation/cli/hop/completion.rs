use crate::presentation::cli::hop::app::HopApp;

pub fn handle_tab(app: &mut HopApp) {
    if app.tab_completion_base.is_none() {
        app.tab_completion_base = Some(app.current_input.clone());
        app.suggestion_index = None;
    }

    let input = app.tab_completion_base.as_ref().unwrap().clone();
    let parts: Vec<&str> = input.split_whitespace().collect();
    let mut suggestions: Vec<String> = Vec::new();

    if !input.contains(' ') {
        // コマンド名の補完
        suggestions = app.commands.iter()
            .map(|c| c.name().to_string())
            .filter(|name| name.starts_with(&input))
            .collect();
    } else if !parts.is_empty() {
        // 引数の補完
        let cmd_name = parts[0];
        if let Some(command) = app.commands.iter().find(|c| c.name() == cmd_name) {
            let last_part = if input.ends_with(' ') { "" } else { parts.last().unwrap_or(&"") };
            suggestions = command.subcommands().into_iter()
                .filter(|(name, _)| name.starts_with(last_part))
                .map(|(name, _)| name.clone())
                .collect();
        }
    }

    if suggestions.is_empty() {
        return;
    }

    let next_idx = match app.suggestion_index {
        Some(idx) => (idx + 1) % suggestions.len(),
        None => 0,
    };
    app.suggestion_index = Some(next_idx);

    let selected = &suggestions[next_idx];

    if !input.contains(' ') {
        app.current_input = selected.clone();
    } else {
        let head = if input.ends_with(' ') {
            &input
        } else {
            input.rsplit_once(' ').map(|(h, _)| h).unwrap_or("")
        };
        if head.is_empty() || head.ends_with(' ') {
            app.current_input = format!("{}{}", head, selected);
        } else {
            app.current_input = format!("{} {}", head, selected);
        }
    }
    app.cursor_pos = app.current_input.chars().count();
}

pub fn compute_suggestions(app: &HopApp) -> (Option<String>, Vec<(String, String)>) {
    let input_to_use = if let Some(base) = &app.tab_completion_base {
        base.clone()
    } else {
        app.current_input.clone()
    };
    
    if input_to_use.is_empty() {
        return (None, Vec::new());
    }

    let parts: Vec<&str> = input_to_use.split_whitespace().collect();
    let mut suggestions: Vec<(String, String)> = Vec::new();
    let mut usage_text: Option<String> = None;

    if !input_to_use.contains(' ') {
        let mut current_suggestions: Vec<(String, String)> = app.commands.iter()
            .filter(|c| c.name().starts_with(&input_to_use))
            .map(|c| (c.name().to_string(), c.description().to_string()))
            .collect();

        if current_suggestions.len() == 1 {
            let name = current_suggestions[0].0.clone();
            if let Some(command) = app.commands.iter().find(|c| c.name() == name) {
                usage_text = command.usage(&[name.as_str()]);
                if name == input_to_use {
                    current_suggestions.clear();
                }
            }
        }
        suggestions = current_suggestions;
    } else if !parts.is_empty() {
        let cmd_name = parts[0];
        if let Some(command) = app.commands.iter().find(|c| c.name() == cmd_name) {
            let last_part = if input_to_use.ends_with(' ') { "" } else { parts.last().unwrap_or(&"") };
            let mut current_suggestions: Vec<(String, String)> = command.subcommands()
                .into_iter()
                .filter(|(name, _)| name.starts_with(last_part))
                .collect();

            if current_suggestions.len() == 1 {
                let name = current_suggestions[0].0.clone();
                let is_perfect_match = name == last_part;

                let mut check_parts = parts.clone();
                if !input_to_use.ends_with(' ') {
                    if let Some(last) = check_parts.last_mut() {
                        *last = name.as_str();
                    }
                } else {
                    if parts.iter().any(|&p| p == name) {
                        current_suggestions.clear();
                    }
                    check_parts.push(name.as_str());
                }

                if let Some(detail_usage) = command.usage(&check_parts) {
                    usage_text = Some(detail_usage);
                } else {
                    usage_text = command.usage(&parts);
                }

                if is_perfect_match {
                    current_suggestions.clear();
                }
            } else {
                usage_text = command.usage(&parts);
            }
            suggestions = current_suggestions;
        }
    }
    
    (usage_text, suggestions)
}
