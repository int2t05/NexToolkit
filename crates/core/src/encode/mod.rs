//! 编解码模块:Base64/URL/HTML/Hex/JWT 等

mod tools;
pub use tools::*;

use crate::{ToolError, ToolResult};

/// Base64 标准编码(含 padding)
pub fn base64_encode(input: &str) -> ToolResult<String> {
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(input.as_bytes()))
}

/// Base64 标准解码(容忍首尾空白)
pub fn base64_decode(input: &str) -> ToolResult<String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD.decode(input.trim())?;
    String::from_utf8(bytes).map_err(ToolError::from)
}

/// URL 百分号编码:对非字母数字与 `-_.~` 外的字节进行 %XX 编码
pub fn url_encode(input: &str) -> ToolResult<String> {
    Ok(urlencoding::encode(input).into_owned())
}

/// URL 百分号解码:将 %XX 还原为原始字符,解码结果非合法 UTF-8 时报错
pub fn url_decode(input: &str) -> ToolResult<String> {
    let cow = urlencoding::decode(input)
        .map_err(|_| ToolError::Parse("URL 解码结果非合法 UTF-8".to_string()))?;
    Ok(cow.into_owned())
}

/// HTML 实体编码:将 `& < > " '` 与 `/` 转为 HTML 实体(采用 html_escape::encode_safe)
pub fn html_encode(input: &str) -> ToolResult<String> {
    Ok(html_escape::encode_safe(input).into_owned())
}

/// HTML 实体解码:将命名/数字字符引用还原为原始字符
pub fn html_decode(input: &str) -> ToolResult<String> {
    Ok(html_escape::decode_html_entities(input).into_owned())
}

/// 十六进制编码:字符串字节转为小写十六进制
pub fn hex_encode(input: &str) -> ToolResult<String> {
    Ok(hex::encode(input.as_bytes()))
}

/// 十六进制解码:十六进制字符串还原为原始字符串(容忍空白与大小写)
pub fn hex_decode(input: &str) -> ToolResult<String> {
    let stripped: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = hex::decode(&stripped).map_err(|e| ToolError::Parse(e.to_string()))?;
    String::from_utf8(bytes).map_err(ToolError::from)
}

/// JWT 解码(不验签):按 `.` 分 header/payload/signature 三段,base64url 解码 header 与 payload,
/// 返回 `{"header":<..>,"payload":<..>}` 的 pretty JSON;签名段不处理,非法格式报错
pub fn jwt_decode(input: &str) -> ToolResult<String> {
    use base64::Engine;
    use serde_json::{json, Value};

    let parts: Vec<&str> = input.split('.').collect();
    if parts.len() != 3 {
        return Err(ToolError::InvalidInput(
            "JWT 格式错误:应为 header.payload.signature 三段".into(),
        ));
    }

    // base64url(无 padding)解码后解析为 JSON
    let decode_seg = |seg: &str| -> ToolResult<Value> {
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(seg)?;
        Ok(serde_json::from_slice(&bytes)?)
    };

    let header = decode_seg(parts[0])?;
    let payload = decode_seg(parts[1])?;
    // 签名段 parts[2] 不处理

    let output = json!({ "header": header, "payload": payload });
    Ok(serde_json::to_string_pretty(&output)?)
}

/// JWT 验签:校验 header.payload 段的签名,支持 HS256(HMAC,secret)与 RS256(RSA,公钥 PEM)
///
/// `key` 为 HS256 的 secret 字符串或 RS256 的公钥 PEM。验签通过返回 payload 的 pretty JSON,
/// 失败(签名错/alg 不支持/key 错)返回 Err。
pub fn jwt_verify(input: &str, key: &str) -> ToolResult<String> {
    use base64::Engine;
    use serde_json::Value;

    let parts: Vec<&str> = input.split('.').collect();
    if parts.len() != 3 {
        return Err(ToolError::InvalidInput(
            "JWT 格式错误:应为 header.payload.signature 三段".into(),
        ));
    }

    let header_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[0])?;
    let header: Value = serde_json::from_slice(&header_bytes)?;
    let alg = header
        .get("alg")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidInput("JWT header 缺 alg 字段".into()))?;

    // 签名输入:header.payload(前两段,含点)
    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let signature = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[2])?;

    match alg {
        "HS256" => {
            use hmac::{Hmac, Mac};
            use sha2::Sha256;
            type HmacSha256 = Hmac<Sha256>;
            let mut mac = HmacSha256::new_from_slice(key.as_bytes())
                .map_err(|e| ToolError::Other(e.to_string()))?;
            mac.update(signing_input.as_bytes());
            mac.verify_slice(&signature)
                .map_err(|_| ToolError::InvalidInput("JWT 签名验证失败".into()))?;
        }
        "RS256" => {
            use rsa::pkcs1v15::VerifyingKey;
            use rsa::signature::Verifier;
            use sha2::Sha256;
            let pub_key = crate::crypto::parse_rsa_public_key(key)?;
            let verifying_key = VerifyingKey::<Sha256>::new(pub_key);
            let signature = rsa::pkcs1v15::Signature::try_from(signature.as_slice())
                .map_err(|e| ToolError::InvalidInput(format!("无效签名: {e}")))?;
            verifying_key
                .verify(signing_input.as_bytes(), &signature)
                .map_err(|_| ToolError::InvalidInput("JWT 签名验证失败".into()))?;
        }
        other => {
            return Err(ToolError::InvalidInput(format!(
                "不支持的 JWT 算法: {other}(仅 HS256/RS256)"
            )));
        }
    }

    // 验签通过,返回 payload pretty JSON
    let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1])?;
    let payload: Value = serde_json::from_slice(&payload_bytes)?;
    Ok(serde_json::to_string_pretty(&payload)?)
}

