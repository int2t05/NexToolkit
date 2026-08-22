//! 生成器模块:Hash/HMAC/UUID/密码/Lorem/QR

mod tools;
pub use tools::*;

use crate::{ToolError, ToolResult};

/// 哈希算法类型
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr, strum::EnumString, strum::EnumIter,
)]
pub enum HashAlgo {
    #[strum(serialize = "md5")]
    Md5,
    #[strum(serialize = "sha1")]
    Sha1,
    #[strum(serialize = "sha256")]
    Sha256,
    #[strum(serialize = "sha512")]
    Sha512,
}

/// 计算字符串哈希,返回十六进制摘要
///
/// 支持 Md5/Sha1/Sha256/Sha512。
pub fn hash(s: &str, algo: HashAlgo) -> ToolResult<String> {
    use sha1::Sha1;
    use sha2::{Digest, Sha256, Sha512};
    match algo {
        HashAlgo::Md5 => {
            let mut h = md5::Md5::new();
            h.update(s.as_bytes());
            Ok(format!("{:x}", h.finalize()))
        }
        HashAlgo::Sha1 => {
            let mut h = Sha1::new();
            h.update(s.as_bytes());
            Ok(format!("{:x}", h.finalize()))
        }
        HashAlgo::Sha256 => {
            let mut h = Sha256::new();
            h.update(s.as_bytes());
            Ok(format!("{:x}", h.finalize()))
        }
        HashAlgo::Sha512 => {
            let mut h = Sha512::new();
            h.update(s.as_bytes());
            Ok(format!("{:x}", h.finalize()))
        }
    }
}

/// 计算 HMAC 消息认证码,返回十六进制字符串
///
/// 支持 Md5/Sha1/Sha256/Sha512。
pub fn hmac_compute(data: &str, key: &str, algo: HashAlgo) -> ToolResult<String> {
    use hmac::{Hmac, Mac};
    use sha1::Sha1;
    use sha2::{Sha256, Sha512};
    match algo {
        HashAlgo::Md5 => {
            let mut mac = Hmac::<md5::Md5>::new_from_slice(key.as_bytes())
                .map_err(|e| ToolError::Other(e.to_string()))?;
            mac.update(data.as_bytes());
            Ok(format!("{:x}", mac.finalize().into_bytes()))
        }
        HashAlgo::Sha1 => {
            let mut mac = Hmac::<Sha1>::new_from_slice(key.as_bytes())
                .map_err(|e| ToolError::Other(e.to_string()))?;
            mac.update(data.as_bytes());
            Ok(format!("{:x}", mac.finalize().into_bytes()))
        }
        HashAlgo::Sha256 => {
            let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())
                .map_err(|e| ToolError::Other(e.to_string()))?;
            mac.update(data.as_bytes());
            Ok(format!("{:x}", mac.finalize().into_bytes()))
        }
        HashAlgo::Sha512 => {
            let mut mac = Hmac::<Sha512>::new_from_slice(key.as_bytes())
                .map_err(|e| ToolError::Other(e.to_string()))?;
            mac.update(data.as_bytes());
            Ok(format!("{:x}", mac.finalize().into_bytes()))
        }
    }
}

/// 生成 UUID v4
pub fn uuid_v4() -> ToolResult<String> {
    Ok(uuid::Uuid::new_v4().to_string())
}

/// 生成 UUID v7(基于时间戳)
pub fn uuid_v7() -> ToolResult<String> {
    Ok(uuid::Uuid::now_v7().to_string())
}

/// 密码生成选项
#[derive(Debug, Clone, Copy)]
pub struct PasswordOpts {
    /// 是否包含大写字母 A-Z
    pub upper: bool,
    /// 是否包含小写字母 a-z
    pub lower: bool,
    /// 是否包含数字 0-9
    pub digits: bool,
    /// 是否包含符号字符
    pub symbols: bool,
}

/// 默认字符集:字母 + 数字(不含符号)
impl Default for PasswordOpts {
    fn default() -> Self {
        Self {
            upper: true,
            lower: true,
            digits: true,
            symbols: false,
        }
    }
}

