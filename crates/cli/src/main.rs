//! nextool-cli:NexToolkit 命令行入口
//!
//! clap 解析子命令后调用 nextool-core,输出格式化归本层。
//! 输入可来自参数或 stdin(管道友好)。命令按域分文件模块,避免单巨型入口。

mod convert_cmd;
mod crypto_cmd;
mod encode_cmd;
mod format_cmd;
mod generate_cmd;
mod io;
mod nettime_cmd;
mod text_cmd;

use clap::{Parser, Subcommand};

use convert_cmd::ConvertArgs;
use crypto_cmd::CryptoArgs;
use encode_cmd::EncodeArgs;
use format_cmd::FormatArgs;
use generate_cmd::GenerateArgs;
use nettime_cmd::NetTimeArgs;
use text_cmd::TextArgs;

/// NexToolkit:开源本地工具集(纯 Rust,零网络上报)
#[derive(Parser)]
#[command(name = "nextool", version, about = "开源本地工具集", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 编解码:base64/url/html/hex/jwt
    Encode(EncodeArgs),
    /// 格式转换:json-yaml/json-toml/json-csv/md-html/numbase
    Convert(ConvertArgs),
    /// 格式化:json/sql/xml 美化压缩、css 压缩
    Format(FormatArgs),
    /// 生成器:hash/hmac/uuid/password/lorem/qr
    Generate(GenerateArgs),
    /// 文本:case/sort/dedup/reverse/regex/diff
    Text(TextArgs),
    /// 加密:aes-gcm/rsa/kdf
    Crypto(CryptoArgs),
    /// 网络/时间:ipcalc/timestamp/cron/dns
    NetTime(NetTimeArgs),
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Encode(args) => encode_cmd::run(args),
        Command::Convert(args) => convert_cmd::run(args),
        Command::Format(args) => format_cmd::run(args),
        Command::Generate(args) => generate_cmd::run(args),
        Command::Text(args) => text_cmd::run(args),
        Command::Crypto(args) => crypto_cmd::run(args),
        Command::NetTime(args) => nettime_cmd::run(args),
    };
    match result {
        Ok(()) => {}
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    }
}
