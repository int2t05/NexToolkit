//! 工具注册:转换工具的元数据 + 字符串参数适配,供 registry 聚合

use crate::registry::{OutputKind, ParamKind, ParamSpec, Tool, ToolArgs, ToolMeta};

use super::{
    csv_to_json, json_to_csv, json_to_toml, json_to_yaml, md_to_html, numbase_convert,
    toml_to_json, yaml_to_json,
};

pub struct JsonToYaml;
impl Tool for JsonToYaml {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "json_to_yaml",
            name: "JSON → YAML",
            desc: "JSON 转 YAML",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("yaml"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        json_to_yaml(input)
    }
}

pub struct YamlToJson;
impl Tool for YamlToJson {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "yaml_to_json",
            name: "YAML → JSON",
            desc: "YAML 转 JSON",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("json"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        yaml_to_json(input)
    }
}

pub struct JsonToToml;
impl Tool for JsonToToml {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "json_to_toml",
            name: "JSON → TOML",
            desc: "JSON 转 TOML",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("toml"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        json_to_toml(input)
    }
}

pub struct TomlToJson;
impl Tool for TomlToJson {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "toml_to_json",
            name: "TOML → JSON",
            desc: "TOML 转 JSON",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("json"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        toml_to_json(input)
    }
}

pub struct JsonToCsv;
impl Tool for JsonToCsv {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "json_to_csv",
            name: "JSON → CSV",
            desc: "JSON 数组转 CSV",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        json_to_csv(input)
    }
}

pub struct CsvToJson;
impl Tool for CsvToJson {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "csv_to_json",
            name: "CSV → JSON",
            desc: "CSV 转 JSON 数组",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("json"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        csv_to_json(input)
    }
}

pub struct MdToHtml;
impl Tool for MdToHtml {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "md_to_html",
            name: "Markdown → HTML",
            desc: "Markdown 转 HTML",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("xml"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        md_to_html(input)
    }
}

pub struct NumbaseConvert;
impl Tool for NumbaseConvert {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "numbase_convert",
            name: "进制转换",
            desc: "任意进制互转(2..36)",
            group: "convert",
            params: &[
                ParamSpec {
                    key: "from",
                    kind: ParamKind::Number,
                    label: "源进制",
                    default: Some("10"),
                    options: &[],
                    placeholder: None,
                    multiple: false,
                },
                ParamSpec {
                    key: "to",
                    kind: ParamKind::Number,
                    label: "目标进制",
                    default: Some("16"),
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
        let from = args.get_u32("from")?;
        let to = args.get_u32("to")?;
        numbase_convert(input, from, to)
    }
}
