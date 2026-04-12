use anyhow::Result;
use std::path::Path;
use std::process::Command as ProcessCommand;
use crate::presentation::commands::Command;

pub struct DoctorCommand;

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

    fn run(&self, _args: Vec<String>, _project_path: &Path, _current_worktree: &str, _term: &console::Term) -> Result<String> {
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

        let mut output = String::from("Checking dependencies...\n\n");
        for (name, success, info, essential) in results {
            let status = if success { 
                "✅" 
            } else if essential {
                "❌"
            } else {
                "⚠️"
            };
            let essential_str = if essential { "(Essential)" } else { "(Optional) " };
            output.push_str(&format!("{} {:10} {:12} {}\n", status, name, essential_str, info));
        }

        if output.contains("❌") {
            output.push_str("\nSome essential commands are missing. Please install them to use usagi.ai properly.\n");
        } else {
            output.push_str("\nAll essential commands are available.\n");
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
