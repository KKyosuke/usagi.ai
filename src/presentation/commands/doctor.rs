use anyhow::Result;
use std::path::Path;
use std::process::Command as ProcessCommand;
use async_trait::async_trait;
use console::{style, Term};
use crate::presentation::commands::Command;
use crate::presentation::tui::home::ui::animate_rabbit;

pub struct DoctorCommand;

#[async_trait]
impl Command for DoctorCommand {
    fn name(&self) -> &str {
        "doctor"
    }

    fn description(&self) -> &str {
        "Check if necessary commands are installed"
    }

    fn help(&self) -> &str {
        "Checks for required dependencies like git.
Usage: doctor"
    }

    async fn run(&self, _args: Vec<String>, _project_path: &Path, _current_worktree: &str, term: &Term) -> Result<String> {
        animate_rabbit(term, 1200, false);
        Ok(self.check_all())
    }
}

impl DoctorCommand {
    pub fn check_all(&self) -> String {
        let mut results = Vec::new();

        // Essential commands
        results.push(check_command("git", &["--version"], true));
        
        // Shell
        if cfg!(windows) {
            results.push(check_command("cmd.exe", &["/c", "ver"], true));
        } else {
            results.push(check_command("bash", &["--version"], true));
        }

        // Common development tools (Optional)
        results.push(check_command("node", &["--version"], false));
        results.push(check_command("npm", &["--version"], false));
        results.push(check_command("python3", &["--version"], false));
        results.push(check_command("python", &["--version"], false));
        results.push(check_command("aws", &["--version"], false));

        let mut output = format!("{}\n\n", style("🐰 USAGI DOCTOR is checking your system... 🐰").magenta().bold());
        for (name, success, info, essential) in results {
            let status_icon = if success { 
                "🥕" 
            } else if essential {
                "❌"
            } else {
                "🐾"
            };
            
            let padded_name = format!("{:<10}", name);
            let name_str = if success {
                style(padded_name).green().bold().to_string()
            } else if essential {
                style(padded_name).red().bold().to_string()
            } else {
                style(padded_name).yellow().to_string()
            };

            let label = if essential { "(Essential)" } else { "(Optional) " };
            let essential_label = if essential { 
                style(label).cyan().to_string()
            } else { 
                style(label).dim().to_string() 
            };
            
            output.push_str(&format!("{} {} {} {}\n", status_icon, name_str, essential_label, style(info).dim()));
        }

        if output.contains("❌") {
            output.push_str(&format!(
                "\n{}\n", 
                style("😭 Oh no! Some essential carrots are missing. Please install them to let usagi jump!").red().bold()
            ));
        } else {
            output.push_str(&format!(
                "\n{}\n", 
                style("✨ Everything looks fluffy! Usagi is ready to hop! ✨").green().bold()
            ));
        }

        output
    }
}

fn check_command(name: &'static str, args: &[&str], essential: bool) -> (&'static str, bool, String, bool) {
    match ProcessCommand::new(name).args(args).output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let version = version.lines().next().unwrap_or("").to_string();
            (name, true, version, essential)
        }
        _ => (name, false, "Not found".to_string(), essential),
    }
}
