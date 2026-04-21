use anyhow::Result;
use std::path::PathBuf;
use console::{measure_text_width, strip_ansi_codes};
use crate::domain::project::ProjectHistory;
use crate::infrastructure::project_history::{get_project_history, save_project_history};

pub struct HistoryManager {
    project_path: PathBuf,
    pub input_history: ProjectHistory,
    pub input_index: Option<usize>,
    pub terminal_lines: Vec<String>,
}

impl HistoryManager {
    pub fn new(project_path: PathBuf, max_lines: usize) -> Result<Self> {
        let input_history = get_project_history(&project_path).unwrap_or_default();
        
        let mut terminal_lines: Vec<String> = input_history.history.iter().filter(|s| !s.trim().is_empty()).cloned().collect();
        if terminal_lines.len() > max_lines {
            let skip_count = terminal_lines.len() - max_lines;
            terminal_lines = terminal_lines.into_iter().skip(skip_count).collect();
        }

        Ok(Self {
            project_path,
            input_history,
            input_index: None,
            terminal_lines,
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        if let Ok(new_history) = get_project_history(&self.project_path) {
            self.input_history = new_history;
        }
        Ok(())
    }

    pub fn save_input(&mut self, cmd: &str) -> Result<()> {
        self.refresh()?;

        let cmd_str = cmd.to_string();
        if let Some(pos) = self.input_history.history.iter().position(|x| *x == cmd_str) {
            self.input_history.history.remove(pos);
        }
        self.input_history.history.push(cmd_str);
        
        save_project_history(&self.project_path, &self.input_history)?;
        Ok(())
    }

    pub fn prev_input(&mut self) -> Option<String> {
        if self.input_history.history.is_empty() {
            return None;
        }
        let new_index = match self.input_index {
            None => Some(self.input_history.history.len() - 1),
            Some(idx) if idx > 0 => Some(idx - 1),
            Some(_) => Some(0),
        };
        self.input_index = new_index;
        new_index.map(|idx| self.input_history.history[idx].clone())
    }

    pub fn next_input(&mut self) -> Option<String> {
        if let Some(idx) = self.input_index {
            if idx < self.input_history.history.len() - 1 {
                let next_idx = idx + 1;
                self.input_index = Some(next_idx);
                return Some(self.input_history.history[next_idx].clone());
            }
        }
        self.input_index = None;
        None
    }

    pub fn reset_input_index(&mut self) {
        self.input_index = None;
    }

    pub fn push_output(&mut self, text: &str, max_width: usize) {
        if text.is_empty() {
            return;
        }
        for line in text.lines() {
            let mut current = line.to_string();
            if current.is_empty() {
                self.terminal_lines.push(" ".to_string());
                continue;
            }
            while measure_text_width(&strip_ansi_codes(&current)) > max_width && max_width > 0 {
                let mut split_idx = 0;
                let mut width = 0;
                let mut in_escape = false;
                
                for (i, c) in current.char_indices() {
                    if c == '\x1b' {
                        in_escape = true;
                    } else if in_escape {
                        if c >= '@' && c <= '~' {
                            in_escape = false;
                        }
                    } else {
                        let c_width = measure_text_width(&c.to_string());
                        if width + c_width > max_width {
                            break;
                        }
                        width += c_width;
                    }
                    split_idx = i + c.len_utf8();
                }
                
                if split_idx == 0 || split_idx == current.len() {
                    break;
                }
                
                let (head, tail) = current.split_at(split_idx);
                self.terminal_lines.push(head.to_string());
                current = tail.to_string();
            }
            self.terminal_lines.push(current);
        }
    }

    pub fn pop_output(&mut self) {
        self.terminal_lines.pop();
    }

    pub fn clear_output(&mut self) {
        self.terminal_lines.clear();
    }

    pub fn limit_output(&mut self, max_lines: usize) {
        let actual_max = max_lines.max(1);
        while self.terminal_lines.len() > actual_max {
            self.terminal_lines.remove(0);
        }
    }
}