// ---------------------------------------------------------------------------
// Base32(RFC 4648)
// ---------------------------------------------------------------------------

const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// Base32 编码(RFC 4648,标准字母表 A-Z2-7,含 padding)
pub fn base32_encode(input: &str) -> ToolResult<String> {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len().div_ceil(5) * 8);

    for chunk in bytes.chunks(5) {
        // 将 chunk 补齐到 5 字节(高位在前)
        let mut buf = [0u8; 5];
        buf[..chunk.len()].copy_from_slice(chunk);

        // 5 字节 → 40 bit → 8 个 5-bit 组(高位在前)
        let bits = u64::from_be_bytes({
            let mut b = [0u8; 8];
            b[3..8].copy_from_slice(&buf);
            b
        });
        let groups = [
            ((bits >> 35) & 0x1F) as usize,
            ((bits >> 30) & 0x1F) as usize,
            ((bits >> 25) & 0x1F) as usize,
            ((bits >> 20) & 0x1F) as usize,
            ((bits >> 15) & 0x1F) as usize,
            ((bits >> 10) & 0x1F) as usize,
            ((bits >> 5) & 0x1F) as usize,
            (bits & 0x1F) as usize,
        ];

        // 根据输入字节数决定输出字符数与 padding
        let (chars, pad) = match chunk.len() {
            1 => (2, 6),
            2 => (4, 4),
            3 => (5, 3),
            4 => (7, 1),
            _ => (8, 0),
        };
        for &g in &groups[..chars] {
            out.push(BASE32_ALPHABET[g] as char);
        }
        out.push_str(&"=".repeat(pad));
    }
    Ok(out)
}

/// Base32 解码(RFC 4648,容忍大小写与首尾空白,padding 可省略)
pub fn base32_decode(input: &str) -> ToolResult<String> {
    let stripped: String = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    let bytes = stripped.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 8 * 5);

    for chunk in bytes.chunks(8) {
        let chars = chunk.iter().take_while(|&&c| c != b'=').count();
        if chars == 0 {
            continue;
        }
        if !(2..=8).contains(&chars) {
            return Err(ToolError::InvalidInput(format!(
                "Base32 分组长度无效: {chars}(应为 2-8)"
            )));
        }

        // 8 个 5-bit 值 → 40 bit → 5 字节
        let mut bits: u64 = 0;
        for &c in &chunk[..chars] {
            let val = match c {
                b'A'..=b'Z' => c - b'A',
                b'2'..=b'7' => c - b'2' + 26,
                _ => return Err(ToolError::Parse(format!("非法 Base32 字符: {}", c as char))),
            };
            bits = (bits << 5) | val as u64;
        }
        // 不足 8 字符的分组需要左移补齐到 40 bit
        bits <<= (8 - chars) * 5;

        let raw = bits.to_be_bytes();
        // 根据字符数决定输出字节数
        let out_bytes = match chars {
            2 => 1,
            4 => 2,
            5 => 3,
            7 => 4,
            8 => 5,
            _ => return Err(ToolError::InvalidInput("Base32 分组字符数无效".into())),
        };
        out.extend_from_slice(&raw[3..3 + out_bytes]);
    }

    String::from_utf8(out).map_err(ToolError::from)
}

// ---------------------------------------------------------------------------
// Base58(Bitcoin 字母表)
// ---------------------------------------------------------------------------

/// Base58 编码(Bitcoin 字母表,无前缀/校验)
pub fn base58_encode(input: &str) -> ToolResult<String> {
    Ok(bs58::encode(input.as_bytes()).into_string())
}

/// Base58 解码(Bitcoin 字母表,容忍首尾空白)
pub fn base58_decode(input: &str) -> ToolResult<String> {
    let bytes = bs58::decode(input.trim())
        .into_vec()
        .map_err(|e| ToolError::Parse(e.to_string()))?;
    String::from_utf8(bytes).map_err(ToolError::from)
}

// ---------------------------------------------------------------------------
// Base85 / Ascii85(Adobe 变体,`!` 起始,`z` 零字节简写,不含 <~ ~> 包裹)
// ---------------------------------------------------------------------------

const ASCII85_OFFSET: u32 = 33; // '!' = 33

/// Ascii85 编码:4 字节 → 5 字符(85 进制),全零组输出 `z`,尾部按实际字节数截断
pub fn base85_encode(input: &str) -> ToolResult<String> {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len().div_ceil(4) * 5);

    for chunk in bytes.chunks(4) {
        if chunk.len() == 4 && chunk == [0, 0, 0, 0] {
            out.push('z');
            continue;
        }

        // 补齐到 4 字节(零填充)
        let mut buf = [0u8; 4];
        buf[..chunk.len()].copy_from_slice(chunk);
        let n = u32::from_be_bytes(buf);

        // 5 个 85 进制位(高位在前)
        let digits = [
            (n / 85u32.pow(4)) % 85,
            (n / 85u32.pow(3)) % 85,
            (n / 85u32.pow(2)) % 85,
            (n / 85u32.pow(1)) % 85,
            n % 85,
        ];

        // 完整组输出 5 字符,尾部组输出 chunk.len()+1 字符
        let out_chars = if chunk.len() == 4 { 5 } else { chunk.len() + 1 };
        for &d in &digits[..out_chars] {
            out.push((ASCII85_OFFSET + d) as u8 as char);
        }
    }
    Ok(out)
}

