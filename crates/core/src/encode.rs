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
}
