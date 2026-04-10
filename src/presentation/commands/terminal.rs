use anyhow::{Result, anyhow};
use std::path::Path;
use std::io::Read;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use crate::presentation::commands::Command;
use crate::infrastructure::project_state::get_project_state;

pub struct TerminalCommand;

impl Command for TerminalCommand {
    fn name(&self) -> &str {
        "terminal"
    }

    fn description(&self) -> &str {
        "Execute a terminal command"
    }

    fn help(&self) -> &str {
        "Executes a shell command in the current workspace using a PTY.
Usage: terminal <command> [args]
Example: terminal ls -la"
    }

    fn run(&self, args: Vec<String>, project_path: &Path, current_worktree: &str, term: &console::Term) -> Result<String> {
        if args.is_empty() {
            return Ok("Usage: terminal <command> [args]".to_string());
        }

        let state = get_project_state(project_path)?;
        
        let worktree = state.worktrees.iter().find(|w| w.branch == current_worktree)
            .ok_or_else(|| anyhow!("Worktree '{}' not found", current_worktree))?;

        let dir = project_path.join(&worktree.directory);
        
        // 元のコマンド文字列を再構築
        let cmd_to_run = if args[0] == "terminal" {
            args[1..].join(" ")
        } else {
            args.join(" ")
        };

        if cmd_to_run.is_empty() {
            return Ok("Usage: terminal <command> [args]".to_string());
        }

        let (term_height, term_width) = term.size();
        let left_width = 30;
        let right_width = (term_width as usize).saturating_sub(left_width).saturating_sub(3);
        let right_height = (term_height as usize).saturating_sub(7).max(1);

        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: right_height as u16,
            cols: right_width as u16,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new("sh");
        cmd.arg("-c");
        cmd.arg(&cmd_to_run);
        cmd.cwd(dir);
        cmd.env("CLICOLOR_FORCE", "1");
        cmd.env("TERM", "xterm-256color");

        let mut child = pair.slave.spawn_command(cmd)?;
        
        // slaveをドロップしないと、master側のreadが終了しない場合がある
        drop(pair.slave);

        let mut reader = pair.master.try_clone_reader()?;
        let mut output = String::new();
        let mut buffer = [0u8; 1024];

        // 子プロセスの終了を待機しつつ読み込む
        // シンプルにするため、ここでは読み込めるだけ読み込む
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    output.push_str(&String::from_utf8_lossy(&buffer[..n]));
                }
                Err(_) => break,
            }
            
            // プロセスが終了しているかチェック
            if let Ok(Some(_status)) = child.try_wait() {
                // 残りのデータを読み切る
                while let Ok(n) = reader.read(&mut buffer) {
                    if n == 0 { break; }
                    output.push_str(&String::from_utf8_lossy(&buffer[..n]));
                }
                break;
            }
        }

        if output.is_empty() {
            let status = child.wait()?;
            if status.success() {
                Ok("Command executed successfully (no output)".to_string())
            } else {
                Ok(format!("Command failed with exit status: {}", status))
            }
        } else {
            Ok(output)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_command_help() {
        let cmd = TerminalCommand;
        assert_eq!(cmd.name(), "terminal");
        assert!(!cmd.description().is_empty());
        assert!(cmd.help().contains("PTY"));
    }
}
