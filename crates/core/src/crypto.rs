//! 加密模块:AES-GCM/RSA/KDF

use crate::{ToolError, ToolResult};
use base64::Engine;
use rand::rngs::OsRng;
use rand::RngCore;
use rsa::{Oaep, RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;

// 私有辅助

/// PBKDF2-HMAC-SHA256 派生密钥(RFC 2898 / RFC 8018)
fn pbkdf2_hmac_sha256(password: &[u8], salt: &[u8], rounds: u32, out: &mut [u8]) {
    pbkdf2::pbkdf2_hmac::<Sha256>(password, salt, rounds, out);
}

// AES-256-GCM

/// AES-256-GCM 加密:随机生成 16 字节 salt 与 12 字节 nonce,
/// 以 PBKDF2-HMAC-SHA256(100_000 轮)从口令派生 32 字节密钥,加密明文,
/// 输出 base64(salt || nonce || ciphertext||tag)。
pub fn aes_gcm_encrypt(plaintext: &str, password: &str) -> ToolResult<String> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Key, Nonce};

    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];
    let mut rng = OsRng;
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut nonce);

    let mut key = [0u8; 32];
    pbkdf2_hmac_sha256(password.as_bytes(), &salt, 100_000, &mut key);

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
        .map_err(|e| ToolError::Other(e.to_string()))?;

    let mut blob = Vec::with_capacity(16 + 12 + ciphertext.len());
    blob.extend_from_slice(&salt);
    blob.extend_from_slice(&nonce);
    blob.extend_from_slice(&ciphertext);
    Ok(base64::engine::general_purpose::STANDARD.encode(&blob))
}

/// AES-256-GCM 解密:base64 解码后切出 salt/nonce/ciphertext,
/// 派生密钥并认证解密;数据过短或认证失败返回 [`ToolError::InvalidInput`]。
pub fn aes_gcm_decrypt(b64: &str, password: &str) -> ToolResult<String> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Key, Nonce};

    let blob = base64::engine::general_purpose::STANDARD.decode(b64.trim())?;
    if blob.len() < 16 + 12 {
        return Err(ToolError::InvalidInput(
            "解密失败:口令错误或数据损坏".into(),
        ));
    }
    let (salt, rest) = blob.split_at(16);
    let (nonce, ciphertext) = rest.split_at(12);

    let mut key = [0u8; 32];
    pbkdf2_hmac_sha256(password.as_bytes(), salt, 100_000, &mut key);

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| ToolError::InvalidInput("解密失败:口令错误或数据损坏".into()))?;
    String::from_utf8(plaintext).map_err(ToolError::from)
}

// RSA

/// 生成 RSA 密钥对(`bits` 建议 2048/4096,过小返回 Err),
/// 输出 PKCS#1 PEM:私钥块 + 空行 + 公钥块。
pub fn rsa_keygen(bits: usize) -> ToolResult<String> {
    use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey};
    use rsa::pkcs8::LineEnding;

    if bits < 2048 {
        return Err(ToolError::InvalidInput(format!(
            "密钥位数过小:建议 ≥ 2048,实际 {bits}"
        )));
    }

    let mut rng = OsRng;
    let priv_key =
        RsaPrivateKey::new(&mut rng, bits).map_err(|e| ToolError::Other(e.to_string()))?;
    let pub_key = RsaPublicKey::from(&priv_key);

    let priv_pem = priv_key
        .to_pkcs1_pem(LineEnding::LF)
        .map_err(|e| ToolError::Other(e.to_string()))?;
    let pub_pem = pub_key
        .to_pkcs1_pem(LineEnding::LF)
        .map_err(|e| ToolError::Other(e.to_string()))?;

    Ok(format!(
        "{}\n\n{}\n",
        priv_pem.trim_end(),
        pub_pem.trim_end()
    ))
}

/// 解析 RSA 公钥 PEM,兼容 PKCS#8/SPKI(`-----BEGIN PUBLIC KEY-----`)
/// 与 PKCS#1(`-----BEGIN RSA PUBLIC KEY-----`)两种格式
fn parse_rsa_public_key(pem: &str) -> ToolResult<RsaPublicKey> {
    use rsa::pkcs1::DecodeRsaPublicKey;
    use rsa::pkcs8::DecodePublicKey;
    if let Ok(key) = RsaPublicKey::from_public_key_pem(pem) {
        return Ok(key);
    }
    RsaPublicKey::from_pkcs1_pem(pem)
        .map_err(|e| ToolError::InvalidInput(format!("无效的 RSA 公钥 PEM:{e}")))
}

