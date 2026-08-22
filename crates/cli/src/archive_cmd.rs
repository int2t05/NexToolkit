//! 文件转换子命令:归档解压/压缩/转换/列表 + 图像转换/缩放
//!
//! 薄封装 [`nextool_core::fileconv`] 的纯内存 API 与 [`nextool_core::fileconv::fs_util`] 的落盘边界。
//! 产物落源文件所在目录(碰撞处理见 fs_util)。

use clap::{Args, Subcommand};

use crate::io::read_bytes_input;

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
    /// 引擎转换:音视频/Office/电子书/标记/PDF压缩/OCR(运行时探测系统已装引擎)
    Engine {
        #[command(subcommand)]
        cmd: EngineCmd,
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
        target: nextool_core::ImageFormat,
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
        format: nextool_core::ArchiveFormat,
        #[arg(required = true)]
        files: Vec<String>,
        #[arg(long)]
        output: Option<String>,
    },
    /// 归档格式互转
    Convert {
        input: String,
        target_format: nextool_core::ArchiveFormat,
        #[arg(long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum EngineCmd {
    /// 列出所有引擎及可用性(探测系统已装)
    List,
    /// 音视频转换(ffmpeg,--to 指定目标格式如 mp3/wav/aac/mp4)
    Av {
        input: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        output: Option<String>,
    },
    /// Office 文档转 PDF(LibreOffice,输出源文件旁)
    OfficeToPdf { input: String },
    /// 电子书转换(calibre,--to 指定目标格式如 epub/mobi/pdf)
    Ebook {
        input: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        output: Option<String>,
    },
    /// 标记语言转换(pandoc,--to 指定目标格式如 html/rst/adoc/org/tex)
    Markup {
        input: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        output: Option<String>,
    },
    /// PDF 压缩优化(Ghostscript,输出源文件旁 _converted.pdf)
    PdfCompress {
        input: String,
        #[arg(long)]
        output: Option<String>,
    },
    /// OCR 图片转文本(tesseract,输出源文件旁 .txt)
    Ocr { input: String },
}

pub fn run(args: FileConvArgs) -> Result<(), String> {
    match args.cmd {
        FileConvCmd::Archive { cmd } => match cmd {
            ArchiveCmd::List { input } => {
                let data = read_bytes_input(&input)?;
                let list = nextool_core::archive_list(&data).map_err(|e| e.to_string())?;
                println!("{list}");
                Ok(())
            }
            ArchiveCmd::Extract { input, output_dir } => {
                let (out_dir, written) =
                    nextool_core::extract_to_dir(&input, output_dir.as_deref())
                        .map_err(|e| e.to_string())?;
                println!("已解压 {} 个文件到 {out_dir}", written.len());
                Ok(())
            }
            ArchiveCmd::Compress {
                format,
                files,
                output,
            } => {
                let out = nextool_core::compress_files(&files, format, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已创建归档 {out}");
                Ok(())
            }
            ArchiveCmd::Convert {
                input,
                target_format,
                output,
            } => {
                let out = nextool_core::convert_file(&input, target_format, output.as_deref())
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
                let out = nextool_core::convert_image_file(&input, target, output.as_deref())
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
                let out = nextool_core::resize_image_file(&input, width, height, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已缩放 {out}");
                Ok(())
            }
        },
        FileConvCmd::Pdf { cmd } => match cmd {
            PdfCmd::Split { input, output_dir } => {
                let written = nextool_core::split_pdf(&input, output_dir.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已拆分为 {} 个 PDF", written.len());
                Ok(())
            }
            PdfCmd::Rotate { input, output } => {
                let out = nextool_core::rotate_pdf(&input, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已旋转 {out}");
                Ok(())
            }
            PdfCmd::Encrypt {
                input,
                password,
                output,
            } => {
                let out = nextool_core::encrypt_pdf(&input, &password, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已加密 {out}");
                Ok(())
            }
            PdfCmd::Decrypt {
                input,
                password,
                output,
            } => {
                let out = nextool_core::decrypt_pdf(&input, &password, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已解密 {out}");
                Ok(())
            }
        },
        FileConvCmd::Engine { cmd } => match cmd {
            EngineCmd::List => {
                let statuses = nextool_core::engine_statuses(&nextool_core::SubprocessRunner);
                for s in &statuses {
                    let mark = if s.available { "✓" } else { "✗" };
                    println!("{mark} {:<14} {}", s.binary, s.desc);
                }
                let avail = statuses.iter().filter(|s| s.available).count();
                println!("已装 {avail}/{} 个引擎", statuses.len());
                Ok(())
            }
            EngineCmd::Av { input, to, output } => {
                let out = nextool_core::engine_convert_file(
                    &input,
                    nextool_core::Engine::Ffmpeg,
                    &to,
                    output.as_deref(),
                    &nextool_core::SubprocessRunner,
                )
                .map_err(|e| e.to_string())?;
                println!("已转换 {out}");
                Ok(())
            }
            EngineCmd::OfficeToPdf { input } => {
                let out = nextool_core::engine_convert_file(
                    &input,
                    nextool_core::Engine::LibreOffice,
                    "pdf",
                    None,
                    &nextool_core::SubprocessRunner,
                )
                .map_err(|e| e.to_string())?;
                println!("已转换 {out}");
                Ok(())
            }
            EngineCmd::Ebook { input, to, output } => {
                let out = nextool_core::engine_convert_file(
                    &input,
                    nextool_core::Engine::Calibre,
                    &to,
                    output.as_deref(),
                    &nextool_core::SubprocessRunner,
                )
                .map_err(|e| e.to_string())?;
                println!("已转换 {out}");
                Ok(())
            }
            EngineCmd::Markup { input, to, output } => {
                let out = nextool_core::engine_convert_file(
                    &input,
                    nextool_core::Engine::Pandoc,
                    &to,
                    output.as_deref(),
                    &nextool_core::SubprocessRunner,
                )
                .map_err(|e| e.to_string())?;
                println!("已转换 {out}");
                Ok(())
            }
            EngineCmd::PdfCompress { input, output } => {
                let out = nextool_core::engine_convert_file(
                    &input,
                    nextool_core::Engine::Ghostscript,
                    "pdf",
                    output.as_deref(),
                    &nextool_core::SubprocessRunner,
                )
                .map_err(|e| e.to_string())?;
                println!("已压缩 {out}");
                Ok(())
            }
            EngineCmd::Ocr { input } => {
                let out = nextool_core::engine_convert_file(
                    &input,
                    nextool_core::Engine::Tesseract,
                    "txt",
                    None,
                    &nextool_core::SubprocessRunner,
                )
                .map_err(|e| e.to_string())?;
                println!("已识别 {out}");
                Ok(())
            }
        },
    }
}
