//! 文件转换子命令:归档解压/压缩/转换/列表
//!
//! 薄封装 [`nextool_fileconv`] 的纯内存 API 与 [`nextool_fileconv::fs_util`] 的落盘边界。
//! 产物落源文件所在目录(碰撞处理见 fs_util)。

use clap::{Args, Subcommand, ValueEnum};

use crate::io::read_bytes_input;

/// 归档格式(clap 值枚举,与 [`nextool_fileconv::ArchiveFormat`] 对应)
#[derive(Clone, Debug, ValueEnum)]
pub enum ArchiveFormatArg {
    Zip,
    Tar,
    Targz,
    Gz,
}

impl ArchiveFormatArg {
    fn to_format(&self) -> nextool_fileconv::ArchiveFormat {
        match self {
            ArchiveFormatArg::Zip => nextool_fileconv::ArchiveFormat::Zip,
            ArchiveFormatArg::Tar => nextool_fileconv::ArchiveFormat::Tar,
            ArchiveFormatArg::Targz => nextool_fileconv::ArchiveFormat::TarGz,
            ArchiveFormatArg::Gz => nextool_fileconv::ArchiveFormat::Gz,
        }
    }
}

#[derive(Args)]
pub struct FileConvArgs {
    #[command(subcommand)]
    cmd: FileConvCmd,
}

#[derive(Subcommand)]
enum FileConvCmd {
    /// 归档操作:解压/压缩/转换/列表
    Archive {
        #[command(subcommand)]
        cmd: ArchiveCmd,
    },
}

#[derive(Subcommand)]
enum ArchiveCmd {
    /// 列出归档内文件
    List { input: String },
    /// 解压归档到目录(默认源文件旁 `_extracted`)
    Extract {
        input: String,
        #[arg(long)]
        output_dir: Option<String>,
    },
    /// 压缩文件为归档
    Compress {
        format: ArchiveFormatArg,
        #[arg(required = true)]
        files: Vec<String>,
        #[arg(long)]
        output: Option<String>,
    },
    /// 归档格式互转
    Convert {
        input: String,
        target_format: ArchiveFormatArg,
        #[arg(long)]
        output: Option<String>,
    },
}

pub fn run(args: FileConvArgs) -> Result<(), String> {
    match args.cmd {
        FileConvCmd::Archive { cmd } => match cmd {
            ArchiveCmd::List { input } => {
                let data = read_bytes_input(&input)?;
                let list = nextool_fileconv::archive_list(&data).map_err(|e| e.to_string())?;
                println!("{list}");
                Ok(())
            }
            ArchiveCmd::Extract { input, output_dir } => {
                let (out_dir, written) =
                    nextool_fileconv::extract_to_dir(&input, output_dir.as_deref())
                        .map_err(|e| e.to_string())?;
                println!("已解压 {} 个文件到 {out_dir}", written.len());
                Ok(())
            }
            ArchiveCmd::Compress {
                format,
                files,
                output,
            } => {
                let out =
                    nextool_fileconv::compress_files(&files, format.to_format(), output.as_deref())
                        .map_err(|e| e.to_string())?;
                println!("已创建归档 {out}");
                Ok(())
            }
            ArchiveCmd::Convert {
                input,
                target_format,
                output,
            } => {
                let out = nextool_fileconv::convert_file(
                    &input,
                    target_format.to_format(),
                    output.as_deref(),
                )
                .map_err(|e| e.to_string())?;
                println!("已转换 {out}");
                Ok(())
            }
        },
    }
}