/// 解析 RSA 私钥 PEM,兼容 PKCS#8(`-----BEGIN PRIVATE KEY-----`)
/// 与 PKCS#1(`-----BEGIN RSA PRIVATE KEY-----`)两种格式
fn parse_rsa_private_key(pem: &str) -> ToolResult<RsaPrivateKey> {
    use rsa::pkcs1::DecodeRsaPrivateKey;
    use rsa::pkcs8::DecodePrivateKey;
    if let Ok(key) = RsaPrivateKey::from_pkcs8_pem(pem) {
        return Ok(key);
    }
    RsaPrivateKey::from_pkcs1_pem(pem)
        .map_err(|e| ToolError::InvalidInput(format!("无效的 RSA 私钥 PEM:{e}")))
}

/// RSA-OAEP(SHA256)公钥加密,输出 base64 密文
pub fn rsa_encrypt(input: &str, pub_pem: &str) -> ToolResult<String> {
    let pub_key = parse_rsa_public_key(pub_pem)?;
    let mut rng = OsRng;
    let ciphertext = pub_key
        .encrypt(&mut rng, Oaep::new::<Sha256>(), input.as_bytes())
        .map_err(|e| ToolError::Other(e.to_string()))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&ciphertext))
}

/// RSA-OAEP(SHA256)私钥解密
pub fn rsa_decrypt(b64: &str, priv_pem: &str) -> ToolResult<String> {
    let ciphertext = base64::engine::general_purpose::STANDARD.decode(b64.trim())?;
    let priv_key = parse_rsa_private_key(priv_pem)?;
    let plaintext = priv_key
        .decrypt(Oaep::new::<Sha256>(), &ciphertext)
        .map_err(|e| ToolError::Other(e.to_string()))?;
    String::from_utf8(plaintext).map_err(ToolError::from)
}

// KDF

/// PBKDF2-HMAC-SHA256 派生 32 字节密钥,返回十六进制字符串(小写)
pub fn kdf_pbkdf2(password: &str, salt: &str, iterations: u32) -> ToolResult<String> {
    let mut out = [0u8; 32];
    pbkdf2_hmac_sha256(password.as_bytes(), salt.as_bytes(), iterations, &mut out);
    Ok(hex::encode(out))
}

