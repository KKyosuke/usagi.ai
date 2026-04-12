use anyhow::{Result, anyhow};
use std::path::Path;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use crate::presentation::commands::Command;
use crate::infrastructure::project_state::get_project_state;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub struct TerminalCommand;

impl Command for TerminalCommand {
    fn name(&self) -> &str {
        "terminal"
    }

    fn description(&self) -> &str {
        "Execute an interactive terminal"
    }

    fn help(&self) -> &str {
        "Starts an interactive shell in the current workspace using a PTY.
Usage: terminal [command]
Example: terminal /bin/bash"
    }

    fn run(&self, args: Vec<String>, project_path: &Path, current_worktree: &str, _term: &console::Term) -> Result<String> {
        let state = get_project_state(project_path)?;
        
        let worktree = state.worktrees.iter().find(|w| w.branch == current_worktree)
            .ok_or_else(|| anyhow!("Worktree '{}' not found", current_worktree))?;

        let dir = project_path.join(&worktree.directory);

        // Raw modeを有効化
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let size = terminal.size()?;
        let left_width = 30;
        let right_width = (size.width as usize).saturating_sub(left_width).saturating_sub(3);
        let right_height = (size.height as usize).saturating_sub(7).max(1); // Header(2) + Footer(4) + Label(1) = 7

        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: right_height as u16,
            cols: right_width as u16,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // OSに応じたデフォルトシェル
        let shell = if cfg!(windows) { "cmd.exe" } else { "/bin/bash" };
        let mut cmd = if args.is_empty() || args[0] == "terminal" && args.len() == 1 {
            CommandBuilder::new(shell)
        } else {
            let start_idx = if args[0] == "terminal" { 1 } else { 0 };
            let mut c = CommandBuilder::new(&args[start_idx]);
            for arg in &args[start_idx+1..] {
                c.arg(arg);
            }
            c
        };

        cmd.cwd(dir);
        cmd.env("TERM", "xterm-256color");

        let mut child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave); // slave側は不要なのでドロップ

        let parser = Arc::new(Mutex::new(vt100::Parser::new(right_height as u16, right_width as u16, 0)));
        
        let mut reader = pair.master.try_clone_reader()?;
        let parser_clone = Arc::clone(&parser);
        
