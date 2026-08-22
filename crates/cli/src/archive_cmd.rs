//! 文件转换子命令:归档/图像/PDF/字体/SVG/引擎/电子表格
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
    /// 字体操作:TTF↔WOFF 转换/元数据查看
    Font {
        #[command(subcommand)]
        cmd: FontCmd,
    },
    /// SVG 栅格化:SVG→PNG/JPG
    Svg {
        #[command(subcommand)]
        cmd: SvgCmd,
    },
    /// 引擎转换:音视频/Office/电子书/标记/PDF压缩/OCR(运行时探测系统已装引擎)
    Engine {
        #[command(subcommand)]
        cmd: EngineCmd,
    },
    /// 电子表格:XLSX↔JSON 互转
    Xlsx {
        #[command(subcommand)]
        cmd: XlsxCmd,
    },
    /// 文本提取:PDF/DOCX → TXT
    Extract {
        #[command(subcommand)]
        cmd: ExtractCmd,
    },
}

#[derive(Subcommand)]
enum FontCmd {
    /// 字体格式互转(ttf↔woff)
    Convert {
        input: String,
        target: nextool_core::FontFormat,
        #[arg(long)]
        output: Option<String>,
    },
    /// 查看字体元数据(名称/版权/字重/UPM)
    Meta { input: String },
}

