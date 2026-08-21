//! 格式化子命令:json/sql/xml 美化与压缩、css 压缩

use clap::{Args, Subcommand};

use crate::io::read_input;

#[derive(Args)]
pub struct FormatArgs {
    #[command(subcommand)]
    cmd: FormatCmd,
}

#[derive(Subcommand)]
enum FormatCmd {
    /// JSON 美化
    JsonFmt { input: Option<String> },
    /// JSON 压缩
    JsonMin { input: Option<String> },
    /// SQL 美化
    SqlFmt { input: Option<String> },
    /// XML 美化
    XmlFmt { input: Option<String> },
    /// XML 压缩
    XmlMin { input: Option<String> },
    /// CSS 压缩
    CssMin { input: Option<String> },
}

pub fn run(args: FormatArgs) -> Result<(), String> {
    match args.cmd {
        FormatCmd::JsonFmt { input } => {
            println!("{}", nextool_core::json_format(&read_input(input)?).map_err(|e| e.to_string())?);
        }
        FormatCmd::JsonMin { input } => {
            println!("{}", nextool_core::json_minify(&read_input(input)?).map_err(|e| e.to_string())?);
        }
        FormatCmd::SqlFmt { input } => {
            println!("{}", nextool_core::sql_format(&read_input(input)?).map_err(|e| e.to_string())?);
        }
        FormatCmd::XmlFmt { input } => {
            println!("{}", nextool_core::xml_format(&read_input(input)?).map_err(|e| e.to_string())?);
        }
        FormatCmd::XmlMin { input } => {
            println!("{}", nextool_core::xml_minify(&read_input(input)?).map_err(|e| e.to_string())?);
        }
        FormatCmd::CssMin { input } => {
            println!("{}", nextool_core::css_minify(&read_input(input)?).map_err(|e| e.to_string())?);
        }
    }
    Ok(())
}
