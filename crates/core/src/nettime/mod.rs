//! 网络/时间模块:IP 计算/时间戳/cron/DNS

use std::net::Ipv4Addr;
use std::str::FromStr;

use chrono::{NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use cron::Schedule;
use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    proto::rr::{RData, RecordType},
    Resolver,
};
use ipnet::IpNet;

mod tools;
pub use tools::*;

use crate::{ToolError, ToolResult};

/// IP 子网计算:解析 CIDR(如 "192.168.1.5/24"),输出地址/网络/广播/掩码/主机范围/主机位/主机数
pub fn ipcalc(input: &str) -> ToolResult<String> {
    let net: IpNet = IpNet::from_str(input.trim()).map_err(|e| ToolError::Parse(e.to_string()))?;

    let host_bits = net.max_prefix_len() - net.prefix_len();
    // 可用主机数按位运算计算,避免遍历大子网
    let total = 1u128.checked_shl(host_bits as u32).unwrap_or(u128::MAX);
    // IPv4 前缀 <31 时排除网络地址与广播地址;IPv6 及 /31、/32 全部可用
    let usable = match &net {
        IpNet::V4(v4) if v4.prefix_len() < 31 => total.saturating_sub(2),
        _ => total,
    };

    // 可用主机范围:IPv4 前缀 <31 时首=网络+1、末=广播-1,否则覆盖网络到广播
    let range = match &net {
        IpNet::V4(v4) if v4.prefix_len() < 31 => {
            let n = u32::from(v4.network());
            let b = u32::from(v4.broadcast());
            format!("{} - {}", Ipv4Addr::from(n + 1), Ipv4Addr::from(b - 1))
        }
        _ => format!("{} - {}", net.network(), net.broadcast()),
    };

    Ok(format!(
        "地址: {}\n网络地址: {}\n广播地址: {}\n子网掩码: {}\n可用主机范围: {}\n主机位数: {}\n可用主机数: {}\n",
        net.addr(),
        net.network(),
        net.broadcast(),
        net.netmask(),
        range,
        host_bits,
        usable,
    ))
}

/// Unix 秒(可为负,1970 前)→ 指定时区的 RFC3339 可读时间
pub fn timestamp_to_human(ts: i64, tz: &str) -> ToolResult<String> {
    let tz: Tz = tz
        .parse()
        .map_err(|e| ToolError::Parse(format!("无效时区: {e}")))?;
    // chrono: timestamp_opt 返回 LocalResult,用 .single() 取唯一值
    let utc = Utc
        .timestamp_opt(ts, 0)
        .single()
        .ok_or_else(|| ToolError::Parse(format!("时间戳超出范围: {ts}")))?;
    Ok(utc.with_timezone(&tz).to_rfc3339())
}

/// 可读时间(如 "2023-11-14 22:13:20")按指定时区解析为本地 naive→aware→Unix 秒(字符串)
pub fn timestamp_from_human(input: &str, tz: &str) -> ToolResult<String> {
    let tz: Tz = tz
        .parse()
        .map_err(|e| ToolError::Parse(format!("无效时区: {e}")))?;
    let naive = NaiveDateTime::parse_from_str(input.trim(), "%Y-%m-%d %H:%M:%S")
        .map_err(|e| ToolError::Parse(format!("时间解析失败: {e}")))?;
    // 附加时区:夏令时切换时本地时间可能歧义或不存在
    let dt = naive
        .and_local_timezone(tz)
        .single()
        .ok_or_else(|| ToolError::Parse("本地时间不存在或歧义(夏令时切换)".into()))?;
    Ok(dt.timestamp().to_string())
}

/// cron 表达式(6/7 字段:sec min hour day month weekday[year])→ 接下来 n 次触发时间(RFC3339,每行一个)
pub fn cron_next(expr: &str, n: usize) -> ToolResult<String> {
    if n == 0 {
        return Err(ToolError::InvalidInput("n 必须大于 0".into()));
    }
    let schedule = Schedule::from_str(expr).map_err(|e| ToolError::Parse(e.to_string()))?;
    let times: Vec<String> = schedule
        .upcoming(Utc)
        .take(n)
        .map(|dt| dt.to_rfc3339())
        .collect();
    Ok(times.join("\n"))
}

