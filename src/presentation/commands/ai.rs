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
use std::sync::{Mutex, OnceLock};

struct AiState {
    backend: Option<&'static LlamaBackend>,
    model: Option<&'static LlamaModel>,
    context: Option<llama_cpp_2::context::LlamaContext<'static>>,
    n_cur: usize,
}

unsafe impl Send for AiState {}
unsafe impl Sync for AiState {}

static AI_STATE: OnceLock<Mutex<AiState>> = OnceLock::new();

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
        "Usage: ai <prompt> [--model <path>]\n       ai chat\n       ai --set-model\nSet USAGI_AI_MODEL env var to specify global default model."
    }

    fn subcommands(&self) -> Vec<(String, String)> {
        vec![
            ("chat".to_string(), "Start an interactive AI chat session".to_string()),
        ]
    }

    fn run(&self, args: Vec<String>, _project_path: &Path, _current_worktree: &str, term: &console::Term) -> Result<String> {
        let parsed = match AiArgs::try_parse_from(&args) {
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

        let is_chat_turn = parsed.prompt.get(0).map(|s| s.as_str()) == Some("chat-turn");
        let prompt_input = if is_chat_turn {
            parsed.prompt[1..].join(" ")
        } else {
            parsed.prompt.join(" ")
        };

        if prompt_input.is_empty() && !is_chat_turn {
            return Ok(format!("{}", style("Error: no prompt provided.").red()));
        }

        // Format prompt for Gemma
        let prompt_text = if is_chat_turn {
            format!("<start_of_turn>user\n{}<end_of_turn>\n<start_of_turn>model\n", prompt_input)
        } else {
            format!("<start_of_turn>user\n{}<end_of_turn>\n<start_of_turn>model\n", prompt_input)
        };
        
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
        if is_chat_turn {
            let state_mutex = AI_STATE.get_or_init(|| Mutex::new(AiState {
                backend: None,
                model: None,
                context: None,
                n_cur: 0,
            }));
            let mut state = state_mutex.lock().unwrap();

            if state.backend.is_none() {
                let mut backend_box = Box::new(LlamaBackend::init()?);
                backend_box.void_logs();
                let backend = Box::leak(backend_box);
                let model_params = LlamaModelParams::default();
                let model = Box::leak(Box::new(LlamaModel::load_from_file(backend, &model_path, &model_params).context("Failed to load model")?));
                let ctx_params = LlamaContextParams::default();
                let ctx = model.new_context(backend, ctx_params).context("Failed to create context")?;
                
                state.backend = Some(backend);
                state.model = Some(model);
                state.context = Some(ctx);
                state.n_cur = 0;
            }

            let model = state.model.unwrap();
            let mut n_cur = state.n_cur;
            let ctx = state.context.as_mut().unwrap();
            let max_context_size = ctx.n_ctx() as usize;

            let add_bos = if n_cur == 0 { llama_cpp_2::model::AddBos::Always } else { llama_cpp_2::model::AddBos::Never };
            let prompt_tokens = model.str_to_token(&prompt_text, add_bos).context("Tokenize failed")?;

            if n_cur + prompt_tokens.len() > max_context_size.saturating_sub(1) {
                return Ok(format!("{}", style("Error: context limit reached. Please type 'exit' and start a new session.").red()));
            }

            let mut batch = LlamaBatch::new(512, 1);
            let last_index = prompt_tokens.len().saturating_sub(1);
            for (i, token) in prompt_tokens.into_iter().enumerate() {
                let is_last = i == last_index;
                let _ = batch.add(token, n_cur as i32, &[0], is_last);
                n_cur += 1;
            }

            ctx.decode(&mut batch).context("Decode failed")?;
            
            let mut output_str = String::new();
            loop {
                let candidates = ctx.candidates_ith(batch.n_tokens() - 1);
                let next_token_data = candidates
                    .max_by(|a: &llama_cpp_2::token::data::LlamaTokenData, b: &llama_cpp_2::token::data::LlamaTokenData| a.logit().partial_cmp(&b.logit()).unwrap_or(std::cmp::Ordering::Equal))
                    .unwrap();
                let next_token = next_token_data.id();

                if next_token == model.token_eos() || next_token == model.token_nl() || next_token == model.token_bos() {
                    break;
                }

                if let Ok(piece) = model.token_to_bytes(next_token, llama_cpp_2::model::Special::Tokenize) {
                    let token_str = String::from_utf8_lossy(&piece).to_string();
                    output_str.push_str(&token_str);
                    if output_str.contains("<end_of_turn>") || output_str.contains("<start_of_turn>") {
                        output_str = output_str.replace("<end_of_turn>", "").replace("<start_of_turn>", "");
                        break;
                    }
                }

                batch.clear();
                let _ = batch.add(next_token, n_cur as i32, &[0], true);
                ctx.decode(&mut batch).context("Decode failed")?;
                n_cur += 1;
                
                if n_cur >= max_context_size {
                    break;
                }
            }
            
            let end_turn_tokens = model.str_to_token("<end_of_turn>\n", llama_cpp_2::model::AddBos::Never).unwrap_or_default();
            if !end_turn_tokens.is_empty() {
                batch.clear();
                let length = end_turn_tokens.len();
                for (i, token) in end_turn_tokens.into_iter().enumerate() {
                    let is_last = i == length - 1;
                    let _ = batch.add(token, n_cur as i32, &[0], is_last);
                    n_cur += 1;
                }
                let _ = ctx.decode(&mut batch);
            }

            state.n_cur = n_cur;
            return Ok(output_str.trim().to_string());
        }


        // Initialize llama backend
        let mut backend = LlamaBackend::init()?;
        backend.void_logs();
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
