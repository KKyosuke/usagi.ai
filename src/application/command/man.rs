use anyhow::Result;
use std::path::Path;
use super::{Command, get_commands};

pub struct ManCommand;

impl Command for ManCommand {
    fn name(&self) -> &str {
        "man"
    }

    fn description(&self) -> &str {
        "マニュアルを表示する"
    }

    fn help(&self) -> &str {
        "利用可能なコマンドの一覧、または指定したコマンドのヘルプを表示します。
使用法: man [command]"
    }

    fn run(&self, args: Vec<String>, _project_path: &Path) -> Result<String> {
        let commands = get_commands();
        if args.len() > 1 {
            let cmd_name = &args[1];
            if let Some(cmd) = commands.iter().find(|c| c.name() == cmd_name) {
                return Ok(format!("コマンド: {}\n説明: {}\nヘルプ:\n{}", cmd.name(), cmd.description(), cmd.help()));
            } else {
                return Ok(format!("コマンド '{}' は見つかりません。", cmd_name));
            }
        }

        let mut output = String::from("利用可能なコマンド:\n");
        for cmd in commands {
            output.push_str(&format!("  {:10} {}\n", cmd.name(), cmd.description()));
        }
        output.push_str("\n'man <command>' で詳細なヘルプを表示できます。");
        Ok(output)
    }
}

pub fn run(args: Vec<String>, project_path: &Path) -> Result<String> {
    ManCommand.run(args, project_path)
}
