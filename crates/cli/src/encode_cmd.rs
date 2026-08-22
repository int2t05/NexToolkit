//! 编解码子命令:base64/url/html/hex/jwt

use clap::{Args, Subcommand};

use crate::io::read_input;

#[derive(Args)]
pub struct EncodeArgs {
    #[command(subcommand)]
    cmd: EncodeCmd,
}

#[derive(Subcommand)]
enum EncodeCmd {
    /// Base64 编解码
    Base64 { mode: Mode, input: Option<String> },
    /// URL 百分号编解码
    Url { mode: Mode, input: Option<String> },
    /// HTML 实体编解码
    Html { mode: Mode, input: Option<String> },
    /// 十六进制编解码
    Hex { mode: Mode, input: Option<String> },
    /// JWT 解码(不验签,输出 header/payload JSON)
    Jwt { input: Option<String> },
    /// JWT 验签:--key(HS256 secret 或 RS256 公钥 PEM),通过输出 payload JSON
    JwtVerify {
        #[arg(long)]
        key: String,
        input: Option<String>,
    },
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Mode {
    Encode,
    Decode,
}

pub fn run(args: EncodeArgs) -> Result<(), String> {
    match args.cmd {
        EncodeCmd::Base64 { mode, input } => {
            let input = read_input(input)?;
            let out = match mode {
                Mode::Encode => nextool_core::base64_encode(&input),
                Mode::Decode => nextool_core::base64_decode(&input),
            };
            println!("{}", out.map_err(|e| e.to_string())?);
        }
        EncodeCmd::Url { mode, input } => {
            let input = read_input(input)?;
            let out = match mode {
                Mode::Encode => nextool_core::url_encode(&input),
                Mode::Decode => nextool_core::url_decode(&input),
            };
            println!("{}", out.map_err(|e| e.to_string())?);
        }
        EncodeCmd::Html { mode, input } => {
            let input = read_input(input)?;
            let out = match mode {
                Mode::Encode => nextool_core::html_encode(&input),
                Mode::Decode => nextool_core::html_decode(&input),
            };
            println!("{}", out.map_err(|e| e.to_string())?);
        }
        EncodeCmd::Hex { mode, input } => {
            let input = read_input(input)?;
            let out = match mode {
                Mode::Encode => nextool_core::hex_encode(&input),
                Mode::Decode => nextool_core::hex_decode(&input),
            };
            println!("{}", out.map_err(|e| e.to_string())?);
        }
        EncodeCmd::Jwt { input } => {
            let input = read_input(input)?;
            println!(
                "{}",
                nextool_core::jwt_decode(&input).map_err(|e| e.to_string())?
            );
        }
        EncodeCmd::JwtVerify { key, input } => {
            let input = read_input(input)?;
            // key 若以 -----BEGIN 视为内联 PEM,否则作 HS256 secret(也可文件路径)
            let key = if key.starts_with("-----BEGIN") {
                key
            } else if std::path::Path::new(&key).exists() {
                std::fs::read_to_string(&key).map_err(|e| format!("读取 key 文件失败: {e}"))?
            } else {
                key
            };
            println!(
                "{}",
                nextool_core::jwt_verify(&input, &key).map_err(|e| e.to_string())?
            );
        }
    }
    Ok(())
}
