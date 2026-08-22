//! 加密子命令:aes-gcm/rsa/kdf

use clap::{Args, Subcommand};

use crate::io::read_input;

#[derive(Args)]
pub struct CryptoArgs {
    #[command(subcommand)]
    cmd: CryptoCmd,
}

#[derive(Subcommand)]
enum CryptoCmd {
    /// AES-256-GCM 加密:--password 指定口令
    AesEncrypt {
        #[arg(long)]
        password: String,
        input: Option<String>,
    },
    /// AES-256-GCM 解密:--password 指定口令
    AesDecrypt {
        #[arg(long)]
        password: String,
        input: Option<String>,
    },
    /// 生成 RSA 密钥对(PEM):位数(2048/4096)
    RsaKeygen { bits: usize },
    /// RSA 加密:--pub-pem 公钥 PEM 文件路径或内联
    RsaEncrypt {
        #[arg(long)]
        pub_pem: String,
        input: Option<String>,
    },
    /// RSA 解密:--priv-pem 私钥 PEM 文件路径或内联
    RsaDecrypt {
        #[arg(long)]
        priv_pem: String,
        input: Option<String>,
    },
    /// RSA 签名:--priv-pem 私钥,返回 base64 签名
    RsaSign {
        #[arg(long)]
        priv_pem: String,
        input: Option<String>,
    },
    /// RSA 验签:--pub-pem 公钥 + 签名参数,验证通过退出 0
    RsaVerify {
        #[arg(long)]
        pub_pem: String,
        #[arg(long)]
        signature: String,
        input: Option<String>,
    },
    /// PBKDF2 派生:--salt --iterations
    Pbkdf2 {
        #[arg(long)]
        salt: String,
        #[arg(long, default_value_t = 100_000)]
        iterations: u32,
        input: Option<String>,
    },
    /// Argon2id 派生:--salt
    Argon2 {
        #[arg(long)]
        salt: String,
        input: Option<String>,
    },
}

/// 读取 PEM:若以 `-----BEGIN` 开头视为内联 PEM,否则当作文件路径读取
fn load_pem(spec: &str) -> Result<String, String> {
    if spec.starts_with("-----BEGIN") {
        Ok(spec.to_string())
    } else {
        std::fs::read_to_string(spec).map_err(|e| format!("读取 PEM 文件失败 {spec}: {e}"))
    }
}

pub fn run(args: CryptoArgs) -> Result<(), String> {
    match args.cmd {
        CryptoCmd::AesEncrypt { password, input } => {
            let input = read_input(input)?;
            println!(
                "{}",
                nextool_core::aes_gcm_encrypt(&input, &password).map_err(|e| e.to_string())?
            );
        }
        CryptoCmd::AesDecrypt { password, input } => {
            let input = read_input(input)?;
            println!(
                "{}",
                nextool_core::aes_gcm_decrypt(&input, &password).map_err(|e| e.to_string())?
            );
        }
        CryptoCmd::RsaKeygen { bits } => {
            println!(
                "{}",
                nextool_core::rsa_keygen(bits).map_err(|e| e.to_string())?
            );
        }
        CryptoCmd::RsaEncrypt { pub_pem, input } => {
            let pem = load_pem(&pub_pem)?;
            let input = read_input(input)?;
            println!(
                "{}",
                nextool_core::rsa_encrypt(&input, &pem).map_err(|e| e.to_string())?
            );
        }
        CryptoCmd::RsaDecrypt { priv_pem, input } => {
            let pem = load_pem(&priv_pem)?;
            let input = read_input(input)?;
            println!(
                "{}",
                nextool_core::rsa_decrypt(&input, &pem).map_err(|e| e.to_string())?
            );
        }
        CryptoCmd::RsaSign { priv_pem, input } => {
            let pem = load_pem(&priv_pem)?;
            let input = read_input(input)?;
            println!(
                "{}",
                nextool_core::rsa_sign(&input, &pem).map_err(|e| e.to_string())?
            );
        }
        CryptoCmd::RsaVerify {
            pub_pem,
            signature,
            input,
        } => {
            let pem = load_pem(&pub_pem)?;
            let input = read_input(input)?;
            nextool_core::rsa_verify(&input, &pem, &signature).map_err(|e| e.to_string())?;
            println!("签名验证通过");
        }
        CryptoCmd::Pbkdf2 {
            salt,
            iterations,
            input,
        } => {
            let input = read_input(input)?;
            println!(
                "{}",
                nextool_core::kdf_pbkdf2(&input, &salt, iterations).map_err(|e| e.to_string())?
            );
        }
        CryptoCmd::Argon2 { salt, input } => {
            let input = read_input(input)?;
            println!(
                "{}",
                nextool_core::kdf_argon2(&input, &salt).map_err(|e| e.to_string())?
            );
        }
    }
    Ok(())
}
