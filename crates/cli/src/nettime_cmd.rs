//! 网络/时间子命令:ipcalc/timestamp/cron/dns/http

use clap::{Args, Subcommand};

use crate::io::read_input;

#[derive(Args)]
pub struct NetTimeArgs {
    #[command(subcommand)]
    cmd: NetTimeCmd,
}

#[derive(Subcommand)]
enum NetTimeCmd {
    /// IP 子网计算:输入 CIDR(如 192.168.1.5/24)
    Ipcalc { input: Option<String> },
    /// 时间戳转可读时间:--tz 时区(如 Asia/Shanghai)
    TsToHuman {
        ts: i64,
        #[arg(long, default_value = "UTC")]
        tz: String,
    },
    /// 可读时间转时间戳:格式 YYYY-MM-DD HH:MM:SS,--tz 时区
    TsFromHuman {
        input: Option<String>,
        #[arg(long, default_value = "UTC")]
        tz: String,
    },
    /// cron 下次触发:--count 次数
    CronNext {
        expr: String,
        #[arg(long, default_value_t = 3)]
        count: usize,
    },
    /// DNS 查询:A/AAAA/MX/TXT
    Dns { rtype: String, domain: String },
    /// HTTP 探测:URL 状态码与响应头
    Http { url: String },
}

pub fn run(args: NetTimeArgs) -> Result<(), String> {
    match args.cmd {
        NetTimeCmd::Ipcalc { input } => {
            println!(
                "{}",
                nextool_core::ipcalc(&read_input(input)?).map_err(|e| e.to_string())?
            );
        }
        NetTimeCmd::TsToHuman { ts, tz } => {
            println!(
                "{}",
                nextool_core::timestamp_to_human(ts, &tz).map_err(|e| e.to_string())?
            );
        }
        NetTimeCmd::TsFromHuman { input, tz } => {
            let input = read_input(input)?;
            println!(
                "{}",
                nextool_core::timestamp_from_human(&input, &tz).map_err(|e| e.to_string())?
            );
        }
        NetTimeCmd::CronNext { expr, count } => {
            print!(
                "{}",
                nextool_core::cron_next(&expr, count).map_err(|e| e.to_string())?
            );
        }
        NetTimeCmd::Dns { rtype, domain } => {
            print!(
                "{}",
                nextool_core::dns_lookup(&domain, &rtype).map_err(|e| e.to_string())?
            );
        }
        NetTimeCmd::Http { url } => {
            let probe = nextool_core::http_probe(&url).map_err(|e| e.to_string())?;
            println!("{}", probe.to_display());
        }
    }
    Ok(())
}
