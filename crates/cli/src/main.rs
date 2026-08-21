//! nextool-cli:NexToolkit 命令行入口
//!
//! clap 解析子命令后直接调用 nextool-core,输出格式化归本层。
//! 输入可来自参数或 stdin(管道友好)。

use std::io::{Read, Write};

use clap::{Parser, Subcommand};

/// NexToolkit 命令行工具集
#[derive(Parser)]
#[command(name = "nextool", version, about = "开源本地工具集", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Base64 编解码
    Base64 {
        /// encode 或 decode
        mode: Mode,
        /// 输入文本;省略则读 stdin
        input: Option<String>,
    },
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Mode {
    Encode,
    Decode,
}

fn main() {
    let cli = Cli::parse();
    let code = match run(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("错误: {e}");
            1
        }
    };
    std::process::exit(code);
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Base64 { mode, input } => {
            let input = read_input(input)?;
            let out = match mode {
                Mode::Encode => nextool_core::base64_encode(&input),
                Mode::Decode => nextool_core::base64_decode(&input),
            }
            .map_err(|e| e.to_string())?;
            println!("{out}");
            Ok(())
        }
    }
}

/// 读取输入:有参数用参数,否则读 stdin 全部
fn read_input(input: Option<String>) -> Result<String, String> {
    match input {
        Some(s) => Ok(s),
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| e.to_string())?;
            if buf.is_empty() {
                return Err("无输入:请提供参数或通过 stdin 传入".into());
            }
            Ok(buf)
        }
    }
}

// 防止未用警告(stdout 写出在 println!,此处保留 Write trait 引用)
#[allow(dead_code)]
fn _ensure_write() {
    let _ = std::io::stdout().flush();
}
