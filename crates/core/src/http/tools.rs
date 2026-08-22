//! 工具注册:HTTP 探测工具的元数据 + 字符串参数适配,供 registry 聚合

use crate::registry::{OutputKind, ParamKind, ParamSpec, Tool, ToolArgs, ToolMeta};

use super::http_probe;

pub struct HttpProbe;
impl Tool for HttpProbe {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "http_probe",
            name: "HTTP 探测",
            desc: "URL 状态码与响应头",
            group: "nettime",
            params: &[ParamSpec {
                key: "url",
                kind: ParamKind::Text,
                label: "URL",
                default: None,
                options: &[],
                placeholder: Some("https://example.com"),
                multiple: false,
            }],
            needs_main_input: false,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, _input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let url = args.get_str("url")?;
        http_probe(url).map(|h| h.to_display())
    }
}