/// DNS 查询:rtype ∈ A/AAAA/MX/TXT;A/AAAA 每行一个 IP,MX 输出 priority+host,TXT 输出文本
pub fn dns_lookup(domain: &str, rtype: &str) -> ToolResult<String> {
    let record_type = match rtype {
        "A" => RecordType::A,
        "AAAA" => RecordType::AAAA,
        "MX" => RecordType::MX,
        "TXT" => RecordType::TXT,
        _ => return Err(ToolError::InvalidInput(format!("无效记录类型: {rtype}"))),
    };

    // hickory-resolver 同步 Resolver:内部自建 current-thread tokio runtime,block_on 执行查询
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())?;
    let lookup = resolver
        .lookup(domain, record_type)
        .map_err(|e| ToolError::Other(e.to_string()))?;

    let lines: Vec<String> = match record_type {
        RecordType::A | RecordType::AAAA => lookup
            .iter()
            .filter_map(|r| r.ip_addr().map(|ip| ip.to_string()))
            .collect(),
        RecordType::MX => lookup
            .iter()
            .filter_map(|r| match r {
                RData::MX(mx) => Some(format!("{} {}", mx.preference(), mx.exchange())),
                _ => None,
            })
            .collect(),
        RecordType::TXT => lookup
            .iter()
            .filter_map(|r| match r {
                RData::TXT(txt) => Some(format!("{txt}")),
                _ => None,
            })
            .collect(),
        _ => unreachable!("record_type 已校验为 A/AAAA/MX/TXT"),
    };

    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- ipcalc ----

    #[test]
    fn ipcalc_v4_24() {
        let out = ipcalc("192.168.1.5/24").unwrap();
        assert!(out.contains("192.168.1.0"), "应含网络地址");
        assert!(out.contains("192.168.1.255"), "应含广播地址");
        assert!(out.contains("255.255.255.0"), "应含子网掩码");
        assert!(out.contains("192.168.1.1 - 192.168.1.254"), "应含主机范围");
        assert!(out.contains("主机位数: 8"));
        assert!(out.contains("可用主机数: 254"));
    }

    #[test]
    fn ipcalc_v4_8() {
        let out = ipcalc("10.0.0.0/8").unwrap();
        assert!(out.contains("10.0.0.0"));
        assert!(out.contains("10.255.255.255"));
        assert!(out.contains("可用主机数: 16777214")); // 2^24 - 2
    }

    #[test]
    fn ipcalc_v6() {
        let out = ipcalc("2001:db8::/32").unwrap();
        assert!(out.contains("2001:db8::"));
        assert!(out.contains("ffff:ffff::"));
        assert!(out.contains("主机位数: 96"));
    }

    #[test]
    fn ipcalc_invalid() {
        assert!(ipcalc("not-a-cidr").is_err());
        assert!(ipcalc("192.168.1.5/33").is_err()); // 前缀越界
    }

    // ---- timestamp_to_human ----

    #[test]
    fn ts_to_human_shanghai() {
        let out = timestamp_to_human(1700000000, "Asia/Shanghai").unwrap();
        assert!(out.contains("2023"), "年份应为 2023");
        assert!(out.contains("+08"), "时区应为 +08");
    }

    #[test]
    fn ts_to_human_negative() {
        // 1970 前的负时间戳
        let out = timestamp_to_human(-1, "UTC").unwrap();
        assert!(out.contains("1969"), "Unix -1 应在 1969");
        assert!(out.contains("+00:00"));
    }

    #[test]
    fn ts_to_human_invalid_tz() {
        assert!(timestamp_to_human(0, "Invalid/Zone").is_err());
    }

    // ---- timestamp_from_human ----

    #[test]
    fn ts_from_human_roundtrip() {
        let tz = "Asia/Shanghai";
        let ts: i64 = timestamp_from_human("2023-11-14 22:13:20", tz)
            .unwrap()
            .parse()
            .unwrap();
        let back = timestamp_to_human(ts, tz).unwrap();
        assert!(back.contains("2023-11-14"), "roundtrip 应还原日期");
        assert!(back.contains("22:13:20"), "roundtrip 应还原时间");
    }

    #[test]
    fn ts_from_human_epoch() {
        // UTC 1970-01-01 00:00:00 == 0
        let ts = timestamp_from_human("1970-01-01 00:00:00", "UTC").unwrap();
        assert_eq!(ts, "0");
    }

    #[test]
    fn ts_from_human_invalid() {
        assert!(timestamp_from_human("not a date", "UTC").is_err());
        assert!(timestamp_from_human("2023-11-14 22:13:20", "Invalid/Zone").is_err());
    }

    // ---- cron_next ----

    #[test]
    fn cron_next_three() {
        let out = cron_next("0 * * * * *", 3).unwrap();
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 3, "应输出 3 行");
        for line in &lines {
            assert!(line.contains('T'), "每行应为 RFC3339 时间: {line}");
        }
    }

    #[test]
    fn cron_next_invalid_expr() {
        assert!(cron_next("not a cron expr", 1).is_err());
    }

    #[test]
    fn cron_next_zero_n() {
        assert!(cron_next("0 * * * * *", 0).is_err());
    }

    // ---- dns_lookup ----

    #[test]
    fn dns_lookup_invalid_rtype() {
        // 纯逻辑:rtype 非法在发起网络请求前即返回错误
        assert!(dns_lookup("example.com", "CNAME").is_err());
        assert!(dns_lookup("example.com", "invalid").is_err());
    }

    #[test]
    #[ignore] // 需要真实网络: cargo test --package nextool-core nettime::tests::dns_lookup_a -- --ignored
    fn dns_lookup_a() {
        let out = dns_lookup("example.com", "A").unwrap();
        assert!(!out.is_empty(), "应返回非空 IP 列表");
        for line in out.lines() {
            line.parse::<std::net::IpAddr>()
                .unwrap_or_else(|_| panic!("每行应为合法 IP: {line}"));
        }
    }

    #[test]
    #[ignore] // 需要真实网络: cargo test --package nextool-core nettime::tests::dns_lookup_txt -- --ignored
    fn dns_lookup_txt() {
        let out = dns_lookup("example.com", "TXT").unwrap();
        assert!(!out.is_empty(), "example.com 应有 TXT 记录");
    }
}
