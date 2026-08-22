//! 工具注册:编解码工具的元数据 + 字符串参数适配,供 registry 聚合

use crate::registry::{OutputKind, ParamKind, ParamSpec, Tool, ToolArgs, ToolMeta};

use super::{
    base64_decode, base64_encode, hex_decode, hex_encode, html_decode, html_encode, jwt_decode,
    jwt_verify, url_decode, url_encode,
};

pub struct Base64Encode;
impl Tool for Base64Encode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "base64_encode",
            name: "Base64 编码",
            desc: "Base64 标准编码(含 padding)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        base64_encode(input)
    }
}

pub struct Base64Decode;
impl Tool for Base64Decode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "base64_decode",
            name: "Base64 解码",
            desc: "Base64 标准解码(容忍首尾空白)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        base64_decode(input)
    }
}

pub struct UrlEncode;
impl Tool for UrlEncode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "url_encode",
            name: "URL 编码",
            desc: "百分号编码",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        url_encode(input)
    }
}

pub struct UrlDecode;
impl Tool for UrlDecode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "url_decode",
            name: "URL 解码",
            desc: "百分号解码",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        url_decode(input)
    }
}

pub struct HtmlEncode;
impl Tool for HtmlEncode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "html_encode",
            name: "HTML 编码",
            desc: "HTML 实体编码",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        html_encode(input)
    }
}

pub struct HtmlDecode;
impl Tool for HtmlDecode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "html_decode",
            name: "HTML 解码",
            desc: "HTML 实体解码",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        html_decode(input)
    }
}

pub struct HexEncode;
impl Tool for HexEncode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "hex_encode",
            name: "Hex 编码",
            desc: "十六进制编码",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        hex_encode(input)
    }
}

pub struct HexDecode;
impl Tool for HexDecode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "hex_decode",
            name: "Hex 解码",
            desc: "十六进制解码",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        hex_decode(input)
    }
}

pub struct JwtDecode;
impl Tool for JwtDecode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "jwt_decode",
            name: "JWT 解码",
            desc: "解析 header/payload(不验签)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("json"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        jwt_decode(input)
    }
}

pub struct JwtVerify;
impl Tool for JwtVerify {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "jwt_verify",
            name: "JWT 验签",
            desc: "HS256/RS256 验证签名",
            group: "encode",
            params: &[ParamSpec {
                key: "key",
                kind: ParamKind::Textarea,
                label: "密钥(HS256 secret 或 RS256 公钥 PEM)",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("json"),
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let key = args.get_str("key")?;
        jwt_verify(input, key)
    }
}