/// Ascii85 解码:`z` → 4 零字节,5 字符 → 4 字节,尾部按字符数截断
pub fn base85_decode(input: &str) -> ToolResult<String> {
    let bytes: Vec<u8> = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c as u8)
        .collect();

    // 逐字符处理,按 5 字符分组(或 z 单字符组)
    let mut out = Vec::new();
    let mut group: Vec<u8> = Vec::with_capacity(5);

    for &c in &bytes {
        if c == b'z' && group.is_empty() {
            out.extend_from_slice(&[0, 0, 0, 0]);
        } else if c == b'z' {
            return Err(ToolError::Parse("Ascii85: z 只能出现在完整组边界".into()));
        } else if (b'!'..=b'u').contains(&c) {
            group.push(c);
            if group.len() == 5 {
                let n = decode_ascii85_group(&group)?;
                out.extend_from_slice(&n.to_be_bytes());
                group.clear();
            }
        } else {
            return Err(ToolError::Parse(format!(
                "非法 Ascii85 字符: {}",
                c as char
            )));
        }
    }

    // 处理尾部不完整组(2-4 字符)
    if !group.is_empty() {
        let chars = group.len();
        if !(2..=4).contains(&chars) {
            return Err(ToolError::Parse(format!(
                "Ascii85 尾部分组字符数无效: {chars}(应为 2-4)"
            )));
        }
        // 补齐到 5 字符(用 'u' = 最大值填充)
        while group.len() < 5 {
            group.push(b'u');
        }
        let n = decode_ascii85_group(&group)?;
        // 输出 chars-1 字节
        let raw = n.to_be_bytes();
        out.extend_from_slice(&raw[..chars - 1]);
    }

    String::from_utf8(out).map_err(ToolError::from)
}

/// 将 5 个 Ascii85 字符解码为 u32
fn decode_ascii85_group(group: &[u8]) -> ToolResult<u32> {
    let mut n: u32 = 0;
    for &c in group {
        n = n
            .checked_mul(85)
            .ok_or_else(|| ToolError::Parse("Ascii85 解码溢出".into()))?;
        n = n
            .checked_add((c - ASCII85_OFFSET as u8) as u32)
            .ok_or_else(|| ToolError::Parse("Ascii85 解码溢出".into()))?;
    }
    Ok(n)
}

// ---------------------------------------------------------------------------
// Punycode(RFC 3492,域名标签 xn-- 前缀)
// ---------------------------------------------------------------------------

/// Punycode 编码:Unicode 字符串 → `xn--<punycode>` 域名标签
pub fn punycode_encode(input: &str) -> ToolResult<String> {
    let encoded = punycode::encode(input)
        .map_err(|_| ToolError::InvalidInput("Punycode 编码失败: 输入含无效字符".into()))?;
    Ok(format!("xn--{encoded}"))
}

/// Punycode 解码:`xn--<punycode>` 或裸 punycode → Unicode 字符串
pub fn punycode_decode(input: &str) -> ToolResult<String> {
    let stripped = input.trim();
    let bare = stripped
        .strip_prefix("xn--")
        .or_else(|| stripped.strip_prefix("XN--"))
        .unwrap_or(stripped);
    punycode::decode(bare)
        .map_err(|_| ToolError::InvalidInput("Punycode 解码失败: 输入格式错误".into()))
}

// ---------------------------------------------------------------------------
// Quoted-Printable(RFC 2045)
// ---------------------------------------------------------------------------

/// Quoted-Printable 编码:将文本编码为 ASCII 安全的 QP 格式
pub fn quoted_printable_encode(input: &str) -> ToolResult<String> {
    Ok(quoted_printable::encode_to_str(input.as_bytes()))
}

/// Quoted-Printable 解码:将 QP 编码文本还原为原始字符串
pub fn quoted_printable_decode(input: &str) -> ToolResult<String> {
    let bytes = quoted_printable::decode(input.as_bytes(), quoted_printable::ParseMode::Robust)
        .map_err(|e| ToolError::Parse(e.to_string()))?;
    String::from_utf8(bytes).map_err(ToolError::from)
}

// ---------------------------------------------------------------------------
// Morse(国际摩斯码,ITU-R M.1677)
// ---------------------------------------------------------------------------

/// 摩斯码表:A-Z 0-9 → 点划字符串
const MORSE_TABLE: &[(&str, &str)] = &[
    ("A", ".-"),
    ("B", "-..."),
    ("C", "-.-."),
    ("D", "-.."),
    ("E", "."),
    ("F", "..-."),
    ("G", "--."),
    ("H", "...."),
    ("I", ".."),
    ("J", ".---"),
    ("K", "-.-"),
    ("L", ".-.."),
    ("M", "--"),
    ("N", "-."),
    ("O", "---"),
    ("P", ".--."),
    ("Q", "--.-"),
    ("R", ".-."),
    ("S", "..."),
    ("T", "-"),
    ("U", "..-"),
    ("V", "...-"),
    ("W", ".--"),
    ("X", "-..-"),
    ("Y", "-.--"),
    ("Z", "--.."),
    ("0", "-----"),
    ("1", ".----"),
    ("2", "..---"),
    ("3", "...--"),
    ("4", "....-"),
    ("5", "....."),
    ("6", "-...."),
    ("7", "--..."),
    ("8", "---.."),
    ("9", "----."),
];

