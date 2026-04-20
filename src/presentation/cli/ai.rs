use anyhow::{Context, Result};
use console::style;
use std::fs;
use std::process::Command;
use inquire::Select;

#[derive(Clone)]
struct ModelOption {
    name: &'static str,
    description: &'static str,
    url: &'static str,
    filename: &'static str,
}

impl std::fmt::Display for ModelOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.name, self.description)
    }
}

pub fn run_install() -> Result<()> {
    let term = console::Term::stdout();

    let user_dirs = directories::UserDirs::new().context("Could not find user home directory")?;
    let models_dir = user_dirs.home_dir().join(".usagi").join("models");
    fs::create_dir_all(&models_dir).context("Failed to create models directory")?;
    
    let base_models = vec![
        ModelOption {
            name: "Gemma 2 (2B) Q4_K_M",
            description: "Fast & lightweight. Best for laptops (1.6GB)",
            url: "https://huggingface.co/bartowski/gemma-2-2b-it-GGUF/resolve/main/gemma-2-2b-it-Q4_K_M.gguf",
            filename: "gemma-2-2b-it-Q4_K_M.gguf",
        },
        ModelOption {
            name: "Gemma 4 E4B Instruct Q4_K_M",
            description: "Unsloth tuned Gemma 4 E4B (approx 2.5GB)",
            url: "https://huggingface.co/unsloth/gemma-4-E4B-it-GGUF/resolve/main/gemma-4-E4B-it-Q4_K_M.gguf",
            filename: "gemma-4-E4B-it-Q4_K_M.gguf",
        },
        ModelOption {
            name: "Llama-3.1 (8B) Instruct Q4_K_M",
            description: "High quality general model (4.9GB)",
            url: "https://huggingface.co/bartowski/Meta-Llama-3.1-8B-Instruct-GGUF/resolve/main/Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf",
            filename: "Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf",
        },
        ModelOption {
            name: "Phi-3 Mini (3.8B) Instruct Q4_K_M",
            description: "Microsoft's efficient coding/logic model (2.4GB)",
            url: "https://huggingface.co/bartowski/Phi-3-mini-4k-instruct-GGUF/resolve/main/Phi-3-mini-4k-instruct-Q4_K_M.gguf",
            filename: "Phi-3-mini-4k-instruct-Q4_K_M.gguf",
        },
    ];

    struct DisplayModelOption {
        option: ModelOption,
        is_installed: bool,
    }

    impl std::fmt::Display for DisplayModelOption {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.is_installed {
                write!(f, "{} - {} {}", self.option.name, self.option.description, console::style("[Installed]").green())
            } else {
                write!(f, "{} - {}", self.option.name, self.option.description)
            }
        }
    }

    let models: Vec<DisplayModelOption> = base_models.into_iter().map(|m| {
        let path = models_dir.join(m.filename);
        let is_installed = path.exists() && fs::metadata(&path).map(|md| md.len() > 1_000_000_000).unwrap_or(false);
        DisplayModelOption {
            option: m,
            is_installed,
        }
    }).collect();

    let selected_wrapper = Select::new("Which model would you like to install?", models)
        .prompt()
        .context("Model selection cancelled")?;
        
    let selected_model = selected_wrapper.option;

    if selected_wrapper.is_installed {
        term.write_line(&format!("{}", style(format!("✨ {} is already installed in ~/.usagi/models/", selected_model.filename)).green()))?;
        return Ok(());
    }

    term.write_line(&format!("\n{}", style(format!("🐰 Installing {}...", selected_model.name)).cyan().bold()))?;
    
    let target_path = models_dir.join(selected_model.filename);


    term.write_line(&format!("Downloading to {:?} (this will take a few minutes)...", target_path))?;

    let status = Command::new("curl")
        .arg("-#")
        .arg("-L")
        .arg("-o")
        .arg(&target_path)
        .arg(selected_model.url)
        .status()
        .context("Failed to execute curl command")?;

    if status.success() {
        term.write_line(&format!("\n{}", style("🥕 Download complete!").green().bold()))?;
        term.write_line(&format!(
            "{}\n{}",
            style("You can now use the model inside the 'usagi hop' REPL.").dim(),
            style(format!("Example: ai --model ~/.usagi/models/{} \"Hello\"", selected_model.filename)).dim(),
        ))?;
    } else {
        term.write_line(&format!("\n{}", style("❌ Download failed. Please check your internet connection.").red().bold()))?;
    }

    Ok(())
}