/// Argon2id 派生 32 字节哈希,返回十六进制字符串(小写);salt 至少 8 字节,过短返回 Err
pub fn kdf_argon2(password: &str, salt: &str) -> ToolResult<String> {
    use argon2::Argon2;

    if salt.len() < 8 {
        return Err(ToolError::InvalidInput(format!(
            "salt 过短:至少 8 字节,实际 {} 字节",
            salt.len()
        )));
    }

    let argon2 = Argon2::default();
    let mut out = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt.as_bytes(), &mut out)
        .map_err(|e| ToolError::Other(e.to_string()))?;
    Ok(hex::encode(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- AES-256-GCM ----

    #[test]
    fn aes_gcm_roundtrip() {
        for s in [
            "",
            "a",
            "Hello, NexToolkit!",
            "中文测试🎉",
            "多行\n文本\t含特殊字符",
        ] {
            let enc = aes_gcm_encrypt(s, "p@ssw0rd").unwrap();
            let dec = aes_gcm_decrypt(&enc, "p@ssw0rd").unwrap();
            assert_eq!(dec, s, "roundtrip 失败:明文 {s:?}");
        }
    }

    #[test]
    fn aes_gcm_wrong_password_fails() {
        let enc = aes_gcm_encrypt("secret", "right-password").unwrap();
        assert!(aes_gcm_decrypt(&enc, "wrong-password").is_err());
    }

    #[test]
    fn aes_gcm_ciphertext_is_nondeterministic() {
        // 随机 salt+nonce 导致同一明文+口令产生不同密文,但均可解回原明文
        let a = aes_gcm_encrypt("same", "pw").unwrap();
        let b = aes_gcm_encrypt("same", "pw").unwrap();
        assert_ne!(a, b);
        assert_eq!(aes_gcm_decrypt(&a, "pw").unwrap(), "same");
        assert_eq!(aes_gcm_decrypt(&b, "pw").unwrap(), "same");
    }

    #[test]
    fn aes_gcm_corrupted_input_fails() {
        // 数据过短:base64 解码后不足 28 字节(salt+nonce)
        assert!(aes_gcm_decrypt("AAAA", "pw").is_err());
        // 非 base64 字符
        assert!(aes_gcm_decrypt("!!!!不是合法base64!!!!", "pw").is_err());
    }

    // ---- RSA 密钥生成 ----

    #[test]
    fn rsa_keygen_2048_has_pem_markers() {
        let pem = rsa_keygen(2048).unwrap();
        assert!(pem.contains("-----BEGIN RSA PRIVATE KEY-----"));
        assert!(pem.contains("-----END RSA PRIVATE KEY-----"));
        assert!(pem.contains("-----BEGIN RSA PUBLIC KEY-----"));
        assert!(pem.contains("-----END RSA PUBLIC KEY-----"));
        // 私钥块与公钥块之间有空行
        assert!(pem.contains("-----END RSA PRIVATE KEY-----\n\n-----BEGIN RSA PUBLIC KEY-----"));
    }

    #[test]
    fn rsa_keygen_too_small_rejected() {
        assert!(rsa_keygen(512).is_err());
        assert!(rsa_keygen(1024).is_err());
    }

    #[test]
    fn rsa_keygen_pem_parseable() {
        // 生成的 PEM 可被反向解析回密钥对象(覆盖 parse_rsa_* 辅助函数)
        let pem = rsa_keygen(2048).unwrap();
        assert!(parse_rsa_private_key(&priv_pem_block(&pem)).is_ok());
        assert!(parse_rsa_public_key(&pub_pem_block(&pem)).is_ok());
    }

    // ---- RSA 加解密 ----

    #[test]
    fn rsa_encrypt_decrypt_roundtrip() {
        let pem = rsa_keygen(2048).unwrap();
        let priv_pem = priv_pem_block(&pem);
        let pub_pem = pub_pem_block(&pem);
        for msg in ["hello", "NexToolkit 中文测试", "x"] {
            let enc = rsa_encrypt(msg, &pub_pem).unwrap();
            let dec = rsa_decrypt(&enc, &priv_pem).unwrap();
            assert_eq!(dec, msg);
        }
    }

    #[test]
    fn rsa_encrypt_wrong_key_fails() {
        let k1 = rsa_keygen(2048).unwrap();
        let k2 = rsa_keygen(2048).unwrap();
        let enc = rsa_encrypt("data", &pub_pem_block(&k1)).unwrap();
        assert!(rsa_decrypt(&enc, &priv_pem_block(&k2)).is_err());
    }

    #[test]
    fn rsa_invalid_pem_rejected() {
        assert!(rsa_encrypt("x", "not a pem").is_err());
        assert!(rsa_encrypt("x", "").is_err());
    }

    // ---- PBKDF2-HMAC-SHA256 ----

    #[test]
    fn pbkdf2_matches_reference_vector() {
        // 官方 pbkdf2 crate 文档参考向量:PBKDF2-HMAC-SHA256("password","salt",600_000,20B)
        // 验证自实现与参考实现逐字节一致
        let mut buf = [0u8; 20];
        pbkdf2_hmac_sha256(b"password", b"salt", 600_000, &mut buf);
        assert_eq!(hex::encode(buf), "669cfe52482116fda1aa2cbe409b2f56c8e45637");
    }

    #[test]
    fn pbkdf2_deterministic() {
        let a = kdf_pbkdf2("password", "salt", 1000).unwrap();
        let b = kdf_pbkdf2("password", "salt", 1000).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.len(), 64); // 32 字节 = 64 hex 字符
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn pbkdf2_different_inputs_differ() {
        let base = kdf_pbkdf2("password", "salt", 1000).unwrap();
        assert_ne!(base, kdf_pbkdf2("password", "salt", 2000).unwrap());
        assert_ne!(base, kdf_pbkdf2("password2", "salt", 1000).unwrap());
        assert_ne!(base, kdf_pbkdf2("password", "salt2", 1000).unwrap());
    }

    // ---- Argon2id ----

    #[test]
    fn argon2_deterministic() {
        let a = kdf_argon2("password", "saltsalt").unwrap();
        let b = kdf_argon2("password", "saltsalt").unwrap();
        assert_eq!(a, b);
        assert_eq!(a.len(), 64); // 32 字节 = 64 hex 字符
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn argon2_different_inputs_differ() {
        // 不同 salt 产出不同哈希;不同口令亦然
        let a = kdf_argon2("password", "saltsalt").unwrap();
        let b = kdf_argon2("password", "saltsalt2").unwrap();
        let c = kdf_argon2("password2", "saltsalt").unwrap();
        assert_ne!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn argon2_short_salt_rejected() {
        assert!(kdf_argon2("password", "short").is_err()); // 5 字节
        assert!(kdf_argon2("password", "").is_err()); // 0 字节
                                                      // 恰好 8 字节应通过
        assert!(kdf_argon2("password", "12345678").is_ok());
    }

    // ---- 测试辅助:从 keygen 输出中拆出 PEM 块 ----

    fn priv_pem_block(keygen: &str) -> String {
        let begin = "-----BEGIN RSA PRIVATE KEY-----";
        let end = "-----END RSA PRIVATE KEY-----";
        let s = keygen.find(begin).expect("含私钥块");
        let e = keygen.find(end).expect("含私钥结束标记") + end.len();
        keygen[s..e].to_string()
    }

    fn pub_pem_block(keygen: &str) -> String {
        let begin = "-----BEGIN RSA PUBLIC KEY-----";
        let end = "-----END RSA PUBLIC KEY-----";
        let s = keygen.find(begin).expect("含公钥块");
        let e = keygen.find(end).expect("含公钥结束标记") + end.len();
        keygen[s..e].to_string()
    }
}
