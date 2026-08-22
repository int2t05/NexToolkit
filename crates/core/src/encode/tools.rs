//! 工具注册:编解码工具的元数据 + 字符串参数适配,供 registry 聚合

use crate::registry::{OutputKind, ParamKind, ParamSpec, Tool, ToolArgs, ToolMeta};

use super::{
    base32_decode, base32_encode, base58_decode, base58_encode, base64_decode, base64_encode,
    base85_decode, base85_encode, braille_decode, braille_encode, hex_decode, hex_encode,
    html_decode, html_encode, jwt_decode, jwt_verify, morse_decode, morse_encode, punycode_decode,
    punycode_encode, quoted_printable_decode, quoted_printable_encode, url_decode, url_encode,
    zero_width_decode, zero_width_encode,
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

pub struct Base32Encode;
impl Tool for Base32Encode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "base32_encode",
            name: "Base32 编码",
            desc: "RFC 4648 标准 Base32 编码(A-Z2-7,含 padding)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        base32_encode(input)
    }
}

pub struct Base32Decode;
impl Tool for Base32Decode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "base32_decode",
            name: "Base32 解码",
            desc: "RFC 4648 标准 Base32 解码(容忍大小写与首尾空白)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        base32_decode(input)
    }
}

pub struct Base58Encode;
impl Tool for Base58Encode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "base58_encode",
            name: "Base58 编码",
            desc: "Bitcoin 字母表 Base58 编码",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        base58_encode(input)
    }
}

pub struct Base58Decode;
impl Tool for Base58Decode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "base58_decode",
            name: "Base58 解码",
            desc: "Bitcoin 字母表 Base58 解码(容忍首尾空白)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        base58_decode(input)
    }
}

pub struct Base85Encode;
impl Tool for Base85Encode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "base85_encode",
            name: "Base85 编码",
            desc: "Ascii85 编码(Adobe 变体,`!` 起始,`z` 零字节简写)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        base85_encode(input)
    }
}

pub struct Base85Decode;
impl Tool for Base85Decode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "base85_decode",
            name: "Base85 解码",
            desc: "Ascii85 解码(Adobe 变体,含 `z` 零字节简写)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        base85_decode(input)
    }
}

pub struct PunycodeEncode;
impl Tool for PunycodeEncode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "punycode_encode",
            name: "Punycode 编码",
            desc: "Unicode 域名标签编码为 xn-- 前缀的 Punycode(RFC 3492)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        punycode_encode(input)
    }
}

pub struct PunycodeDecode;
impl Tool for PunycodeDecode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "punycode_decode",
            name: "Punycode 解码",
            desc: "xn-- Punycode 域名标签解码为 Unicode(RFC 3492)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        punycode_decode(input)
    }
}

pub struct QuotedPrintableEncode;
impl Tool for QuotedPrintableEncode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "quoted_printable_encode",
            name: "Quoted-Printable 编码",
            desc: "RFC 2045 Quoted-Printable 编码(ASCII 安全传输)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        quoted_printable_encode(input)
    }
}

pub struct QuotedPrintableDecode;
impl Tool for QuotedPrintableDecode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "quoted_printable_decode",
            name: "Quoted-Printable 解码",
            desc: "RFC 2045 Quoted-Printable 解码",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        quoted_printable_decode(input)
    }
}

pub struct MorseEncode;
impl Tool for MorseEncode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "morse_encode",
            name: "Morse 编码",
            desc: "国际摩斯码编码(A-Z 0-9,字母间空格,单词间 / )",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        morse_encode(input)
    }
}

pub struct MorseDecode;
impl Tool for MorseDecode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "morse_decode",
            name: "Morse 解码",
            desc: "国际摩斯码解码(点划 → 文本)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        morse_decode(input)
    }
}

pub struct BrailleEncode;
impl Tool for BrailleEncode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "braille_encode",
            name: "Braille 编码",
            desc: "文本 → Unicode 盲文字符(a-z + 空格,6 点基本盲文)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        braille_encode(input)
    }
}

pub struct BrailleDecode;
impl Tool for BrailleDecode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "braille_decode",
            name: "Braille 解码",
            desc: "Unicode 盲文字符 → 文本",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        braille_decode(input)
    }
}

pub struct ZeroWidthEncode;
impl Tool for ZeroWidthEncode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "zero_width_encode",
            name: "零宽字符隐写编码",
            desc: "文本 → 零宽字符序列(U+200B=0,U+200C=1,每字节 8 字符)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        zero_width_encode(input)
    }
}

pub struct ZeroWidthDecode;
impl Tool for ZeroWidthDecode {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "zero_width_decode",
            name: "零宽字符隐写解码",
            desc: "零宽字符序列 → 文本(提取 U+200B/U+200C 隐写数据)",
            group: "encode",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        zero_width_decode(input)
    }
}