#[derive(Subcommand)]
enum SvgCmd {
    /// SVG 栅格化为位图(--target png/jpg)
    Convert {
        input: String,
        target: nextool_core::SvgFormat,
        #[arg(long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum PdfCmd {
    /// 拆分 PDF:默认每页一个;--ranges/--every-n/--parity 三选一(优先级 ranges > every_n > parity)
    Split {
        input: String,
        #[arg(long, help = "自定义范围,如 \"1-3,5,7-10\"")]
        ranges: Option<String>,
        #[arg(long, help = "每 N 页一段")]
        every_n: Option<u32>,
        #[arg(long, help = "奇偶页分离(odd/even)")]
        parity: Option<nextool_core::Parity>,
        #[arg(long)]
        output_dir: Option<String>,
    },
    /// 旋转 PDF 所有页(默认 90 度,--degrees 90/180/270)
    Rotate {
        input: String,
        #[arg(long, default_value = "90")]
        degrees: u32,
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
    /// 合并多个 PDF(顺序拼接,默认输出第一个文件旁)
    Merge {
        #[arg(required = true)]
        files: Vec<String>,
        #[arg(long)]
        output: Option<String>,
    },
    /// 删除指定页(--pages 逗号分隔页号,如 \"2,4,6\")
    DeletePages {
        input: String,
        #[arg(long)]
        pages: String,
        #[arg(long)]
        output: Option<String>,
    },
    /// 提取指定页,删除其余(--pages 逗号分隔页号)
    ExtractPages {
        input: String,
        #[arg(long)]
        pages: String,
        #[arg(long)]
        output: Option<String>,
    },
    /// 设置元数据(--title/--author/--subject/--keywords,仅非空字段写入)
    SetMetadata {
        input: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        author: Option<String>,
        #[arg(long)]
        subject: Option<String>,
        #[arg(long)]
        keywords: Option<String>,
        #[arg(long)]
        output: Option<String>,
    },
    /// 为每页添加右下角页码(1-based)
    AddPageNumbers {
        input: String,
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
    /// 图像裁剪(--x/--y/--width/--height 指定区域)
    Crop {
        input: String,
        #[arg(long)]
        x: u32,
        #[arg(long)]
        y: u32,
        #[arg(long)]
        width: u32,
        #[arg(long)]
        height: u32,
        #[arg(long)]
        output: Option<String>,
    },
    /// 图像翻转(--direction h/v)
    Flip {
        input: String,
        #[arg(long)]
        direction: nextool_core::FlipDirection,
        #[arg(long)]
        output: Option<String>,
    },
    /// 图像滤镜(--filter grayscale/invert/sepia/blur)
    Filter {
        input: String,
        #[arg(long)]
        filter: nextool_core::FilterKind,
        #[arg(long)]
        output: Option<String>,
    },
    /// 亮度/对比度调整(--brightness i32/--contrast f32)
    Adjust {
        input: String,
        #[arg(long, default_value = "0")]
        brightness: i32,
        #[arg(long, default_value = "1.0")]
        contrast: f32,
        #[arg(long)]
        output: Option<String>,
    },
    /// JPEG 压缩(--quality 1..=100)
    CompressJpeg {
        input: String,
        #[arg(long)]
        quality: u8,
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

#[derive(Subcommand)]
enum XlsxCmd {
    /// XLSX → JSON(首个 sheet 转二维数组)
    ToJson {
        input: String,
        #[arg(long)]
        output: Option<String>,
    },
    /// JSON → XLSX(二维数组写首个 sheet)
    FromJson {
        input: String,
        #[arg(long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum ExtractCmd {
    /// PDF → TXT(纯文本提取)
    Pdf {
        input: String,
        #[arg(long)]
        output: Option<String>,
    },
    /// DOCX → TXT(Word 文档文本提取)
    Docx {
        input: String,
        #[arg(long)]
        output: Option<String>,
    },
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
            ImageCmd::Crop {
                input,
                x,
                y,
                width,
                height,
                output,
            } => {
                let out =
                    nextool_core::crop_image_file(&input, x, y, width, height, output.as_deref())
                        .map_err(|e| e.to_string())?;
                println!("已裁剪 {out}");
                Ok(())
            }
            ImageCmd::Flip {
                input,
                direction,
                output,
            } => {
                let out = nextool_core::flip_image_file(&input, direction, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已翻转 {out}");
                Ok(())
            }
            ImageCmd::Filter {
                input,
                filter,
                output,
            } => {
                let out = nextool_core::filter_image_file(&input, filter, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已应用滤镜 {out}");
                Ok(())
            }
            ImageCmd::Adjust {
                input,
                brightness,
                contrast,
                output,
            } => {
                let out = nextool_core::adjust_image_file(
                    &input,
                    brightness,
                    contrast,
                    output.as_deref(),
                )
                .map_err(|e| e.to_string())?;
                println!("已调整亮度/对比度 {out}");
                Ok(())
            }
            ImageCmd::CompressJpeg {
                input,
                quality,
                output,
            } => {
                let out =
                    nextool_core::compress_jpeg_image_file(&input, quality, output.as_deref())
                        .map_err(|e| e.to_string())?;
                println!("已压缩 JPEG {out}");
                Ok(())
            }
        },
        FileConvCmd::Pdf { cmd } => match cmd {
            PdfCmd::Split {
                input,
                ranges,
                every_n,
                parity,
                output_dir,
            } => {
                let written = if let Some(s) = ranges {
                    let r = nextool_core::parse_page_ranges(&s).map_err(|e| e.to_string())?;
                    nextool_core::split_pdf_ranges(&input, &r, output_dir.as_deref())
                } else if let Some(n) = every_n {
                    nextool_core::split_pdf_every_n(&input, n, output_dir.as_deref())
                } else if let Some(p) = parity {
                    nextool_core::split_pdf_parity(&input, p, output_dir.as_deref())
                } else {
                    nextool_core::split_pdf(&input, output_dir.as_deref())
                }
                .map_err(|e| e.to_string())?;
                println!("已拆分为 {} 个 PDF", written.len());
                Ok(())
            }
            PdfCmd::Rotate {
                input,
                degrees,
                output,
            } => {
                let out = nextool_core::rotate_pdf(&input, degrees, output.as_deref())
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
            PdfCmd::Merge { files, output } => {
                let out = nextool_core::merge_pdfs(&files, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已合并 {out}");
                Ok(())
            }
            PdfCmd::DeletePages {
                input,
                pages,
                output,
            } => {
                let nums = parse_page_list(&pages)?;
                let out = nextool_core::delete_pdf_pages(&input, &nums, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已删除页 {out}");
                Ok(())
            }
            PdfCmd::ExtractPages {
                input,
                pages,
                output,
            } => {
                let nums = parse_page_list(&pages)?;
                let out = nextool_core::extract_pdf_pages(&input, &nums, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已提取页 {out}");
                Ok(())
            }
            PdfCmd::SetMetadata {
                input,
                title,
                author,
                subject,
                keywords,
                output,
            } => {
                let out = nextool_core::set_pdf_metadata(
                    &input,
                    title.as_deref(),
                    author.as_deref(),
                    subject.as_deref(),
                    keywords.as_deref(),
                    output.as_deref(),
                )
                .map_err(|e| e.to_string())?;
                println!("已设置元数据 {out}");
                Ok(())
            }
            PdfCmd::AddPageNumbers { input, output } => {
                let out = nextool_core::add_pdf_page_numbers(&input, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已添加页码 {out}");
                Ok(())
            }
        },
        FileConvCmd::Font { cmd } => match cmd {
            FontCmd::Convert {
                input,
                target,
                output,
            } => {
                let out = nextool_core::convert_font_file(&input, target, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已转换 {out}");
                Ok(())
            }
            FontCmd::Meta { input } => {
                let text = nextool_core::read_font_meta_file(&input).map_err(|e| e.to_string())?;
                println!("{text}");
                Ok(())
            }
        },
        FileConvCmd::Svg { cmd } => match cmd {
            SvgCmd::Convert {
                input,
                target,
                output,
            } => {
                let out = nextool_core::convert_svg_file(&input, target, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已栅格化 {out}");
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
        FileConvCmd::Xlsx { cmd } => match cmd {
            XlsxCmd::ToJson { input, output } => {
                let out = nextool_core::xlsx_to_json_file(&input, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已转换 {out}");
                Ok(())
            }
            XlsxCmd::FromJson { input, output } => {
                let out = nextool_core::json_to_xlsx_file(&input, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已转换 {out}");
                Ok(())
            }
        },
        FileConvCmd::Extract { cmd } => match cmd {
            ExtractCmd::Pdf { input, output } => {
                let out = nextool_core::pdf_to_text_file(&input, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已提取 {out}");
                Ok(())
            }
            ExtractCmd::Docx { input, output } => {
                let out = nextool_core::docx_to_text_file(&input, output.as_deref())
                    .map_err(|e| e.to_string())?;
                println!("已提取 {out}");
                Ok(())
            }
        },
    }
}

/// 解析逗号分隔页号列表 "2,4,6" → Vec<u32>
fn parse_page_list(s: &str) -> Result<Vec<u32>, String> {
    s.split(',')
        .map(|p| {
            p.trim()
                .parse::<u32>()
                .map_err(|e| format!("页号解析失败: {e}"))
        })
        .collect()
}
