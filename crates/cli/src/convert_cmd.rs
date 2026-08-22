//! 转换子命令:json-yaml/json-toml/json-csv/md-html/numbase

use clap::{Args, Subcommand};

use crate::io::print_text;

#[derive(Args)]
pub struct ConvertArgs {
    #[command(subcommand)]
    cmd: ConvertCmd,
}

#[derive(Subcommand)]
enum ConvertCmd {
    /// JSON 与 YAML 互转
    JsonYaml {
        mode: ConvertMode,
        input: Option<String>,
    },
    /// JSON 与 TOML 互转
    JsonToml {
        mode: ConvertMode,
        input: Option<String>,
    },
    /// JSON 与 CSV 互转
    JsonCsv {
        mode: ConvertMode,
        input: Option<String>,
    },
    /// Markdown 转 HTML
    MdHtml { input: Option<String> },
    /// 进制转换:--from 与 --to 指定进制(2..=36)
    Numbase {
        from: u32,
        to: u32,
        input: Option<String>,
    },
    /// 单位换算:unit <value> <from> <to>(长度/面积/体积/质量/温度/时间/速度/数据/能量/频率)
    Unit {
        value: f64,
        from: String,
        to: String,
    },
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum ConvertMode {
    /// 转为目标格式(如 json-yaml 的 yaml 方向)
    To,
    /// 转为源格式(如 json-yaml 的 json 方向)
    From,
}

pub fn run(args: ConvertArgs) -> Result<(), String> {
    match args.cmd {
        ConvertCmd::JsonYaml { mode, input } => {
            print_text(input, |s| match mode {
                ConvertMode::To => nextool_core::json_to_yaml(s),
                ConvertMode::From => nextool_core::yaml_to_json(s),
            })?;
        }
        ConvertCmd::JsonToml { mode, input } => {
            print_text(input, |s| match mode {
                ConvertMode::To => nextool_core::json_to_toml(s),
                ConvertMode::From => nextool_core::toml_to_json(s),
            })?;
        }
        ConvertCmd::JsonCsv { mode, input } => {
            print_text(input, |s| match mode {
                ConvertMode::To => nextool_core::json_to_csv(s),
                ConvertMode::From => nextool_core::csv_to_json(s),
            })?;
        }
        ConvertCmd::MdHtml { input } => print_text(input, nextool_core::md_to_html)?,
        ConvertCmd::Numbase { from, to, input } => {
            print_text(input, |s| nextool_core::numbase_convert(s, from, to))?;
        }
        ConvertCmd::Unit { value, from, to } => {
            let result =
                nextool_core::unit_convert(value, &from, &to).map_err(|e| e.to_string())?;
            println!("{result}");
        }
    }
    Ok(())
}
