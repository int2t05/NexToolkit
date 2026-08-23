//! 编解码子命令:base64/url/html/hex/jwt + base32/base58/base85/punycode/quoted-printable/morse/braille/zero-width

use clap::{Args, Subcommand};

use crate::io::{print_text, read_input};

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
    /// Base32 编解码(RFC 4648)
    Base32 { mode: Mode, input: Option<String> },
    /// Base58 编解码(Bitcoin 字母表)
    Base58 { mode: Mode, input: Option<String> },
    /// Base85 编解码(Ascii85 Adobe)
    Base85 { mode: Mode, input: Option<String> },
    /// Punycode 编解码(RFC 3492,域名 xn-- 前缀)
    Punycode { mode: Mode, input: Option<String> },
    /// Quoted-Printable 编解码(RFC 2045)
    QuotedPrintable { mode: Mode, input: Option<String> },
    /// Morse 编解码(国际摩斯码)
    Morse { mode: Mode, input: Option<String> },
    /// Braille 编解码(Unicode 6 点盲文)
    Braille { mode: Mode, input: Option<String> },
    /// 零宽字符隐写编解码
    ZeroWidth { mode: Mode, input: Option<String> },
    /// 字符编码转换:文本 → 指定字符集 hex(GBK/Big5/Shift_JIS 等)
    Charset {
        mode: Mode,
        #[arg(
            long,
            help = "字符集:utf-8/gbk/gb2312/gb18030/big5/shift_jis/euc-jp/euc-kr/iso-8859-1/windows-1252"
        )]
        charset: String,
        input: Option<String>,
    },
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
            print_text(input, |s| match mode {
                Mode::Encode => nextool_core::base64_encode(s),
                Mode::Decode => nextool_core::base64_decode(s),
            })?;
        }
        EncodeCmd::Url { mode, input } => {
            print_text(input, |s| match mode {
                Mode::Encode => nextool_core::url_encode(s),
                Mode::Decode => nextool_core::url_decode(s),
            })?;
        }
        EncodeCmd::Html { mode, input } => {
            print_text(input, |s| match mode {
                Mode::Encode => nextool_core::html_encode(s),
                Mode::Decode => nextool_core::html_decode(s),
            })?;
        }
        EncodeCmd::Hex { mode, input } => {
            print_text(input, |s| match mode {
                Mode::Encode => nextool_core::hex_encode(s),
                Mode::Decode => nextool_core::hex_decode(s),
            })?;
        }
        EncodeCmd::Base32 { mode, input } => {
            print_text(input, |s| match mode {
                Mode::Encode => nextool_core::base32_encode(s),
                Mode::Decode => nextool_core::base32_decode(s),
            })?;
        }
        EncodeCmd::Base58 { mode, input } => {
            print_text(input, |s| match mode {
                Mode::Encode => nextool_core::base58_encode(s),
                Mode::Decode => nextool_core::base58_decode(s),
            })?;
        }
        EncodeCmd::Base85 { mode, input } => {
            print_text(input, |s| match mode {
                Mode::Encode => nextool_core::base85_encode(s),
                Mode::Decode => nextool_core::base85_decode(s),
            })?;
        }
        EncodeCmd::Punycode { mode, input } => {
            print_text(input, |s| match mode {
                Mode::Encode => nextool_core::punycode_encode(s),
                Mode::Decode => nextool_core::punycode_decode(s),
            })?;
        }
        EncodeCmd::QuotedPrintable { mode, input } => {
            print_text(input, |s| match mode {
                Mode::Encode => nextool_core::quoted_printable_encode(s),
                Mode::Decode => nextool_core::quoted_printable_decode(s),
            })?;
        }
        EncodeCmd::Morse { mode, input } => {
            print_text(input, |s| match mode {
                Mode::Encode => nextool_core::morse_encode(s),
                Mode::Decode => nextool_core::morse_decode(s),
            })?;
        }
        EncodeCmd::Braille { mode, input } => {
            print_text(input, |s| match mode {
                Mode::Encode => nextool_core::braille_encode(s),
                Mode::Decode => nextool_core::braille_decode(s),
            })?;
        }
        EncodeCmd::ZeroWidth { mode, input } => {
            print_text(input, |s| match mode {
                Mode::Encode => nextool_core::zero_width_encode(s),
                Mode::Decode => nextool_core::zero_width_decode(s),
            })?;
        }
        EncodeCmd::Charset {
            mode,
            charset,
            input,
        } => {
            print_text(input, |s| match mode {
                Mode::Encode => nextool_core::charset_encode(s, &charset),
                Mode::Decode => nextool_core::charset_decode(s, &charset),
            })?;
        }
        EncodeCmd::Jwt { input } => print_text(input, nextool_core::jwt_decode)?,
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