        // PTYの出力を読むスレッド
        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        let mut p = parser_clone.lock().unwrap();
                        p.process(&buffer[..n]);
                    }
                    Err(_) => break,
                }
            }
        });

        let mut writer = pair.master.take_writer()?;
        
        // イベントループ
        loop {
            // 描画
            terminal.draw(|f| {
                let size = f.size();
                
                // --- Layout (2 Header + Content + 4 Footer) ---
                let header_chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Length(1), // 0: ----- USAGI TERMINAL -----
                        ratatui::layout::Constraint::Length(1), // 1: MODE: ...
                        ratatui::layout::Constraint::Min(0),    // 2: Content
                        ratatui::layout::Constraint::Length(1), // 3: Separator line
                        ratatui::layout::Constraint::Length(1), // 4: COMMAND
                        ratatui::layout::Constraint::Length(1), // 5: (empty)
                        ratatui::layout::Constraint::Length(1), // 6: Help text
                    ])
                    .split(size);

                let header_text = ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled("----- USAGI TERMINAL -----", ratatui::style::Style::default().fg(ratatui::style::Color::Magenta).add_modifier(ratatui::style::Modifier::BOLD)),
                ]);
                f.render_widget(ratatui::widgets::Paragraph::new(header_text), header_chunks[0]);

                let mode_text = ratatui::text::Line::from(vec![
                    ratatui::text::Span::raw("MODE: "),
                    ratatui::text::Span::styled("TERMINAL", ratatui::style::Style::default().fg(ratatui::style::Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)),
                ]);
                f.render_widget(ratatui::widgets::Paragraph::new(mode_text), header_chunks[1]);

                // --- Main Content (Left: Workspace, Right: Terminal) ---
                let content_chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Horizontal)
                    .constraints([
                        ratatui::layout::Constraint::Length(left_width as u16),
                        ratatui::layout::Constraint::Length(3),
                        ratatui::layout::Constraint::Min(0),
                    ])
                    .split(header_chunks[2]);

                // Left: Workspace
                let mut workspace_lines = vec![
                    ratatui::text::Line::from(ratatui::text::Span::styled("workspace", ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD))),
                ];
                for (_idx, wt) in state.worktrees.iter().enumerate() {
                    let is_selected = wt.branch == current_worktree;
                    let cursor = if is_selected { ">" } else { " " };
                    let mark = if is_selected {
                        ratatui::text::Span::styled("●", ratatui::style::Style::default().fg(ratatui::style::Color::Green))
                    } else {
                        ratatui::text::Span::raw(" ")
                    };
                    
                    let branch_style = if is_selected {
                        ratatui::style::Style::default().fg(ratatui::style::Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)
                    } else {
                        ratatui::style::Style::default()
                    };
                    
                    workspace_lines.push(ratatui::text::Line::from(vec![
                        ratatui::text::Span::raw(format!("{} ", cursor)),
                        mark,
                        ratatui::text::Span::raw("  "),
                        ratatui::text::Span::styled(&wt.branch, branch_style),
                    ]));
                    workspace_lines.push(ratatui::text::Line::from(vec![
                        ratatui::text::Span::raw("   "),
                        ratatui::text::Span::styled(crate::presentation::tui::layout::format_modified_at(&wt.modified_at), ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::DIM)),
                    ]));
                }
                f.render_widget(ratatui::widgets::Paragraph::new(workspace_lines), content_chunks[0]);

                // Separator
                for y in content_chunks[1].y..content_chunks[1].y + content_chunks[1].height {
                    f.buffer_mut().set_string(content_chunks[1].x + 1, y, "|", ratatui::style::Style::default());
                }

                // Terminal Label (Top of right side)
                let term_label = ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled("TERMINAL", ratatui::style::Style::default().fg(ratatui::style::Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)),
                    ratatui::text::Span::raw(format!(" (Dir: {})", worktree.directory)),
                ]);
                f.render_widget(ratatui::widgets::Paragraph::new(term_label), content_chunks[2].inner(&ratatui::layout::Margin { vertical: 0, horizontal: 0 }));

                // --- Terminal Area ---
                let term_rect = ratatui::layout::Rect {
                    x: content_chunks[2].x,
                    y: content_chunks[2].y + 1,
                    width: content_chunks[2].width,
                    height: content_chunks[2].height.saturating_sub(1),
                };

                let p = parser.lock().unwrap();
                let screen = p.screen();
                
                // vt100 の内容を ratatui のバッファに描画
                for row in 0..screen.size().0 {
                    for col in 0..screen.size().1 {
                        if let Some(cell) = screen.cell(row, col) {
                            let x = term_rect.x + col;
                            let y = term_rect.y + row;
                            if x < term_rect.x + term_rect.width && y < term_rect.y + term_rect.height {
                                let mut style = ratatui::style::Style::default();
                                
                                // 色の設定
                                match cell.fgcolor() {
                                    vt100::Color::Rgb(r, g, b) => {
                                        style = style.fg(ratatui::style::Color::Rgb(r, g, b));
                                    }
                                    vt100::Color::Idx(i) => {
                                        style = style.fg(ratatui::style::Color::Indexed(i));
                                    }
                                    _ => {}
                                }
                                match cell.bgcolor() {
                                    vt100::Color::Rgb(r, g, b) => {
                                        style = style.bg(ratatui::style::Color::Rgb(r, g, b));
                                    }
                                    vt100::Color::Idx(i) => {
                                        style = style.bg(ratatui::style::Color::Indexed(i));
                                    }
                                    _ => {}
                                }

                                // 属性の設定
                                if cell.bold() {
                                    style = style.add_modifier(ratatui::style::Modifier::BOLD);
                                }
                                if cell.italic() {
                                    style = style.add_modifier(ratatui::style::Modifier::ITALIC);
                                }
                                if cell.underline() {
                                    style = style.add_modifier(ratatui::style::Modifier::UNDERLINED);
                                }

                                let char_str = cell.contents();
                                f.buffer_mut().set_string(x, y, char_str, style);
                            }
                        }
                    }
                }

                // --- Footer ---
                let sep_line = "-".repeat(size.width as usize);
                f.render_widget(ratatui::widgets::Paragraph::new(sep_line), header_chunks[3]);

                let cmd_text = format!("COMMAND{:padding$} | terminal", "", padding = (left_width as usize).saturating_sub(7));
                f.render_widget(ratatui::widgets::Paragraph::new(cmd_text), header_chunks[4]);

                let help_text = ratatui::text::Span::styled("Press Ctrl-D or type 'exit' to close terminal.", ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::DIM));
                f.render_widget(ratatui::widgets::Paragraph::new(help_text), header_chunks[6]);

                // カーソルの描画
                if !screen.hide_cursor() {
                    let (cursor_row, cursor_col) = screen.cursor_position();
                    let cursor_x = term_rect.x + cursor_col;
                    let cursor_y = term_rect.y + cursor_row;
                    if cursor_x < term_rect.x + term_rect.width && cursor_y < term_rect.y + term_rect.height {
                        f.set_cursor(cursor_x, cursor_y);
                    }
                }
            })?;

            // 入力処理
            if event::poll(Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    // Ctrl-C で終了するなどの暫定的な処理は入れない（シェルに送るべき）
                    // ただし、もしシェルが終了していたらループを抜ける
                    
                    let mut input = Vec::new();
                    match key.code {
                        KeyCode::Char(c) => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                match c {
                                    'c' => input.push(3),
                                    'd' => input.push(4),
                                    _ => {
                                        if (c as u8) >= b'a' && (c as u8) <= b'z' {
                                            input.push((c as u8) - b'a' + 1);
                                        }
                                    }
                                }
                            } else {
                                let mut buf = [0u8; 4];
                                input.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                            }
                        }
                        KeyCode::Enter => input.push(b'\r'),
                        KeyCode::Backspace => input.push(8),
                        KeyCode::Tab => input.push(9),
                        KeyCode::Esc => input.push(27),
                        KeyCode::Up => input.extend_from_slice(b"\x1b[A"),
                        KeyCode::Down => input.extend_from_slice(b"\x1b[B"),
                        KeyCode::Right => input.extend_from_slice(b"\x1b[C"),
                        KeyCode::Left => input.extend_from_slice(b"\x1b[D"),
                        _ => {}
                    }
                    if !input.is_empty() {
                        writer.write_all(&input)?;
                        writer.flush()?;
                    }
                }
            }

            // 子プロセスの終了チェック
            if let Ok(Some(_status)) = child.try_wait() {
                // 少し待ってから終了（最後の出力を処理するため）
                thread::sleep(Duration::from_millis(100));
                break;
            }
        }

        // 後始末
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok("Terminal closed".to_string())
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
