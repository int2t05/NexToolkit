//! 工具注册:文本工具的元数据 + 字符串参数适配,供 registry 聚合

use crate::registry::{OutputKind, ParamKind, ParamSpec, Tool, ToolArgs, ToolMeta};

use super::{
    case_convert, dedup_lines, diff_text, regex_match, regex_replace, reverse_text, sort_lines,
    CaseMode,
};

pub struct CaseConvert;
impl Tool for CaseConvert {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "case_convert",
            name: "大小写转换",
            desc: "snake/camel/kebab 等",
            group: "text",
            params: &[ParamSpec {
                key: "mode",
                kind: ParamKind::Select,
                label: "模式",
                default: Some("snake"),
                options: &["upper", "lower", "title", "snake", "camel", "kebab"],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let mode = args.get_enum::<CaseMode>("mode")?;
        case_convert(input, mode)
    }
}

pub struct SortLines;
impl Tool for SortLines {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "sort_lines",
            name: "行排序",
            desc: "升序排序",
            group: "text",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        sort_lines(input)
    }
}

pub struct DedupLines;
impl Tool for DedupLines {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "dedup_lines",
            name: "行去重",
            desc: "保序去重",
            group: "text",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        dedup_lines(input)
    }
}

pub struct ReverseText;
impl Tool for ReverseText {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "reverse_text",
            name: "文本反转",
            desc: "按 Unicode 字符",
            group: "text",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        reverse_text(input)
    }
}

pub struct RegexMatch;
impl Tool for RegexMatch {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "regex_match",
            name: "正则匹配",
            desc: "每匹配一行",
            group: "text",
            params: &[ParamSpec {
                key: "pattern",
                kind: ParamKind::Text,
                label: "正则",
                default: None,
                options: &[],
                placeholder: Some("\\d+"),
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let pattern = args.get_str("pattern")?;
        regex_match(pattern, input)
    }
}

pub struct RegexReplace;
impl Tool for RegexReplace {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "regex_replace",
            name: "正则替换",
            desc: "支持 $0/$1",
            group: "text",
            params: &[
                ParamSpec {
                    key: "pattern",
                    kind: ParamKind::Text,
                    label: "正则",
                    default: None,
                    options: &[],
                    placeholder: None,
                    multiple: false,
                },
                ParamSpec {
                    key: "replacement",
                    kind: ParamKind::Text,
                    label: "替换",
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
        let pattern = args.get_str("pattern")?;
        let replacement = args.get_str("replacement")?;
        regex_replace(pattern, replacement, input)
    }
}

pub struct DiffText;
impl Tool for DiffText {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "diff_text",
            name: "文本 Diff",
            desc: "unified diff",
            group: "text",
            params: &[ParamSpec {
                key: "other",
                kind: ParamKind::Textarea,
                label: "对比文本",
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
        let other = args.get_str("other")?;
        diff_text(input, other)
    }
}
