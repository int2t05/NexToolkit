//! 转换子命令:json-yaml/json-toml/json-csv/md-html/numbase

use clap::{Args, Subcommand};

use crate::io::read_input;

#[derive(Args)]
pub struct ConvertArgs {
    #[command(subcommand)]
    cmd: ConvertCmd,
}

#[derive(Subcommand)]
enum ConvertCmd {
    /// JSON 与 YAML 互转
    JsonYaml { mode: ConvertMode, input: Option<String> },
    /// JSON 与 TOML 互转
    JsonToml { mode: ConvertMode, input: Option<String> },
    /// JSON 与 CSV 互转
    JsonCsv { mode: ConvertMode, input: Option<String> },
    /// Markdown 转 HTML
    MdHtml { input: Option<String> },
    /// 进制转换:--from 与 --to 指定进制(2..=36)
    Numbase {
        from: u32,
        to: u32,
        input: Option<String>,
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
            let input = read_input(input)?;
            let out = match mode {
                ConvertMode::To => nextool_core::json_to_yaml(&input),
                ConvertMode::From => nextool_core::yaml_to_json(&input),
            };
            println!("{}", out.map_err(|e| e.to_string())?);
        }
        ConvertCmd::JsonToml { mode, input } => {
            let input = read_input(input)?;
            let out = match mode {
                ConvertMode::To => nextool_core::json_to_toml(&input),
                ConvertMode::From => nextool_core::toml_to_json(&input),
            };
            println!("{}", out.map_err(|e| e.to_string())?);
        }
        ConvertCmd::JsonCsv { mode, input } => {
            let input = read_input(input)?;
            let out = match mode {
                ConvertMode::To => nextool_core::json_to_csv(&input),
                ConvertMode::From => nextool_core::csv_to_json(&input),
            };
            println!("{}", out.map_err(|e| e.to_string())?);
        }
        ConvertCmd::MdHtml { input } => {
            let input = read_input(input)?;
            println!("{}", nextool_core::md_to_html(&input).map_err(|e| e.to_string())?);
        }
        ConvertCmd::Numbase { from, to, input } => {
            let input = read_input(input)?;
            println!(
                "{}",
                nextool_core::numbase_convert(&input, from, to).map_err(|e| e.to_string())?
            );
        }
    }
    Ok(())
}
