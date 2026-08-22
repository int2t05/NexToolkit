//! 工具注册:格式化工具的元数据 + 字符串参数适配,供 registry 聚合

use crate::registry::{OutputKind, Tool, ToolArgs, ToolMeta};

use super::{css_minify, json_format, json_minify, sql_format, xml_format, xml_minify};

pub struct JsonFormat;
impl Tool for JsonFormat {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "json_format",
            name: "JSON 美化",
            desc: "2 空格缩进",
            group: "format",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("json"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        json_format(input)
    }
}

pub struct JsonMinify;
impl Tool for JsonMinify {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "json_minify",
            name: "JSON 压缩",
            desc: "紧凑输出",
            group: "format",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("json"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        json_minify(input)
    }
}

pub struct SqlFormat;
impl Tool for SqlFormat {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "sql_format",
            name: "SQL 美化",
            desc: "关键字大写",
            group: "format",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("sql"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        sql_format(input)
    }
}

pub struct XmlFormat;
impl Tool for XmlFormat {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "xml_format",
            name: "XML 美化",
            desc: "2 空格缩进",
            group: "format",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("xml"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        xml_format(input)
    }
}

pub struct XmlMinify;
impl Tool for XmlMinify {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "xml_minify",
            name: "XML 压缩",
            desc: "去空白",
            group: "format",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("xml"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        xml_minify(input)
    }
}

pub struct CssMinify;
impl Tool for CssMinify {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "css_minify",
            name: "CSS 压缩",
            desc: "去注释空白",
            group: "format",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("css"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        css_minify(input)
    }
}