/// Morse 编码:文本 → 摩斯码(字母间空格分隔,单词间 ` / ` 分隔)
pub fn morse_encode(input: &str) -> ToolResult<String> {
    let mut words: Vec<String> = Vec::new();
    for word in input.split_whitespace() {
        let mut letters: Vec<String> = Vec::new();
        for ch in word.chars() {
            let upper = ch.to_ascii_uppercase().to_string();
            let code = MORSE_TABLE
                .iter()
                .find(|(c, _)| *c == upper)
                .map(|(_, m)| *m)
                .ok_or_else(|| {
                    ToolError::InvalidInput(format!("Morse 不支持字符: {ch}(仅 A-Z 0-9)"))
                })?;
            letters.push(code.to_string());
        }
        words.push(letters.join(" "));
    }
    Ok(words.join(" / "))
}

/// Morse 解码:摩斯码 → 文本(字母间空格,单词间 ` / `)
pub fn morse_decode(input: &str) -> ToolResult<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    let mut words: Vec<String> = Vec::new();
    for word in trimmed.split(" / ") {
        let mut letters: Vec<String> = Vec::new();
        for code in word.split_whitespace() {
            let ch = MORSE_TABLE
                .iter()
                .find(|(_, m)| *m == code)
                .map(|(c, _)| *c)
                .ok_or_else(|| ToolError::InvalidInput(format!("无效摩斯码: {code}")))?;
            letters.push(ch.to_string());
        }
        words.push(letters.join(""));
    }
    Ok(words.join(" "))
}

// ---------------------------------------------------------------------------
// Braille(Unicode 盲文,基本拉丁字母 a-z + 空格)
// ---------------------------------------------------------------------------

/// a-z → 盲文字符位模式(6 点:bit0=dot1 .. bit5=dot6,Unicode U+2800 + pattern)
const BRAILLE_TABLE: &[(char, u8)] = &[
    ('a', 0b000001),
    ('b', 0b000011),
    ('c', 0b001001),
    ('d', 0b011001),
    ('e', 0b010001),
    ('f', 0b001011),
    ('g', 0b011011),
    ('h', 0b010011),
    ('i', 0b001010),
    ('j', 0b011010),
    ('k', 0b000101),
    ('l', 0b000111),
    ('m', 0b001101),
    ('n', 0b011101),
    ('o', 0b010101),
    ('p', 0b001111),
    ('q', 0b011111),
    ('r', 0b010111),
    ('s', 0b001110),
    ('t', 0b011110),
    ('u', 0b100101),
    ('v', 0b100111),
    ('w', 0b111010),
    ('x', 0b101101),
    ('y', 0b111101),
    ('z', 0b110101),
];

const BRAILLE_BLANK: char = '\u{2800}'; // 空白盲文(无点)

/// Braille 编码:文本 → Unicode 盲文字符(a-z 不区分大小写,空格→空白盲文)
pub fn braille_encode(input: &str) -> ToolResult<String> {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch == ' ' {
            out.push(BRAILLE_BLANK);
        } else if ch.is_ascii_alphabetic() {
            let lower = ch.to_ascii_lowercase();
            let pattern = BRAILLE_TABLE
                .iter()
                .find(|(c, _)| *c == lower)
                .map(|(_, p)| *p)
                .ok_or_else(|| ToolError::InvalidInput(format!("Braille 不支持字符: {ch}")))?;
            out.push(char::from_u32(0x2800 + pattern as u32).unwrap());
        } else {
            return Err(ToolError::InvalidInput(format!(
                "Braille 不支持字符: {ch}(仅 a-z 与空格)"
            )));
        }
    }
    Ok(out)
}

