//! 文本子命令:case/sort-dedup/reverse/regex/diff

use clap::{Args, Subcommand};

use crate::io::read_input;
use nextool_core::CaseMode;

#[derive(Args)]
pub struct TextArgs {
    #[command(subcommand)]
    cmd: TextCmd,
}

#[derive(Subcommand)]
enum TextCmd {
    /// 大小写转换:upper/lower/title/snake/camel/kebab
    Case {
        mode: CaseModeArg,
        input: Option<String>,
    },
    /// 行排序(升序)
    SortLines { input: Option<String> },
    /// 行去重(保序)
    DedupLines { input: Option<String> },
    /// 文本反转(按 Unicode 字符)
    Reverse { input: Option<String> },
    /// 正则匹配:每匹配一行输出
    RegexMatch {
        pattern: String,
        input: Option<String>,
    },
    /// 正则替换:支持 $0/$1 捕获组
    RegexReplace {
        pattern: String,
        replacement: String,
        input: Option<String>,
    },
    /// 文本 Diff:比较两个输入(unified diff)
    Diff {
        a: Option<String>,
        b: Option<String>,
    },
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum CaseModeArg {
    Upper,
    Lower,
    Title,
    Snake,
    Camel,
    Kebab,
}

impl From<CaseModeArg> for CaseMode {
    fn from(m: CaseModeArg) -> Self {
        match m {
            CaseModeArg::Upper => CaseMode::Upper,
            CaseModeArg::Lower => CaseMode::Lower,
            CaseModeArg::Title => CaseMode::Title,
            CaseModeArg::Snake => CaseMode::Snake,
            CaseModeArg::Camel => CaseMode::Camel,
            CaseModeArg::Kebab => CaseMode::Kebab,
        }
    }
}

pub fn run(args: TextArgs) -> Result<(), String> {
    match args.cmd {
        TextCmd::Case { mode, input } => {
            let input = read_input(input)?;
            println!(
                "{}",
                nextool_core::case_convert(&input, mode.into()).map_err(|e| e.to_string())?
            );
        }
        TextCmd::SortLines { input } => {
            println!(
                "{}",
                nextool_core::sort_lines(&read_input(input)?).map_err(|e| e.to_string())?
            );
        }
        TextCmd::DedupLines { input } => {
            println!(
                "{}",
                nextool_core::dedup_lines(&read_input(input)?).map_err(|e| e.to_string())?
            );
        }
        TextCmd::Reverse { input } => {
            println!(
                "{}",
                nextool_core::reverse_text(&read_input(input)?).map_err(|e| e.to_string())?
            );
        }
        TextCmd::RegexMatch { pattern, input } => {
            let input = read_input(input)?;
            println!(
                "{}",
                nextool_core::regex_match(&pattern, &input).map_err(|e| e.to_string())?
            );
        }
        TextCmd::RegexReplace {
            pattern,
            replacement,
            input,
        } => {
            let input = read_input(input)?;
            println!(
                "{}",
                nextool_core::regex_replace(&pattern, &replacement, &input)
                    .map_err(|e| e.to_string())?
            );
        }
        TextCmd::Diff { a, b } => {
            let a = read_input(a)?;
            let b = read_input(b)?;
            let out = nextool_core::diff_text(&a, &b).map_err(|e| e.to_string())?;
            if !out.is_empty() {
                println!("{out}");
            }
        }
    }
    Ok(())
}
