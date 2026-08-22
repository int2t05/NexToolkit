//! 工具注册表:文本工具的自描述元数据 + 统一执行入口
//!
//! 每个文本工具实现 [`Tool`] trait,元数据(id/名称/分组/参数 schema)与逻辑一体。
//! [`tools()`] 返回全部注册工具,供 GUI 动态渲染;[`find_tool()`] 按 id 查找供 `run_tool` 分发。
//! 现有裸函数保留(供 CLI 类型化直接调用),trait impl 是元数据 + 字符串参数适配层。
//! 仅覆盖 str→str 文本工具;文件工具(字节域/路径)I/O 模型不同,不进此 trait。

use crate::{ToolError, ToolResult};

/// 文本工具端口:元数据 + 字符串参数执行
///
/// 仅覆盖 str→str 文本工具(编码/文本/加密/格式化/单位/生成器)。
/// 文件工具(archive/image/pdf 字节域、文件路径、多产物)I/O 模型不同,不进此 trait。
pub trait Tool: Send + Sync {
    /// 静态元数据(所有字段 &'static,可 const 构造)
    fn meta(&self) -> &'static ToolMeta;
    /// 执行:主输入 + 命名参数(字符串),返回字符串结果
    fn run(&self, input: &str, args: &ToolArgs) -> ToolResult<String>;
}

/// 工具元数据(静态:所有字段为 &'static,可在 `static` 项中构造)
#[derive(Debug)]
pub struct ToolMeta {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub group: &'static str,
    pub params: &'static [ParamSpec],
    pub needs_main_input: bool,
    pub output_kind: OutputKind,
}

/// 参数规格(UI 渲染 + 字符串解析)
#[derive(Debug)]
pub struct ParamSpec {
    pub key: &'static str,
    pub kind: ParamKind,
    pub label: &'static str,
    pub default: Option<&'static str>,
    pub options: &'static [&'static str],
    pub placeholder: Option<&'static str>,
    pub multiple: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ParamKind {
    Text,
    Textarea,
    Select,
    Number,
    Password,
    Bool,
    File,
}

/// 输出类型(供前端决定高亮/渲染)
#[derive(Debug, Clone, Copy)]
pub enum OutputKind {
    /// 纯文本不高亮
    Text,
    /// highlight.js 语言名:"json"/"sql"/"xml"/"yaml"
    Highlight(&'static str),
    /// SVG 直接渲染
    Svg,
}

/// 参数访问 helper:封装类型强转,消除每工具手写 parse
pub struct ToolArgs<'a> {
    pairs: &'a [(String, String)],
}

