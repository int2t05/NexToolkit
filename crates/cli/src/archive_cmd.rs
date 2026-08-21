//! 文件转换子命令:归档解压/压缩/转换/列表 + 图像转换/缩放
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

/// 图像格式(clap 值枚举,与 [`nextool_fileconv::ImageFormat`] 对应)
#[derive(Clone, Debug, ValueEnum)]
pub enum ImageFormatArg {
    Png,
    Jpg,
    Gif,
    Bmp,
    Webp,
    Tiff,
    Ico,
}

impl ImageFormatArg {
    fn to_format(&self) -> nextool_fileconv::ImageFormat {
        match self {
            ImageFormatArg::Png => nextool_fileconv::ImageFormat::Png,
            ImageFormatArg::Jpg => nextool_fileconv::ImageFormat::Jpeg,
            ImageFormatArg::Gif => nextool_fileconv::ImageFormat::Gif,
            ImageFormatArg::Bmp => nextool_fileconv::ImageFormat::Bmp,
            ImageFormatArg::Webp => nextool_fileconv::ImageFormat::Webp,
            ImageFormatArg::Tiff => nextool_fileconv::ImageFormat::Tiff,
            ImageFormatArg::Ico => nextool_fileconv::ImageFormat::Ico,
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
    /// 图像操作:格式转换/缩放
    Image {
        #[command(subcommand)]
        cmd: ImageCmd,
    },
    /// PDF 操作:拆分/旋转/加密/解密
    Pdf {
        #[command(subcommand)]
        cmd: PdfCmd,
    },
}

#[derive(Subcommand)]
enum PdfCmd {
    /// 拆分 PDF:每页一个独立 PDF
    Split {
        input: String,
        #[arg(long)]
        output_dir: Option<String>,
    },
    /// 旋转 PDF 所有页 90 度(顺时针)
    Rotate {
        input: String,
        #[arg(long)]
        output: Option<String>,
    },
    /// 加密 PDF(--password)
    Encrypt {
        input: String,
        #[arg(long)]
        password: String,
        #[arg(long)]
        output: Option<String>,
    },
    /// 解密 PDF(--password)
    Decrypt {
        input: String,
        #[arg(long)]
        password: String,
        #[arg(long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum ImageCmd {
    /// 图像格式转换
    Convert {
        input: String,
        target: ImageFormatArg,
        #[arg(long)]
        output: Option<String>,
    },
    /// 图像缩放(--width/--height 一维为 0 时按另一维等比)
    Resize {
        input: String,
        #[arg(long)]
        width: u32,
        #[arg(long)]
        height: u32,
        #[arg(long)]
        output: Option<String>,
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
        FileConvCmd::Image { cmd } => match cmd {
            ImageCmd::Convert {
                input,
                target,
                output,
            } => {
                let out = nextool_fileconv::convert_image_file(
                    &input,
                    target.to_format(),
                    output.as_deref(),
                )
                .map_err(|e| e.to_string())?;
                println!("已转换 {out}");
                Ok(())
            }
            ImageCmd::Resize {
                input,
                width,
                height,
                output,
            } => {
                let out =
                    nextool_fileconv::resize_image_file(&input, width, height, output.as_deref())
                        .map_err(|e| e.to_string())?;
                println!("已缩放 {out}");
                Ok(())
            }
        },
        FileConvCmd::Pdf { cmd } => match cmd {
            PdfCmd::Split { input, output_dir } => {
                let written = nextool_fileconv::split_pdf(&input, output_dir.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已拆分为 {} 个 PDF", written.len());
                Ok(())
            }
            PdfCmd::Rotate { input, output } => {
                let out = nextool_fileconv::rotate_pdf(&input, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已旋转 {out}");
                Ok(())
            }
            PdfCmd::Encrypt {
                input,
                password,
                output,
            } => {
                let out = nextool_fileconv::encrypt_pdf(&input, &password, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已加密 {out}");
                Ok(())
            }
            PdfCmd::Decrypt {
                input,
                password,
                output,
            } => {
                let out = nextool_fileconv::decrypt_pdf(&input, &password, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已解密 {out}");
                Ok(())
            }
        },
    }
}
