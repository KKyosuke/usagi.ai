use anyhow::{Context, Result};
use clap::Parser;
use console::style;
use std::path::Path;

use crate::presentation::commands::Command;

use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::context::params::LlamaContextParams;

pub struct AiCommand;

#[derive(Parser, Debug)]
#[command(name = "ai")]
struct AiArgs {
    /// The prompt to send to the AI
    #[arg(required = false)]
    prompt: Vec<String>,

    /// Path to the GGUF model file
    #[arg(short, long)]
    model: Option<String>,

    /// Set the provided model as the default for this project
    #[arg(long)]
    set_model: bool,
}

impl Command for AiCommand {
    fn name(&self) -> &str {
        "ai"
    }

    fn description(&self) -> &str {
        "Ask AI a question using a local llama.cpp model"
    }

    fn help(&self) -> &str {
        "Usage: ai <prompt> [--model <path>]\n       ai --set-model\nSet USAGI_AI_MODEL env var to specify global default model."
    }

    fn run(&self, args: Vec<String>, _project_path: &Path, _current_worktree: &str, term: &console::Term) -> Result<String> {
        let mut cli_args = vec!["ai".to_string()];
        cli_args.extend(args);

        let parsed = match AiArgs::try_parse_from(&cli_args) {
            Ok(opts) => opts,
            Err(e) => return Ok(e.to_string()),
        };

        // Handle setting default model
        if parsed.set_model {
            let user_dirs = directories::UserDirs::new().context("Could not find user home directory")?;
            let models_dir = user_dirs.home_dir().join(".usagi").join("models");
            
            let mut available_models = Vec::new();
            if models_dir.exists() {
                for entry in std::fs::read_dir(&models_dir).unwrap().flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "gguf") {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            available_models.push(name.to_string());
                        }
                    }
                }
            }
            
            if available_models.is_empty() {
                return Ok(format!("{}", style("No models found in ~/.usagi/models/. Please run 'usagi ai install' first.").red()));
            }
            
            if let Ok(selected) = inquire::Select::new("Which model would you like to set as default for this project?", available_models).prompt() {
                let full_path = models_dir.join(&selected).to_string_lossy().to_string();
                if let Ok(mut state) = crate::infrastructure::project_state::get_project_state(_project_path) {
                    state.ai_model = Some(full_path.clone());
                    if let Err(e) = crate::infrastructure::project_state::save_project_state(_project_path, &state) {
                        return Ok(format!("{}", style(format!("Failed to save state: {}", e)).red()));
                    }
                    return Ok(format!("{}", style(format!("Default AI model set to: {}", selected)).green()));
                } else {
                    return Ok(format!("{}", style("Failed to read project state.").red()));
                }
            } else {
                return Ok(format!("{}", style("Model selection cancelled.").yellow()));
            }
        }

        if parsed.prompt.is_empty() {
            return Ok(format!("{}", style("Error: no prompt provided.").red()));
        }

        // Format prompt for Gemma
        let prompt_text = format!("<start_of_turn>user\n{}<end_of_turn>\n<start_of_turn>model\n", parsed.prompt.join(" "));
        
        let state_model = crate::infrastructure::project_state::get_project_state(_project_path).ok().and_then(|s| s.ai_model);
        
        let mut model_path = match parsed.model.or(state_model).or_else(|| std::env::var("USAGI_AI_MODEL").ok()) {
            Some(p) => p,
            None => {
                return Ok(format!(
                    "{}\n{}",
                    style("No model specified.").red(),
                    style("Please provide --model, use --set-model, or set USAGI_AI_MODEL environment variable.").yellow()
                ));
            }
        };

        if model_path.starts_with("~/") {
            if let Some(user_dirs) = directories::UserDirs::new() {
                model_path = model_path.replacen("~", user_dirs.home_dir().to_str().unwrap(), 1);
            }
        }

        // Initialize llama backend
        let backend = LlamaBackend::init()?;
        term.write_line(&format!("{}", style("Loading model...").dim()))?;

        let model_params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
            .context("Failed to load model")?;

        let ctx_params = LlamaContextParams::default();
        let mut ctx = model.new_context(&backend, ctx_params)
            .context("Failed to create context")?;

        // Format prompt as needed, here we just pass the text
        let prompt_tokens = model.str_to_token(&prompt_text, llama_cpp_2::model::AddBos::Always)
            .context("Failed to tokenize prompt")?;

        let max_context_size = ctx.n_ctx() as usize;
        let max_tokens_list_size = max_context_size.saturating_sub(1);
        if prompt_tokens.len() > max_tokens_list_size {
            return Ok(format!("{}", style("Error: prompt is too long!").red()));
        }

        let mut batch = LlamaBatch::new(512, 1);
        let last_index = prompt_tokens.len() - 1;
        
        for (i, token) in prompt_tokens.into_iter().enumerate() {
            let is_last = i == last_index;
            let _ = batch.add(token, i as i32, &[0], is_last);
        }

        ctx.decode(&mut batch).context("Failed to decode prompt")?;

        let mut n_cur = batch.n_tokens();
        let mut output_str = String::new();
        term.write_str("\n")?;

        // Simple greedy decoding loop
        while n_cur <= model.n_ctx_train().try_into().unwrap_or(i32::MAX) {
            let candidates = ctx.candidates_ith(batch.n_tokens() - 1);
            let next_token_data = candidates
                .max_by(|a, b| a.logit().partial_cmp(&b.logit()).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap();
            let next_token = next_token_data.id();

            if next_token == model.token_eos() {
                break;
            }

            #[allow(deprecated)]
            if let Ok(piece) = model.token_to_bytes(next_token, llama_cpp_2::model::Special::Tokenize) {
                let token_str = String::from_utf8_lossy(&piece).to_string();
                output_str.push_str(&token_str);
                let _ = term.write_str(&token_str);
                let _ = term.flush();
            }

            batch.clear();
            let _ = batch.add(next_token, n_cur, &[0], true);
            ctx.decode(&mut batch).context("Failed to decode token")?;
            n_cur += 1;
        }

        term.write_str("\n")?;
        Ok(format!("\n{}", style("AI generation completed.").dim()))
    }
}
