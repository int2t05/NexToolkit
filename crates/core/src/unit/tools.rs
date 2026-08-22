//! 工具注册:单位换算工具的元数据 + 字符串参数适配,供 registry 聚合

use crate::registry::{OutputKind, ParamKind, ParamSpec, Tool, ToolArgs, ToolMeta};
use crate::ToolError;

use super::unit_convert;

pub struct UnitConvert;
impl Tool for UnitConvert {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "unit_convert",
            name: "单位换算",
            desc: "长度/质量/温度等 10 类",
            group: "convert",
            params: &[
                ParamSpec {
                    key: "value",
                    kind: ParamKind::Number,
                    label: "值",
                    default: Some("1"),
                    options: &[],
                    placeholder: None,
                    multiple: false,
                },
                ParamSpec {
                    key: "from",
                    kind: ParamKind::Text,
                    label: "从",
                    default: None,
                    options: &[],
                    placeholder: Some("km"),
                    multiple: false,
                },
                ParamSpec {
                    key: "to",
                    kind: ParamKind::Text,
                    label: "到",
                    default: None,
                    options: &[],
                    placeholder: Some("m"),
                    multiple: false,
                },
            ],
            needs_main_input: false,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, _input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let value: f64 = args
            .get_str("value")?
            .parse()
            .map_err(|e| ToolError::Parse(format!("参数 value 需为数值: {e}")))?;
        let from = args.get_str("from")?;
        let to = args.get_str("to")?;
        unit_convert(value, from, to).map(|v| v.to_string())
    }
}
