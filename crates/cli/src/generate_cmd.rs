//! 生成器子命令:hash/uuid/password/lorem/qr/hmac

use clap::{Args, Subcommand};

use crate::io::{read_input, read_input_optional};
use nextool_core::HashAlgo;

#[derive(Args)]
pub struct GenerateArgs {
    #[command(subcommand)]
    cmd: GenerateCmd,
}

#[derive(Subcommand)]
enum GenerateCmd {
    /// 计算哈希:md5/sha1/sha256/sha512
    Hash { algo: HashAlgoArg, input: Option<String> },
    /// 生成 HMAC:--key 指定密钥
    Hmac {
        algo: HashAlgoArg,
        #[arg(long)]
        key: String,
        input: Option<String>,
    },
    /// 生成 UUID v4
    UuidV4,
    /// 生成 UUID v7(基于时间戳)
    UuidV7,
    /// 生成密码:--length 长度;--upper/--lower/--digits/--symbols 选择字符集
    Password {
        #[arg(long, default_value_t = 16)]
        length: usize,
        #[arg(long)]
        upper: bool,
        #[arg(long)]
        lower: bool,
        #[arg(long)]
        digits: bool,
        #[arg(long)]
        symbols: bool,
    },
    /// 生成 Lorem Ipsum:段落数
    Lorem { paragraphs: usize },
    /// 生成二维码 SVG
    Qr { input: Option<String> },
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum HashAlgoArg {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

impl From<HashAlgoArg> for HashAlgo {
    fn from(a: HashAlgoArg) -> Self {
        match a {
            HashAlgoArg::Md5 => HashAlgo::Md5,
            HashAlgoArg::Sha1 => HashAlgo::Sha1,
            HashAlgoArg::Sha256 => HashAlgo::Sha256,
            HashAlgoArg::Sha512 => HashAlgo::Sha512,
        }
    }
}

pub fn run(args: GenerateArgs) -> Result<(), String> {
    match args.cmd {
        GenerateCmd::Hash { algo, input } => {
            let input = read_input(input)?;
            println!("{}", nextool_core::hash(&input, algo.into()).map_err(|e| e.to_string())?);
        }
        GenerateCmd::Hmac { algo, key, input } => {
            let input = read_input(input)?;
            println!(
                "{}",
                nextool_core::hmac_compute(&input, &key, algo.into()).map_err(|e| e.to_string())?
            );
        }
        GenerateCmd::UuidV4 => {
            println!("{}", nextool_core::uuid_v4().map_err(|e| e.to_string())?);
        }
        GenerateCmd::UuidV7 => {
            println!("{}", nextool_core::uuid_v7().map_err(|e| e.to_string())?);
        }
        GenerateCmd::Password { length, upper, lower, digits, symbols } => {
            // 未指定任何字符集时默认全开
            let opts = nextool_core::PasswordOpts {
                upper: upper || (!upper && !lower && !digits && !symbols),
                lower: lower || (!upper && !lower && !digits && !symbols),
                digits: digits || (!upper && !lower && !digits && !symbols),
                symbols,
            };
            println!(
                "{}",
                nextool_core::password_generate(length, &opts).map_err(|e| e.to_string())?
            );
        }
        GenerateCmd::Lorem { paragraphs } => {
            println!("{}", nextool_core::lorem_ipsum(paragraphs).map_err(|e| e.to_string())?);
        }
        GenerateCmd::Qr { input } => {
            let input = read_input_optional(input)?.ok_or("二维码需要输入文本")?;
            println!("{}", nextool_core::qr_svg(&input).map_err(|e| e.to_string())?);
        }
    }
    Ok(())
}