impl<'a> ToolArgs<'a> {
    pub fn new(pairs: &'a [(String, String)]) -> Self {
        Self { pairs }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn get_str(&self, key: &str) -> ToolResult<&str> {
        self.get(key)
            .ok_or_else(|| ToolError::InvalidInput(format!("缺少参数: {key}")))
    }

    pub fn get_u32(&self, key: &str) -> ToolResult<u32> {
        self.get_str(key)?
            .parse()
            .map_err(|_| ToolError::Parse(format!("参数 {key} 需为正整数")))
    }

    pub fn get_bool(&self, key: &str) -> bool {
        self.get(key)
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }

    pub fn get_enum<T: std::str::FromStr>(&self, key: &str) -> ToolResult<T>
    where
        T::Err: std::fmt::Display,
    {
        self.get_str(key)?
            .parse()
            .map_err(|e| ToolError::Parse(format!("参数 {key}: {e}")))
    }
}

/// 全部注册文本工具(各种类模块定义 struct 并在此聚合)
pub fn tools() -> &'static [&'static dyn Tool] {
    &[
        // encode
        &crate::encode::Base64Encode,
        &crate::encode::Base64Decode,
        &crate::encode::UrlEncode,
        &crate::encode::UrlDecode,
        &crate::encode::HtmlEncode,
        &crate::encode::HtmlDecode,
        &crate::encode::HexEncode,
        &crate::encode::HexDecode,
        &crate::encode::JwtDecode,
        &crate::encode::JwtVerify,
        &crate::encode::Base32Encode,
        &crate::encode::Base32Decode,
        &crate::encode::Base58Encode,
        &crate::encode::Base58Decode,
        &crate::encode::Base85Encode,
        &crate::encode::Base85Decode,
        &crate::encode::PunycodeEncode,
        &crate::encode::PunycodeDecode,
        &crate::encode::QuotedPrintableEncode,
        &crate::encode::QuotedPrintableDecode,
        &crate::encode::MorseEncode,
        &crate::encode::MorseDecode,
        &crate::encode::BrailleEncode,
        &crate::encode::BrailleDecode,
        &crate::encode::ZeroWidthEncode,
        &crate::encode::ZeroWidthDecode,
        // convert
        &crate::convert::JsonToYaml,
        &crate::convert::YamlToJson,
        &crate::convert::JsonToToml,
        &crate::convert::TomlToJson,
        &crate::convert::JsonToCsv,
        &crate::convert::CsvToJson,
        &crate::convert::MdToHtml,
        &crate::convert::NumbaseConvert,
        &crate::convert::CsvToTsv,
        &crate::convert::TsvToCsv,
        &crate::convert::CsvToYaml,
        &crate::convert::YamlToCsv,
        &crate::convert::CsvToXml,
        &crate::convert::XmlToCsv,
        &crate::convert::TsvToJson,
        &crate::convert::JsonToTsv,
        &crate::convert::JsonToXml,
        &crate::convert::XmlToJson,
        &crate::convert::YamlToToml,
        &crate::convert::TomlToYaml,
        &crate::convert::YamlToXml,
        &crate::convert::XmlToYaml,
        &crate::convert::TomlToXml,
        &crate::convert::XmlToToml,
        &crate::convert::MdToTxt,
        // format
        &crate::format::JsonFormat,
        &crate::format::JsonMinify,
        &crate::format::SqlFormat,
        &crate::format::XmlFormat,
        &crate::format::XmlMinify,
        &crate::format::CssMinify,
        // generate
        &crate::generate::Hash,
        &crate::generate::HmacCompute,
        &crate::generate::UuidV4,
        &crate::generate::UuidV7,
        &crate::generate::PasswordGenerate,
        &crate::generate::LoremIpsum,
        &crate::generate::QrSvg,
        // text
        &crate::text::CaseConvert,
        &crate::text::SortLines,
        &crate::text::DedupLines,
        &crate::text::ReverseText,
        &crate::text::RegexMatch,
        &crate::text::RegexReplace,
        &crate::text::DiffText,
        // crypto
        &crate::crypto::AesGcmEncrypt,
        &crate::crypto::AesGcmDecrypt,
        &crate::crypto::RsaKeygen,
        &crate::crypto::RsaEncrypt,
        &crate::crypto::RsaDecrypt,
        &crate::crypto::RsaSign,
        &crate::crypto::RsaVerify,
        &crate::crypto::KdfPbkdf2,
        &crate::crypto::KdfArgon2,
        // nettime
        &crate::nettime::Ipcalc,
        &crate::nettime::TimestampToHuman,
        &crate::nettime::TimestampFromHuman,
        &crate::nettime::CronNext,
        &crate::nettime::DnsLookup,
        &crate::http::HttpProbe,
        // unit(group=convert)
        &crate::unit::UnitConvert,
    ]
}

/// 按 id 查找工具
pub fn find_tool(id: &str) -> Option<&'static dyn Tool> {
    tools().iter().copied().find(|t| t.meta().id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_tool_by_id() {
        let t = find_tool("base64_encode").expect("base64_encode 应已注册");
        assert_eq!(t.meta().name, "Base64 编码");
        assert_eq!(t.meta().group, "encode");
    }

    #[test]
    fn unknown_tool_not_found() {
        assert!(find_tool("nonexistent").is_none());
    }

    #[test]
    fn run_registered_tool() {
        let t = find_tool("base64_encode").unwrap();
        let args = ToolArgs::new(&[]);
        assert_eq!(t.run("hello", &args).unwrap(), "aGVsbG8=");
    }

    #[test]
    fn run_tool_with_param() {
        let t = find_tool("jwt_verify").unwrap();
        // 缺 key 参数应报错
        let args = ToolArgs::new(&[]);
        assert!(t.run("x.y.z", &args).is_err());
    }
}