/// 随机生成密码,从启用的字符集中采样
///
/// length < 1 时返回 Err。字符集全 false 时使用默认(字母数字,不含符号),
/// 保证 CLI/GUI 行为一致。
pub fn password_generate(length: usize, opts: &PasswordOpts) -> ToolResult<String> {
    use rand::Rng;

    if length < 1 {
        return Err(ToolError::InvalidInput("密码长度不能小于 1".to_string()));
    }

    // 全 false 时使用默认字符集,避免 CLI/GUI 各自处理默认逻辑
    let opts = if !opts.upper && !opts.lower && !opts.digits && !opts.symbols {
        PasswordOpts::default()
    } else {
        *opts
    };

    let mut charset = String::new();
    if opts.upper {
        charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if opts.lower {
        charset.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    if opts.digits {
        charset.push_str("0123456789");
    }
    if opts.symbols {
        charset.push_str("!@#$%^&*()-_=+[]{};:,.<>?");
    }

    if charset.is_empty() {
        return Err(ToolError::InvalidInput(
            "至少需要选择一种字符集".to_string(),
        ));
    }

    let chars: Vec<char> = charset.chars().collect();
    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect();

    Ok(password)
}

/// Lorem Ipsum 经典段落模板
const LOREM_PARAGRAPH: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris \
nisi ut aliquip ex ea commodo consequat. \
Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. \
Excepteur sint occaecat cupidatat non proident, \
sunt in culpa qui officia deserunt mollit anim id est laborum.";

/// 生成 Lorem Ipsum 文本,段间以空行分隔
///
/// paragraphs == 0 时返回 Err。
pub fn lorem_ipsum(paragraphs: usize) -> ToolResult<String> {
    if paragraphs == 0 {
        return Err(ToolError::InvalidInput("段落数不能为 0".to_string()));
    }
    Ok((0..paragraphs)
        .map(|_| LOREM_PARAGRAPH)
        .collect::<Vec<_>>()
        .join("\n\n"))
}

/// 生成二维码 SVG 字符串
///
/// 空串返回 Err。
pub fn qr_svg(s: &str) -> ToolResult<String> {
    if s.is_empty() {
        return Err(ToolError::EmptyInput);
    }
    let code = qrcode::QrCode::new(s.as_bytes()).map_err(|e| ToolError::Other(e.to_string()))?;
    Ok(code.render::<qrcode::render::svg::Color>().build())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- hash ----

    #[test]
    fn hash_sha256_abc() {
        assert_eq!(
            hash("abc", HashAlgo::Sha256).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn hash_md5_abc() {
        assert_eq!(
            hash("abc", HashAlgo::Md5).unwrap(),
            "900150983cd24fb0d6963f7d28e17f72"
        );
    }

    #[test]
    fn hash_sha512_abc() {
        assert_eq!(
            hash("abc", HashAlgo::Sha512).unwrap(),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
             2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
    }

    #[test]
    fn hash_sha1_abc() {
        // NIST FIPS 180-1 测试向量
        assert_eq!(
            hash("abc", HashAlgo::Sha1).unwrap(),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
    }

    // ---- hmac ----

    #[test]
    fn hmac_sha256_known_value() {
        // 测试向量来自 hmac crate 官方文档
        let result = hmac_compute(
            "input message",
            "my secret and secure key",
            HashAlgo::Sha256,
        )
        .unwrap();
        assert_eq!(
            result,
            "97d2a569059bbcd8ead4444ff99071f4c01d005bcefe0d3567e1be628e5fdcd9"
        );
    }

    #[test]
    fn hmac_sha512_format() {
        let result = hmac_compute("hello", "key", HashAlgo::Sha512).unwrap();
        assert_eq!(result.len(), 128);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hmac_deterministic() {
        let a = hmac_compute("data", "key", HashAlgo::Sha256).unwrap();
        let b = hmac_compute("data", "key", HashAlgo::Sha256).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn hmac_sha1_basic() {
        // RFC 2202 测试用例 2:HMAC-SHA1(data="what do ya want for nothing?", key="Jefe")
        assert_eq!(
            hmac_compute("what do ya want for nothing?", "Jefe", HashAlgo::Sha1).unwrap(),
            "effcdf6ae5eb2fa2d27416d5f184df9c259a7c79"
        );
    }

    // ---- uuid_v4 ----

    #[test]
    fn uuid_v4_length() {
        let id = uuid_v4().unwrap();
        assert_eq!(id.len(), 36);
    }

    #[test]
    fn uuid_v4_hyphens() {
        let id = uuid_v4().unwrap();
        assert_eq!(id.matches('-').count(), 4);
    }

    #[test]
    fn uuid_v4_version() {
        let id = uuid_v4().unwrap();
        assert_eq!(id.chars().nth(14), Some('4'));
    }

    // ---- uuid_v7 ----

    #[test]
    fn uuid_v7_length() {
        let id = uuid_v7().unwrap();
        assert_eq!(id.len(), 36);
    }

    #[test]
    fn uuid_v7_hyphens() {
        let id = uuid_v7().unwrap();
        assert_eq!(id.matches('-').count(), 4);
    }

    #[test]
    fn uuid_v7_version() {
        let id = uuid_v7().unwrap();
        assert_eq!(id.chars().nth(14), Some('7'));
    }

    // ---- password_generate ----

    #[test]
    fn password_alphanumeric_only() {
        let opts = PasswordOpts {
            upper: true,
            lower: true,
            digits: true,
            symbols: false,
        };
        let pwd = password_generate(16, &opts).unwrap();
        assert_eq!(pwd.len(), 16);
        assert!(pwd.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn password_with_symbols() {
        let opts = PasswordOpts {
            upper: true,
            lower: true,
            digits: true,
            symbols: true,
        };
        let pwd = password_generate(20, &opts).unwrap();
        assert_eq!(pwd.len(), 20);
    }

    #[test]
    fn password_zero_length() {
        let opts = PasswordOpts {
            upper: true,
            lower: false,
            digits: false,
            symbols: false,
        };
        assert!(password_generate(0, &opts).is_err());
    }

    #[test]
    fn password_all_false_uses_default() {
        // 全 false 时使用默认字符集(字母数字),不返回错误
        let opts = PasswordOpts {
            upper: false,
            lower: false,
            digits: false,
            symbols: false,
        };
        let pwd = password_generate(16, &opts).unwrap();
        assert_eq!(pwd.len(), 16);
        assert!(pwd.chars().all(|c| c.is_alphanumeric()));
    }

    // ---- lorem_ipsum ----

    #[test]
    fn lorem_two_paragraphs() {
        let text = lorem_ipsum(2).unwrap();
        assert_eq!(text.split("\n\n").count(), 2);
    }

    #[test]
    fn lorem_contains_template() {
        let text = lorem_ipsum(1).unwrap();
        assert!(text.contains("Lorem ipsum"));
    }

    #[test]
    fn lorem_zero_paragraphs() {
        assert!(lorem_ipsum(0).is_err());
    }

    // ---- qr_svg ----

    #[test]
    fn qr_svg_contains_svg_tag() {
        let svg = qr_svg("hello").unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn qr_svg_contains_closing_tag() {
        let svg = qr_svg("https://example.com").unwrap();
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn qr_svg_empty_input() {
        assert!(qr_svg("").is_err());
    }
}
