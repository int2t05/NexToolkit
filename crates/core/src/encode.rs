//! 编解码模块:Base64/URL/HTML/Hex/JWT 等

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
}
