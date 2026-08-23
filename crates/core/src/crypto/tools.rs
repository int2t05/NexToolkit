//! 工具注册:加密工具的元数据 + 字符串参数适配,供 registry 聚合

use crate::registry::{OutputKind, ParamKind, ParamSpec, Tool, ToolArgs, ToolMeta};

use super::{
    aes_gcm_decrypt, aes_gcm_encrypt, bcrypt_hash, bcrypt_verify, chacha20_decrypt,
    chacha20_encrypt, crc32, crc64, ed25519_keygen, ed25519_sign, ed25519_verify, hmac_multi,
    kdf_argon2, kdf_pbkdf2, rsa_decrypt, rsa_encrypt, rsa_keygen, rsa_sign, rsa_verify,
    scrypt_hash, scrypt_verify, HmacAlgo,
};

pub struct AesGcmEncrypt;
impl Tool for AesGcmEncrypt {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "aes_gcm_encrypt",
            name: "AES 加密",
            desc: "AES-256-GCM",
            group: "crypto",
            params: &[ParamSpec {
                key: "password",
                kind: ParamKind::Password,
                label: "口令",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let password = args.get_str("password")?;
        aes_gcm_encrypt(input, password)
    }
}

pub struct AesGcmDecrypt;
impl Tool for AesGcmDecrypt {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "aes_gcm_decrypt",
            name: "AES 解密",
            desc: "AES-256-GCM",
            group: "crypto",
            params: &[ParamSpec {
                key: "password",
                kind: ParamKind::Password,
                label: "口令",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let password = args.get_str("password")?;
        aes_gcm_decrypt(input, password)
    }
}

pub struct RsaKeygen;
impl Tool for RsaKeygen {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "rsa_keygen",
            name: "RSA 密钥对",
            desc: "生成 PEM 密钥对",
            group: "crypto",
            params: &[ParamSpec {
                key: "bits",
                kind: ParamKind::Number,
                label: "位数",
                default: Some("2048"),
                options: &[],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: false,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, _input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let bits = args.get_u32("bits")? as usize;
        rsa_keygen(bits)
    }
}

pub struct RsaEncrypt;
impl Tool for RsaEncrypt {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "rsa_encrypt",
            name: "RSA 加密",
            desc: "RSA-OAEP/SHA256",
            group: "crypto",
            params: &[ParamSpec {
                key: "pubPem",
                kind: ParamKind::Textarea,
                label: "公钥 PEM",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let pub_pem = args.get_str("pubPem")?;
        rsa_encrypt(input, pub_pem)
    }
}

pub struct RsaDecrypt;
impl Tool for RsaDecrypt {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "rsa_decrypt",
            name: "RSA 解密",
            desc: "RSA-OAEP/SHA256",
            group: "crypto",
            params: &[ParamSpec {
                key: "privPem",
                kind: ParamKind::Textarea,
                label: "私钥 PEM",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let priv_pem = args.get_str("privPem")?;
        rsa_decrypt(input, priv_pem)
    }
}

pub struct RsaSign;
impl Tool for RsaSign {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "rsa_sign",
            name: "RSA 签名",
            desc: "PKCS1v15/SHA256",
            group: "crypto",
            params: &[ParamSpec {
                key: "privPem",
                kind: ParamKind::Textarea,
                label: "私钥 PEM",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let priv_pem = args.get_str("privPem")?;
        rsa_sign(input, priv_pem)
    }
}

pub struct RsaVerify;
impl Tool for RsaVerify {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "rsa_verify",
            name: "RSA 验签",
            desc: "PKCS1v15/SHA256",
            group: "crypto",
            params: &[
                ParamSpec {
                    key: "pubPem",
                    kind: ParamKind::Textarea,
                    label: "公钥 PEM",
                    default: None,
                    options: &[],
                    placeholder: None,
                    multiple: false,
                },
                ParamSpec {
                    key: "signature",
                    kind: ParamKind::Text,
                    label: "签名(base64)",
                    default: None,
                    options: &[],
                    placeholder: None,
                    multiple: false,
                },
            ],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let pub_pem = args.get_str("pubPem")?;
        let signature = args.get_str("signature")?;
        rsa_verify(input, pub_pem, signature).map(|_| "验签成功".to_string())
    }
}

pub struct KdfPbkdf2;
impl Tool for KdfPbkdf2 {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "pbkdf2",
            name: "PBKDF2",
            desc: "PBKDF2 密钥派生",
            group: "crypto",
            params: &[
                ParamSpec {
                    key: "salt",
                    kind: ParamKind::Text,
                    label: "salt",
                    default: None,
                    options: &[],
                    placeholder: None,
                    multiple: false,
                },
                ParamSpec {
                    key: "iterations",
                    kind: ParamKind::Number,
                    label: "迭代",
                    default: Some("100000"),
                    options: &[],
                    placeholder: None,
                    multiple: false,
                },
            ],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let salt = args.get_str("salt")?;
        let iterations = args.get_u32("iterations")?;
        kdf_pbkdf2(input, salt, iterations)
    }
}

pub struct KdfArgon2;
impl Tool for KdfArgon2 {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "argon2",
            name: "Argon2",
            desc: "Argon2 密钥派生",
            group: "crypto",
            params: &[ParamSpec {
                key: "salt",
                kind: ParamKind::Text,
                label: "salt(≥8字节)",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let salt = args.get_str("salt")?;
        kdf_argon2(input, salt)
    }
}

pub struct ChaCha20Encrypt;
impl Tool for ChaCha20Encrypt {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "chacha20_encrypt",
            name: "ChaCha20 加密",
            desc: "ChaCha20-Poly1305",
            group: "crypto",
            params: &[ParamSpec {
                key: "password",
                kind: ParamKind::Password,
                label: "口令",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let password = args.get_str("password")?;
        chacha20_encrypt(input, password)
    }
}

pub struct ChaCha20Decrypt;
impl Tool for ChaCha20Decrypt {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "chacha20_decrypt",
            name: "ChaCha20 解密",
            desc: "ChaCha20-Poly1305",
            group: "crypto",
            params: &[ParamSpec {
                key: "password",
                kind: ParamKind::Password,
                label: "口令",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let password = args.get_str("password")?;
        chacha20_decrypt(input, password)
    }
}

pub struct Ed25519Keygen;
impl Tool for Ed25519Keygen {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "ed25519_keygen",
            name: "Ed25519 密钥对",
            desc: "生成 PEM 密钥对",
            group: "crypto",
            params: &[],
            needs_main_input: false,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, _input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        ed25519_keygen()
    }
}

pub struct Ed25519Sign;
impl Tool for Ed25519Sign {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "ed25519_sign",
            name: "Ed25519 签名",
            desc: "EdDSA 签名",
            group: "crypto",
            params: &[ParamSpec {
                key: "privPem",
                kind: ParamKind::Textarea,
                label: "私钥 PEM",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let priv_pem = args.get_str("privPem")?;
        ed25519_sign(input, priv_pem)
    }
}

pub struct Ed25519Verify;
impl Tool for Ed25519Verify {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "ed25519_verify",
            name: "Ed25519 验签",
            desc: "EdDSA 验签",
            group: "crypto",
            params: &[
                ParamSpec {
                    key: "pubPem",
                    kind: ParamKind::Textarea,
                    label: "公钥 PEM",
                    default: None,
                    options: &[],
                    placeholder: None,
                    multiple: false,
                },
                ParamSpec {
                    key: "signature",
                    kind: ParamKind::Text,
                    label: "签名(base64)",
                    default: None,
                    options: &[],
                    placeholder: None,
                    multiple: false,
                },
            ],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let pub_pem = args.get_str("pubPem")?;
        let signature = args.get_str("signature")?;
        ed25519_verify(input, pub_pem, signature).map(|_| "验签成功".to_string())
    }
}

pub struct BcryptHash;
impl Tool for BcryptHash {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "bcrypt_hash",
            name: "Bcrypt 哈希",
            desc: "口令哈希",
            group: "crypto",
            params: &[ParamSpec {
                key: "cost",
                kind: ParamKind::Number,
                label: "代价因子",
                default: Some("12"),
                options: &[],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let cost = args.get_u32("cost")?;
        bcrypt_hash(input, cost)
    }
}

pub struct BcryptVerify;
impl Tool for BcryptVerify {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "bcrypt_verify",
            name: "Bcrypt 验证",
            desc: "口令比对",
            group: "crypto",
            params: &[ParamSpec {
                key: "hash",
                kind: ParamKind::Text,
                label: "Bcrypt 哈希",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let hash = args.get_str("hash")?;
        bcrypt_verify(input, hash).map(|_| "验证成功".to_string())
    }
}

pub struct ScryptHash;
impl Tool for ScryptHash {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "scrypt_hash",
            name: "Scrypt",
            desc: "Scrypt 密钥派生",
            group: "crypto",
            params: &[ParamSpec {
                key: "salt",
                kind: ParamKind::Text,
                label: "salt(≥8字节)",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let salt = args.get_str("salt")?;
        scrypt_hash(input, salt)
    }
}

pub struct ScryptVerify;
impl Tool for ScryptVerify {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "scrypt_verify",
            name: "Scrypt 验证",
            desc: "口令比对",
            group: "crypto",
            params: &[
                ParamSpec {
                    key: "salt",
                    kind: ParamKind::Text,
                    label: "salt",
                    default: None,
                    options: &[],
                    placeholder: None,
                    multiple: false,
                },
                ParamSpec {
                    key: "hash",
                    kind: ParamKind::Text,
                    label: "期望哈希(hex)",
                    default: None,
                    options: &[],
                    placeholder: None,
                    multiple: false,
                },
            ],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let salt = args.get_str("salt")?;
        let hash = args.get_str("hash")?;
        scrypt_verify(input, salt, hash).map(|_| "验证成功".to_string())
    }
}

pub struct HmacMulti;
impl Tool for HmacMulti {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "hmac_sha2",
            name: "HMAC-SHA2",
            desc: "SHA224/384/512",
            group: "crypto",
            params: &[
                ParamSpec {
                    key: "algo",
                    kind: ParamKind::Select,
                    label: "算法",
                    default: Some("sha512"),
                    options: &["sha224", "sha384", "sha512"],
                    placeholder: None,
                    multiple: false,
                },
                ParamSpec {
                    key: "key",
                    kind: ParamKind::Text,
                    label: "密钥",
                    default: None,
                    options: &[],
                    placeholder: Some("HMAC 密钥"),
                    multiple: false,
                },
            ],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let algo = args.get_enum::<HmacAlgo>("algo")?;
        let key = args.get_str("key")?;
        hmac_multi(input, key, algo)
    }
}

pub struct Crc32;
impl Tool for Crc32 {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "crc32",
            name: "CRC32",
            desc: "ISO-HDLC(zlib)",
            group: "crypto",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        crc32(input)
    }
}

pub struct Crc64;
impl Tool for Crc64 {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "crc64",
            name: "CRC64",
            desc: "XZ 多项式",
            group: "crypto",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        crc64(input)
    }
}
