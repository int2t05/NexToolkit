//! 工具注册:转换工具的元数据 + 字符串参数适配,供 registry 聚合

use crate::registry::{OutputKind, ParamKind, ParamSpec, Tool, ToolArgs, ToolMeta};

use super::{
    csv_to_json, csv_to_tsv, csv_to_xml, csv_to_yaml, json_to_csv, json_to_toml, json_to_tsv,
    json_to_xml, json_to_yaml, md_to_html, md_to_txt, numbase_convert, toml_to_json, toml_to_xml,
    toml_to_yaml, tsv_to_csv, tsv_to_json, xml_to_csv, xml_to_json, xml_to_toml, xml_to_yaml,
    yaml_to_csv, yaml_to_json, yaml_to_toml, yaml_to_xml,
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

pub struct CsvToTsv;
impl Tool for CsvToTsv {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "csv_to_tsv",
            name: "CSV → TSV",
            desc: "CSV 转 TSV(分隔符替换)",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        csv_to_tsv(input)
    }
}

pub struct TsvToCsv;
impl Tool for TsvToCsv {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "tsv_to_csv",
            name: "TSV → CSV",
            desc: "TSV 转 CSV",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        tsv_to_csv(input)
    }
}

pub struct CsvToYaml;
impl Tool for CsvToYaml {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "csv_to_yaml",
            name: "CSV → YAML",
            desc: "CSV 转 YAML(经 JSON 中转)",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("yaml"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        csv_to_yaml(input)
    }
}

pub struct YamlToCsv;
impl Tool for YamlToCsv {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "yaml_to_csv",
            name: "YAML → CSV",
            desc: "YAML 转 CSV(经 JSON 中转)",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        yaml_to_csv(input)
    }
}

pub struct CsvToXml;
impl Tool for CsvToXml {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "csv_to_xml",
            name: "CSV → XML",
            desc: "CSV 转 XML(经 JSON 中转)",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("xml"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        csv_to_xml(input)
    }
}

pub struct XmlToCsv;
impl Tool for XmlToCsv {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "xml_to_csv",
            name: "XML → CSV",
            desc: "XML 转 CSV(经 JSON 中转)",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        xml_to_csv(input)
    }
}

pub struct TsvToJson;
impl Tool for TsvToJson {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "tsv_to_json",
            name: "TSV → JSON",
            desc: "TSV 转 JSON 数组",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("json"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        tsv_to_json(input)
    }
}

pub struct JsonToTsv;
impl Tool for JsonToTsv {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "json_to_tsv",
            name: "JSON → TSV",
            desc: "JSON 数组转 TSV",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        json_to_tsv(input)
    }
}

pub struct JsonToXml;
impl Tool for JsonToXml {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "json_to_xml",
            name: "JSON → XML",
            desc: "JSON 转 XML",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("xml"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        json_to_xml(input)
    }
}

pub struct XmlToJson;
impl Tool for XmlToJson {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "xml_to_json",
            name: "XML → JSON",
            desc: "XML 转 JSON",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("json"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        xml_to_json(input)
    }
}

pub struct YamlToToml;
impl Tool for YamlToToml {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "yaml_to_toml",
            name: "YAML → TOML",
            desc: "YAML 转 TOML(经 JSON 中转)",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("toml"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        yaml_to_toml(input)
    }
}

pub struct TomlToYaml;
impl Tool for TomlToYaml {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "toml_to_yaml",
            name: "TOML → YAML",
            desc: "TOML 转 YAML(经 JSON 中转)",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("yaml"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        toml_to_yaml(input)
    }
}

pub struct YamlToXml;
impl Tool for YamlToXml {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "yaml_to_xml",
            name: "YAML → XML",
            desc: "YAML 转 XML(经 JSON 中转)",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("xml"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        yaml_to_xml(input)
    }
}

pub struct XmlToYaml;
impl Tool for XmlToYaml {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "xml_to_yaml",
            name: "XML → YAML",
            desc: "XML 转 YAML(经 JSON 中转)",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("yaml"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        xml_to_yaml(input)
    }
}

pub struct TomlToXml;
impl Tool for TomlToXml {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "toml_to_xml",
            name: "TOML → XML",
            desc: "TOML 转 XML(经 JSON 中转)",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("xml"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        toml_to_xml(input)
    }
}

pub struct XmlToToml;
impl Tool for XmlToToml {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "xml_to_toml",
            name: "XML → TOML",
            desc: "XML 转 TOML(经 JSON 中转)",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Highlight("toml"),
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        xml_to_toml(input)
    }
}

pub struct MdToTxt;
impl Tool for MdToTxt {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "md_to_txt",
            name: "Markdown → 纯文本",
            desc: "Markdown 去标记提纯文本",
            group: "convert",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        md_to_txt(input)
    }
}
