//! 工具注册:生成器工具的元数据 + 字符串参数适配,供 registry 聚合

use crate::registry::{OutputKind, ParamKind, ParamSpec, Tool, ToolArgs, ToolMeta};

use super::{
    hash, hmac_compute, lorem_ipsum, password_generate, qr_svg, uuid_v4, uuid_v7, HashAlgo,
    PasswordOpts,
};

pub struct Hash;
impl Tool for Hash {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "hash",
            name: "哈希",
            desc: "MD5/SHA1/SHA256/SHA512",
            group: "generate",
            params: &[ParamSpec {
                key: "algo",
                kind: ParamKind::Select,
                label: "算法",
                default: Some("sha256"),
                options: &["md5", "sha1", "sha256", "sha512"],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let algo = args.get_enum::<HashAlgo>("algo")?;
        hash(input, algo)
    }
}

pub struct HmacCompute;
impl Tool for HmacCompute {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "hmac_compute",
            name: "HMAC",
            desc: "HMAC 消息认证码",
            group: "generate",
            params: &[
                ParamSpec {
                    key: "algo",
                    kind: ParamKind::Select,
                    label: "算法",
                    default: Some("sha256"),
                    options: &["md5", "sha1", "sha256", "sha512"],
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
        let algo = args.get_enum::<HashAlgo>("algo")?;
        let key = args.get_str("key")?;
        hmac_compute(input, key, algo)
    }
}

pub struct UuidV4;
impl Tool for UuidV4 {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "uuid_v4",
            name: "UUID v4",
            desc: "随机 UUID",
            group: "generate",
            params: &[],
            needs_main_input: false,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, _input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        uuid_v4()
    }
}

pub struct UuidV7;
impl Tool for UuidV7 {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "uuid_v7",
            name: "UUID v7",
            desc: "基于时间戳",
            group: "generate",
            params: &[],
            needs_main_input: false,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, _input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        uuid_v7()
    }
}

pub struct PasswordGenerate;
impl Tool for PasswordGenerate {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "password_generate",
            name: "密码生成",
            desc: "随机密码",
            group: "generate",
            params: &[
                ParamSpec {
                    key: "length",
                    kind: ParamKind::Number,
                    label: "长度",
                    default: Some("16"),
                    options: &[],
                    placeholder: None,
                    multiple: false,
                },
                ParamSpec {
                    key: "upper",
                    kind: ParamKind::Select,
                    label: "大写",
                    default: Some("true"),
                    options: &["true", "false"],
                    placeholder: None,
                    multiple: false,
                },
                ParamSpec {
                    key: "lower",
                    kind: ParamKind::Select,
                    label: "小写",
                    default: Some("true"),
                    options: &["true", "false"],
                    placeholder: None,
                    multiple: false,
                },
                ParamSpec {
                    key: "digits",
                    kind: ParamKind::Select,
                    label: "数字",
                    default: Some("true"),
                    options: &["true", "false"],
                    placeholder: None,
                    multiple: false,
                },
                ParamSpec {
                    key: "symbols",
                    kind: ParamKind::Select,
                    label: "符号",
                    default: Some("false"),
                    options: &["true", "false"],
                    placeholder: None,
                    multiple: false,
                },
            ],
            needs_main_input: false,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, _input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let length = args.get_u32("length")? as usize;
        let opts = PasswordOpts {
            upper: args.get_bool("upper"),
            lower: args.get_bool("lower"),
            digits: args.get_bool("digits"),
            symbols: args.get_bool("symbols"),
        };
        password_generate(length, &opts)
    }
}

pub struct LoremIpsum;
impl Tool for LoremIpsum {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "lorem_ipsum",
            name: "Lorem Ipsum",
            desc: "占位文本",
            group: "generate",
            params: &[ParamSpec {
                key: "paragraphs",
                kind: ParamKind::Number,
                label: "段落数",
                default: Some("3"),
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
        let paragraphs = args.get_u32("paragraphs")? as usize;
        lorem_ipsum(paragraphs)
    }
}

pub struct QrSvg;
impl Tool for QrSvg {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "qr_svg",
            name: "二维码 SVG",
            desc: "生成 SVG 二维码",
            group: "generate",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Svg,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        qr_svg(input)
    }
}
