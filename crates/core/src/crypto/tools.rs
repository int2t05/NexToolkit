//! 工具注册:加密工具的元数据 + 字符串参数适配,供 registry 聚合

use crate::registry::{OutputKind, ParamKind, ParamSpec, Tool, ToolArgs, ToolMeta};

use super::{
    aes_gcm_decrypt, aes_gcm_encrypt, kdf_argon2, kdf_pbkdf2, rsa_decrypt, rsa_encrypt, rsa_keygen,
    rsa_sign, rsa_verify,
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
            desc: "密钥派生",
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
            desc: "Argon2id 派生",
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
