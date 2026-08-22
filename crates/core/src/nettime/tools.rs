//! 工具注册:网络/时间工具的元数据 + 字符串参数适配,供 registry 聚合

use crate::registry::{OutputKind, ParamKind, ParamSpec, Tool, ToolArgs, ToolMeta};
use crate::ToolError;

use super::{cron_next, dns_lookup, ipcalc, timestamp_from_human, timestamp_to_human};

pub struct Ipcalc;
impl Tool for Ipcalc {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "ipcalc",
            name: "IP 子网计算",
            desc: "CIDR 解析",
            group: "nettime",
            params: &[],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, _args: &ToolArgs) -> crate::ToolResult<String> {
        ipcalc(input)
    }
}

pub struct TimestampToHuman;
impl Tool for TimestampToHuman {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "timestamp_to_human",
            name: "时间戳→可读",
            desc: "Unix 秒转可读时间",
            group: "nettime",
            params: &[ParamSpec {
                key: "tz",
                kind: ParamKind::Text,
                label: "时区",
                default: Some("UTC"),
                options: &[],
                placeholder: Some("Asia/Shanghai"),
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let ts: i64 = input
            .trim()
            .parse()
            .map_err(|_| ToolError::Parse("时间戳需为整数".into()))?;
        let tz = args.get("tz").unwrap_or("UTC");
        timestamp_to_human(ts, tz)
    }
}

pub struct TimestampFromHuman;
impl Tool for TimestampFromHuman {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "timestamp_from_human",
            name: "可读→时间戳",
            desc: "可读时间转 Unix 秒",
            group: "nettime",
            params: &[ParamSpec {
                key: "tz",
                kind: ParamKind::Text,
                label: "时区",
                default: Some("UTC"),
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
        let tz = args.get("tz").unwrap_or("UTC");
        timestamp_from_human(input, tz)
    }
}

pub struct CronNext;
impl Tool for CronNext {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "cron_next",
            name: "cron 下次触发",
            desc: "接下来 N 次",
            group: "nettime",
            params: &[ParamSpec {
                key: "count",
                kind: ParamKind::Number,
                label: "次数",
                default: Some("3"),
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
        let n: usize = args
            .get("count")
            .unwrap_or("3")
            .parse()
            .map_err(|_| ToolError::Parse("参数 count 需为正整数".into()))?;
        cron_next(input, n)
    }
}

pub struct DnsLookup;
impl Tool for DnsLookup {
    fn meta(&self) -> &'static ToolMeta {
        static META: ToolMeta = ToolMeta {
            id: "dns_lookup",
            name: "DNS 查询",
            desc: "A/AAAA/MX/TXT",
            group: "nettime",
            params: &[ParamSpec {
                key: "rtype",
                kind: ParamKind::Select,
                label: "类型",
                default: Some("A"),
                options: &["A", "AAAA", "MX", "TXT"],
                placeholder: None,
                multiple: false,
            }],
            needs_main_input: true,
            output_kind: OutputKind::Text,
        };
        &META
    }
    fn run(&self, input: &str, args: &ToolArgs) -> crate::ToolResult<String> {
        let rtype = args.get("rtype").unwrap_or("A");
        dns_lookup(input, rtype)
    }
}
