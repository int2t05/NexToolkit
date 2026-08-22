//! 格式化子命令:json/sql/xml 美化与压缩、css 压缩

use clap::{Args, Subcommand};

use crate::io::print_text;

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
        FormatCmd::JsonFmt { input } => print_text(input, nextool_core::json_format)?,
        FormatCmd::JsonMin { input } => print_text(input, nextool_core::json_minify)?,
        FormatCmd::SqlFmt { input } => print_text(input, nextool_core::sql_format)?,
        FormatCmd::XmlFmt { input } => print_text(input, nextool_core::xml_format)?,
        FormatCmd::XmlMin { input } => print_text(input, nextool_core::xml_minify)?,
        FormatCmd::CssMin { input } => print_text(input, nextool_core::css_minify)?,
    }
    Ok(())
}