/// Braille 解码:Unicode 盲文字符 → 文本
pub fn braille_decode(input: &str) -> ToolResult<String> {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch == BRAILLE_BLANK {
            out.push(' ');
        } else if ('\u{2801}'..='\u{283F}').contains(&ch) {
            let pattern = (ch as u32 - 0x2800) as u8;
            let letter = BRAILLE_TABLE
                .iter()
                .find(|(_, p)| *p == pattern)
                .map(|(c, _)| *c)
                .ok_or_else(|| {
                    ToolError::InvalidInput(format!("无盲文映射: U+{:04X}", ch as u32))
                })?;
            out.push(letter);
        } else if ch.is_whitespace() {
            // 容忍普通空白(视为盲文空白)
            out.push(' ');
        } else {
            return Err(ToolError::InvalidInput(format!(
                "非盲文字符: U+{:04X}",
                ch as u32
            )));
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// 零宽字符隐写(U+200B=0, U+200C=1;文本 ↔ 零宽字符序列)
// ---------------------------------------------------------------------------

const ZW_ZERO: char = '\u{200B}'; // ZERO WIDTH SPACE → bit 0
const ZW_ONE: char = '\u{200C}'; // ZERO WIDTH NON-JOINER → bit 1

/// 零宽字符隐写编码:文本 → UTF-8 字节二进制 → 零宽字符序列(每字节 8 个零宽字符)
pub fn zero_width_encode(input: &str) -> ToolResult<String> {
    let mut out = String::with_capacity(input.len() * 8);
    for byte in input.as_bytes() {
        for i in (0..8).rev() {
            out.push(if (byte >> i) & 1 == 1 {
                ZW_ONE
            } else {
                ZW_ZERO
            });
        }
    }
    Ok(out)
}

/// 零宽字符隐写解码:零宽字符序列 → 二进制 → UTF-8 字节 → 文本
pub fn zero_width_decode(input: &str) -> ToolResult<String> {
    let bits: Vec<u8> = input
        .chars()
        .filter(|c| *c == ZW_ZERO || *c == ZW_ONE)
        .map(|c| if c == ZW_ONE { 1 } else { 0 })
        .collect();

    if bits.is_empty() {
        return Ok(String::new());
    }
    if !bits.len().is_multiple_of(8) {
        return Err(ToolError::InvalidInput(format!(
            "零宽字符数非 8 的倍数: {}(可能是截断或非隐写数据)",
            bits.len()
        )));
    }

    let bytes: Vec<u8> = bits
        .chunks(8)
        .map(|chunk| chunk.iter().fold(0u8, |acc, &bit| (acc << 1) | bit))
        .collect();

    String::from_utf8(bytes).map_err(ToolError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encode_basic() {
        assert_eq!(base64_encode("Hello").unwrap(), "SGVsbG8=");
        assert_eq!(base64_encode("Man").unwrap(), "TWFu");
    }

    #[test]
    fn base64_decode_basic() {
        assert_eq!(base64_decode("SGVsbG8=").unwrap(), "Hello");
        assert_eq!(base64_decode("TWFu").unwrap(), "Man");
    }

    #[test]
    fn base64_roundtrip() {
        for s in ["", "a", "ab", "abc", "中文测试", "Hello, NexToolkit!"] {
            assert_eq!(base64_decode(&base64_encode(s).unwrap()).unwrap(), s);
        }
    }

    #[test]
    fn base64_decode_invalid() {
        assert!(base64_decode("!!!!").is_err());
        assert!(base64_decode("SGVsbG8").is_err()); // 缺 padding
    }

    #[test]
    fn base64_decode_trims_whitespace() {
        assert_eq!(base64_decode("  SGVsbG8= \n").unwrap(), "Hello");
    }

    #[test]
    fn url_encode_basic() {
        assert_eq!(url_encode("hello world").unwrap(), "hello%20world");
        assert_eq!(url_encode("a/b?c=d").unwrap(), "a%2Fb%3Fc%3Dd");
    }

    #[test]
    fn url_encode_safe_chars() {
        // 字母数字与 -_.~ 不编码
        assert_eq!(url_encode("A-Z0-9-_.~").unwrap(), "A-Z0-9-_.~");
    }

    #[test]
    fn url_encode_unicode() {
        assert_eq!(url_encode("中文").unwrap(), "%E4%B8%AD%E6%96%87");
    }

    #[test]
    fn url_decode_basic() {
        assert_eq!(url_decode("hello%20world").unwrap(), "hello world");
        assert_eq!(url_decode("%E4%B8%AD%E6%96%87").unwrap(), "中文");
    }

    #[test]
    fn url_decode_no_encoding() {
        assert_eq!(url_decode("plain text").unwrap(), "plain text");
    }

    #[test]
    fn url_decode_invalid_utf8() {
        // %FF%FE%FD 解码后非合法 UTF-8
        assert!(url_decode("%FF%FE%FD").is_err());
    }

    #[test]
    fn url_roundtrip() {
        for s in ["", "hello", "a b c", "中文测试", "100% sure"] {
            assert_eq!(url_decode(&url_encode(s).unwrap()).unwrap(), s);
        }
    }

    #[test]
    fn html_encode_basic() {
        assert_eq!(html_encode("&").unwrap(), "&amp;");
        assert_eq!(html_encode("<").unwrap(), "&lt;");
        assert_eq!(html_encode(">").unwrap(), "&gt;");
    }

    #[test]
    fn html_encode_quotes() {
        assert_eq!(html_encode("\"").unwrap(), "&quot;");
        assert_eq!(html_encode("'").unwrap(), "&#x27;");
    }

    #[test]
    fn html_encode_mixed() {
        let encoded = html_encode("<a href=\"x\">&</a>").unwrap();
        assert!(encoded.contains("&lt;a"));
        assert!(encoded.contains("&quot;"));
        assert!(encoded.contains("&amp;"));
    }

    #[test]
    fn html_decode_basic() {
        assert_eq!(html_decode("&amp;").unwrap(), "&");
        assert_eq!(html_decode("&lt;").unwrap(), "<");
        assert_eq!(html_decode("&gt;").unwrap(), ">");
        assert_eq!(html_decode("&quot;").unwrap(), "\"");
        assert_eq!(html_decode("&#x27;").unwrap(), "'");
    }

    #[test]
    fn html_decode_numeric_and_chain() {
        assert_eq!(html_decode("&#65;").unwrap(), "A");
        assert_eq!(html_decode("&amp;&lt;").unwrap(), "&<");
    }

    #[test]
    fn html_decode_no_entities() {
        assert_eq!(html_decode("plain text").unwrap(), "plain text");
    }

    #[test]
    fn html_roundtrip() {
        for s in ["&", "<a>", "\"quote\"", "'apostrophe'", "path/to & < >"] {
            assert_eq!(html_decode(&html_encode(s).unwrap()).unwrap(), s);
        }
    }

    #[test]
    fn hex_encode_basic() {
        assert_eq!(hex_encode("A").unwrap(), "41");
        assert_eq!(hex_encode("Hello").unwrap(), "48656c6c6f");
    }

    #[test]
    fn hex_encode_empty_and_unicode() {
        assert_eq!(hex_encode("").unwrap(), "");
        // "中" UTF-8 字节为 E4 B8 AD
        assert_eq!(hex_encode("中").unwrap(), "e4b8ad");
    }

    #[test]
    fn hex_decode_basic() {
        assert_eq!(hex_decode("41").unwrap(), "A");
        assert_eq!(hex_decode("48656c6c6f").unwrap(), "Hello");
    }

    #[test]
    fn hex_decode_whitespace_and_case() {
        assert_eq!(hex_decode(" 41 \n").unwrap(), "A");
        assert_eq!(hex_decode("4A4B4C").unwrap(), "JKL"); // 大写
        assert_eq!(hex_decode("4a 4b 4c").unwrap(), "JKL"); // 小写带空格
    }

    #[test]
    fn hex_decode_invalid() {
        assert!(hex_decode("4").is_err()); // 奇数长度
        assert!(hex_decode("GG").is_err()); // 非十六进制字符
        assert!(hex_decode("ff").is_err()); // 解码出非合法 UTF-8 字节
    }

    #[test]
    fn hex_roundtrip() {
        // hex_encode 输入为 &str(合法 UTF-8),roundtrip 仅覆盖合法 UTF-8
        for s in ["", "A", "Hello", "中文测试", "\u{1F600}"] {
            assert_eq!(hex_decode(&hex_encode(s).unwrap()).unwrap(), s);
        }
    }

    #[test]
    fn jwt_decode_basic() {
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.signature";
        let out = jwt_decode(jwt).unwrap();
        // 解析回 JSON 校验结构,避免依赖具体缩进格式
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["header"]["alg"], "HS256");
        assert_eq!(v["payload"]["sub"], "1");
        assert!(v.get("signature").is_none());
    }

    #[test]
    fn jwt_decode_pretty_format() {
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.sig";
        let out = jwt_decode(jwt).unwrap();
        assert!(out.contains('\n'), "输出应为多行 pretty JSON");
        assert!(out.contains("\"header\""));
        assert!(out.contains("\"payload\""));
    }

    #[test]
    fn jwt_decode_invalid_format() {
        assert!(jwt_decode("not.a.jwt.extra").is_err()); // 4 段
        assert!(jwt_decode("onlyone").is_err()); // 1 段
        assert!(jwt_decode("a.b").is_err()); // 2 段
        assert!(jwt_decode("").is_err()); // 空字符串
    }

    #[test]
    fn jwt_decode_invalid_base64() {
        // payload 段含非法 base64url 字符
        assert!(jwt_decode("eyJhbGciOiJIUzI1NiJ9.!!!!.sig").is_err());
    }

    #[test]
    fn jwt_decode_invalid_json() {
        // "abc" 的 base64url-no-pad 为 YWJj,解码后非合法 JSON
        assert!(jwt_decode("YWJj.eyJzdWIiOiIxIn0.sig").is_err());
    }

    // ---- jwt_verify ----

    /// 构造 HS256 JWT:header.payload 用 base64url 编码,签名用 HMAC-SHA256
    fn make_hs256_jwt(header_json: &str, payload_json: &str, secret: &str) -> String {
        use base64::Engine;
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;
        let header_b64 =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(header_json.as_bytes());
        let payload_b64 =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload_json.as_bytes());
        let signing_input = format!("{header_b64}.{payload_b64}");
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(signing_input.as_bytes());
        let sig = mac.finalize().into_bytes();
        let sig_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sig);
        format!("{signing_input}.{sig_b64}")
    }

    #[test]
    fn jwt_verify_hs256_valid() {
        let jwt = make_hs256_jwt(r#"{"alg":"HS256","typ":"JWT"}"#, r#"{"sub":"1"}"#, "secret");
        let payload = jwt_verify(&jwt, "secret").unwrap();
        let v: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(v["sub"], "1");
    }

    #[test]
    fn jwt_verify_hs256_wrong_secret_fails() {
        let jwt = make_hs256_jwt(r#"{"alg":"HS256","typ":"JWT"}"#, r#"{"sub":"1"}"#, "right");
        assert!(jwt_verify(&jwt, "wrong").is_err());
    }

    #[test]
    fn jwt_verify_hs256_tampered_payload_fails() {
        let jwt = make_hs256_jwt(r#"{"alg":"HS256","typ":"JWT"}"#, r#"{"sub":"1"}"#, "secret");
        // 篡改 payload 段(替换为另一合法 base64url)
        let parts: Vec<&str> = jwt.split('.').collect();
        let tampered = format!("{}.{}.{}", parts[0], "eyJzdWIiOiI5In0", parts[2]); // sub=9
        assert!(
            jwt_verify(&tampered, "secret").is_err(),
            "篡改 payload 应验签失败"
        );
    }

    #[test]
    fn jwt_verify_rs256_valid() {
        let pem = crate::crypto::rsa_keygen(2048).unwrap();
        let pub_pem = extract_pub_pem(&pem);
        let priv_pem = extract_priv_pem(&pem);
        // 构造 RS256 JWT
        use base64::Engine;
        use rsa::pkcs1v15::SigningKey;
        use rsa::signature::{RandomizedSigner, SignatureEncoding};
        use sha2::Sha256;
        let header_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(br#"{"alg":"RS256","typ":"JWT"}"#);
        let payload_b64 =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(br#"{"sub":"abc"}"#);
        let signing_input = format!("{header_b64}.{payload_b64}");
        let priv_key = crate::crypto::parse_rsa_private_key(&priv_pem).unwrap();
        let signing_key = SigningKey::<Sha256>::new(priv_key);
        let mut rng = rand::rngs::OsRng;
        let sig = signing_key.sign_with_rng(&mut rng, signing_input.as_bytes());
        let sig_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sig.to_bytes());
        let jwt = format!("{signing_input}.{sig_b64}");

        let payload = jwt_verify(&jwt, &pub_pem).unwrap();
        let v: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(v["sub"], "abc");
    }

    #[test]
    fn jwt_verify_unsupported_alg_rejected() {
        let jwt = make_hs256_jwt(r#"{"alg":"none","typ":"JWT"}"#, r#"{"sub":"1"}"#, "secret");
        assert!(jwt_verify(&jwt, "secret").is_err(), "alg=none 应不支持");
    }

    #[test]
    fn jwt_verify_invalid_format_rejected() {
        assert!(jwt_verify("not.a.jwt.extra", "secret").is_err());
        assert!(jwt_verify("onlyone", "secret").is_err());
    }

    /// 测试辅助:从 keygen 输出提取公钥 PEM 块
    fn extract_pub_pem(keygen: &str) -> String {
        let begin = "-----BEGIN RSA PUBLIC KEY-----";
        let end = "-----END RSA PUBLIC KEY-----";
        let s = keygen.find(begin).expect("含公钥块");
        let e = keygen.find(end).expect("含公钥结束") + end.len();
        keygen[s..e].to_string()
    }

    fn extract_priv_pem(keygen: &str) -> String {
        let begin = "-----BEGIN RSA PRIVATE KEY-----";
        let end = "-----END RSA PRIVATE KEY-----";
        let s = keygen.find(begin).expect("含私钥块");
        let e = keygen.find(end).expect("含私钥结束") + end.len();
        keygen[s..e].to_string()
    }

    // ---- Base32 ----

    #[test]
    fn base32_rfc4648_vectors() {
        assert_eq!(base32_encode("").unwrap(), "");
        assert_eq!(base32_encode("f").unwrap(), "MY======");
        assert_eq!(base32_encode("fo").unwrap(), "MZXQ====");
        assert_eq!(base32_encode("foo").unwrap(), "MZXW6===");
        assert_eq!(base32_encode("foob").unwrap(), "MZXW6YQ=");
        assert_eq!(base32_encode("fooba").unwrap(), "MZXW6YTB");
        assert_eq!(base32_encode("foobar").unwrap(), "MZXW6YTBOI======");
    }

    #[test]
    fn base32_decode_vectors() {
        assert_eq!(base32_decode("MY======").unwrap(), "f");
        assert_eq!(base32_decode("MZXQ====").unwrap(), "fo");
        assert_eq!(base32_decode("MZXW6===").unwrap(), "foo");
        assert_eq!(base32_decode("MZXW6YQ=").unwrap(), "foob");
        assert_eq!(base32_decode("MZXW6YTB").unwrap(), "fooba");
        assert_eq!(base32_decode("MZXW6YTBOI======").unwrap(), "foobar");
    }

    #[test]
    fn base32_decode_lowercase_and_no_padding() {
        assert_eq!(base32_decode("my======").unwrap(), "f");
        assert_eq!(base32_decode("MY").unwrap(), "f");
        assert_eq!(base32_decode("MZXW6YTB").unwrap(), "fooba");
    }

    #[test]
    fn base32_roundtrip() {
        for s in [
            "",
            "f",
            "fo",
            "foo",
            "foob",
            "fooba",
            "foobar",
            "Hello",
            "中文测试",
        ] {
            assert_eq!(base32_decode(&base32_encode(s).unwrap()).unwrap(), s);
        }
    }

    #[test]
    fn base32_decode_invalid() {
        assert!(base32_decode("!!!").is_err());
        assert!(base32_decode("M").is_err());
    }

    // ---- Base58 ----

    #[test]
    fn base58_empty() {
        assert_eq!(base58_encode("").unwrap(), "");
        assert_eq!(base58_decode("").unwrap(), "");
    }

    #[test]
    fn base58_roundtrip() {
        for s in ["", "Hello", "Hello World", "中文测试", "1234567890"] {
            assert_eq!(base58_decode(&base58_encode(s).unwrap()).unwrap(), s);
        }
    }

    #[test]
    fn base58_decode_invalid() {
        // 0/O/I/l 不在 Bitcoin 字母表
        assert!(base58_decode("0OIl").is_err());
    }

    // ---- Base85 / Ascii85 ----

    #[test]
    fn base85_known_vectors() {
        assert_eq!(base85_encode("Man ").unwrap(), "9jqo^");
        assert_eq!(base85_encode("Man").unwrap(), "9jqo");
        assert_eq!(base85_encode("").unwrap(), "");
        assert_eq!(base85_encode("\u{0}\u{0}\u{0}\u{0}").unwrap(), "z");
    }

    #[test]
    fn base85_decode_known_vectors() {
        assert_eq!(base85_decode("9jqo^").unwrap(), "Man ");
        assert_eq!(base85_decode("9jqo").unwrap(), "Man");
        assert_eq!(base85_decode("z").unwrap(), "\u{0}\u{0}\u{0}\u{0}");
    }

    #[test]
    fn base85_roundtrip() {
        for s in [
            "",
            "Man",
            "Man ",
            "Hello, World!",
            "中文",
            "\u{0}\u{0}\u{0}\u{0}AB",
        ] {
            assert_eq!(base85_decode(&base85_encode(s).unwrap()).unwrap(), s);
        }
    }

    #[test]
    fn base85_decode_invalid() {
        assert!(base85_decode("{").is_err());
    }

    // ---- Punycode ----

    #[test]
    fn punycode_rfc3492_vectors() {
        assert_eq!(punycode_encode("bücher").unwrap(), "xn--bcher-kva");
        assert_eq!(punycode_decode("xn--bcher-kva").unwrap(), "bücher");
        assert_eq!(punycode_encode("münchen").unwrap(), "xn--mnchen-3ya");
        assert_eq!(punycode_decode("xn--mnchen-3ya").unwrap(), "münchen");
    }

    #[test]
    fn punycode_decode_without_prefix() {
        assert_eq!(punycode_decode("bcher-kva").unwrap(), "bücher");
    }

    #[test]
    fn punycode_roundtrip() {
        for s in ["bücher", "münchen", "例え", "中文", "café"] {
            assert_eq!(punycode_decode(&punycode_encode(s).unwrap()).unwrap(), s);
        }
    }

    // ---- Quoted-Printable ----

    #[test]
    fn qp_encode_basic() {
        assert_eq!(quoted_printable_encode("Hello").unwrap(), "Hello");
        assert_eq!(quoted_printable_encode("a=b").unwrap(), "a=3Db");
    }

    #[test]
    fn qp_encode_unicode() {
        // é = U+00E9 = UTF-8 C3 A9
        assert_eq!(quoted_printable_encode("café").unwrap(), "caf=C3=A9");
    }

    #[test]
    fn qp_decode_basic() {
        assert_eq!(quoted_printable_decode("Hello").unwrap(), "Hello");
        assert_eq!(quoted_printable_decode("a=3Db").unwrap(), "a=b");
    }

    #[test]
    fn qp_roundtrip() {
        for s in ["Hello", "a=b", "café", "中文测试", "Hello World!"] {
            assert_eq!(
                quoted_printable_decode(&quoted_printable_encode(s).unwrap()).unwrap(),
                s
            );
        }
    }

    // ---- Morse ----

    #[test]
    fn morse_encode_basic() {
        assert_eq!(morse_encode("SOS").unwrap(), "... --- ...");
        assert_eq!(morse_encode("HELLO").unwrap(), ".... . .-.. .-.. ---");
        assert_eq!(
            morse_encode("HELLO WORLD").unwrap(),
            ".... . .-.. .-.. --- / .-- --- .-. .-.. -.."
        );
    }

    #[test]
    fn morse_encode_lowercase() {
        assert_eq!(morse_encode("hello").unwrap(), ".... . .-.. .-.. ---");
    }

    #[test]
    fn morse_encode_numbers() {
        assert_eq!(morse_encode("123").unwrap(), ".---- ..--- ...--");
    }

    #[test]
    fn morse_decode_basic() {
        assert_eq!(morse_decode("... --- ...").unwrap(), "SOS");
        assert_eq!(morse_decode(".... . .-.. .-.. ---").unwrap(), "HELLO");
        assert_eq!(
            morse_decode(".... . .-.. .-.. --- / .-- --- .-. .-.. -..").unwrap(),
            "HELLO WORLD"
        );
    }

    #[test]
    fn morse_roundtrip() {
        for s in ["SOS", "HELLO WORLD", "ABC 123 XYZ"] {
            assert_eq!(morse_decode(&morse_encode(s).unwrap()).unwrap(), s);
        }
    }

    #[test]
    fn morse_encode_invalid() {
        assert!(morse_encode("你好").is_err());
    }

    // ---- Braille ----

    #[test]
    fn braille_encode_basic() {
        // h=U+2813, e=U+2811, l=U+2807, o=U+2815
        assert_eq!(
            braille_encode("hello").unwrap(),
            "\u{2813}\u{2811}\u{2807}\u{2807}\u{2815}"
        );
        assert_eq!(braille_encode("abc").unwrap(), "\u{2801}\u{2803}\u{2809}");
    }

    #[test]
    fn braille_encode_uppercase() {
        assert_eq!(braille_encode("ABC").unwrap(), "\u{2801}\u{2803}\u{2809}");
    }

    #[test]
    fn braille_encode_space() {
        // a=U+2801, 空格=U+2800, b=U+2803
        assert_eq!(braille_encode("a b").unwrap(), "\u{2801}\u{2800}\u{2803}");
    }

    #[test]
    fn braille_decode_basic() {
        assert_eq!(
            braille_decode("\u{2813}\u{2811}\u{2807}\u{2807}\u{2815}").unwrap(),
            "hello"
        );
        assert_eq!(braille_decode("\u{2801}\u{2803}\u{2809}").unwrap(), "abc");
    }

    #[test]
    fn braille_roundtrip() {
        for s in ["hello", "abc", "a b", "abcdefghijklmnopqrstuvwxyz"] {
            assert_eq!(braille_decode(&braille_encode(s).unwrap()).unwrap(), s);
        }
    }

    #[test]
    fn braille_encode_invalid() {
        assert!(braille_encode("123").is_err());
    }

    // ---- 零宽字符隐写 ----

    #[test]
    fn zero_width_encode_basic() {
        // "Hi" = 2 字节 → 16 个零宽字符
        let encoded = zero_width_encode("Hi").unwrap();
        let zw_count = encoded
            .chars()
            .filter(|c| *c == '\u{200B}' || *c == '\u{200C}')
            .count();
        assert_eq!(zw_count, 16);
    }

    #[test]
    fn zero_width_empty() {
        assert_eq!(zero_width_encode("").unwrap(), "");
        assert_eq!(zero_width_decode("").unwrap(), "");
    }

    #[test]
    fn zero_width_roundtrip() {
        for s in ["", "Hi", "Hello World", "中文测试", "\u{1F600}"] {
            assert_eq!(
                zero_width_decode(&zero_width_encode(s).unwrap()).unwrap(),
                s
            );
        }
    }

    #[test]
    fn zero_width_decode_no_data() {
        // 无零宽字符 → 返回空
        assert_eq!(zero_width_decode("hello").unwrap(), "");
    }

    #[test]
    fn zero_width_decode_invalid() {
        // 零宽字符数非 8 的倍数
        assert!(zero_width_decode("\u{200B}").is_err());
    }
}
