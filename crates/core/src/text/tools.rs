//! 工具注册:文本工具的元数据 + 字符串参数适配,供 registry 聚合

use crate::registry::{OutputKind, ParamKind, ParamSpec, Tool, ToolArgs, ToolMeta};

use super::{
    case_convert, dedup_lines, diff_text, regex_match, regex_replace, reverse_text, sort_lines,
    text_align, text_escape, text_number_lines, text_replace, text_space_to_tab, text_stats,
    text_tab_to_space, text_trim_blank, Align, CaseMode, EscapeKind,
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

pub struct TextStats;
impl Tool for TextStats {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "text_stats",
            name: "文本统计",
            desc: "字符/字/行/字节",
            group: "text",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        text_stats(input)
    }
}

pub struct TextTrimBlank;
impl Tool for TextTrimBlank {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "text_trim_blank",
            name: "去空行修剪",
            desc: "删空行 + 行首尾 trim",
            group: "text",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        text_trim_blank(input)
    }
}

pub struct TabToSpace;
impl Tool for TabToSpace {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "tab_to_space",
            name: "Tab 转空格",
            desc: "制表符展开",
            group: "text",
            params: &[ParamSpec {
                key: "spaces",
                kind: ParamKind::Number,
                label: "空格数",
                default: Some("4"),
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
        let n = args.get_u32("spaces")? as usize;
        text_tab_to_space(input, n)
    }
}

pub struct SpaceToTab;
impl Tool for SpaceToTab {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "space_to_tab",
            name: "空格转 Tab",
            desc: "空格折叠为制表符",
            group: "text",
            params: &[ParamSpec {
                key: "spaces",
                kind: ParamKind::Number,
                label: "空格数",
                default: Some("4"),
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
        let n = args.get_u32("spaces")? as usize;
        text_space_to_tab(input, n)
    }
}

pub struct TextAlign;
impl Tool for TextAlign {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "text_align",
            name: "文本对齐",
            desc: "左/右/居中填充",
            group: "text",
            params: &[
                ParamSpec {
                    key: "direction",
                    kind: ParamKind::Select,
                    label: "方向",
                    default: Some("left"),
                    options: &["left", "right", "center"],
                    placeholder: None,
                    multiple: false,
                },
                ParamSpec {
                    key: "width",
                    kind: ParamKind::Number,
                    label: "宽度",
                    default: Some("80"),
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
        let direction = args.get_enum::<Align>("direction")?;
        let width = args.get_u32("width")? as usize;
        text_align(input, direction, width)
    }
}

pub struct TextReplace;
impl Tool for TextReplace {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "text_replace",
            name: "查找替换",
            desc: "字面量/正则",
            group: "text",
            params: &[
                ParamSpec {
                    key: "find",
                    kind: ParamKind::Text,
                    label: "查找",
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
                ParamSpec {
                    key: "useRegex",
                    kind: ParamKind::Select,
                    label: "正则模式",
                    default: Some("false"),
                    options: &["true", "false"],
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
        let find = args.get_str("find")?;
        let replacement = args.get_str("replacement")?;
        let use_regex = args.get_bool("useRegex");
        text_replace(input, find, replacement, use_regex)
    }
}

pub struct TextEscape;
impl Tool for TextEscape {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "text_escape",
            name: "转义",
            desc: "Shell/C/Regex",
            group: "text",
            params: &[ParamSpec {
                key: "kind",
                kind: ParamKind::Select,
                label: "类型",
                default: Some("shell"),
                options: &["shell", "c", "regex"],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let kind = args.get_enum::<EscapeKind>("kind")?;
        text_escape(input, kind)
    }
}

pub struct NumberLines;
impl Tool for NumberLines {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "number_lines",
            name: "行号添加",
            desc: "每行加行号",
            group: "text",
            params: &[ParamSpec {
                key: "start",
                kind: ParamKind::Number,
                label: "起始行号",
                default: Some("1"),
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
        let start = args.get_u32("start")?;
        text_number_lines(input, start)
    }
}
