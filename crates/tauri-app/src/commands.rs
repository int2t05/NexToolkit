//! Tauri command 层:薄封装 nextool-core/fileconv,供前端 invoke 调用
//!
//! 文本工具经 list_tools/run_tool 通用入口(前端动态渲染);文件工具签名各异,保留独立命令。
//! 错误经 serde 序列化为前端可读字符串。

use nextool_core::registry::{OutputKind, ParamKind, ParamSpec, ToolMeta};
use nextool_core::ToolError;
use nextool_core::{ArchiveFormat, ImageFormat};

/// 命令错误:序列化为字符串供前端展示
#[derive(Debug, serde::Serialize)]
pub struct CmdError(String);

impl From<ToolError> for CmdError {
    fn from(e: ToolError) -> Self {
        CmdError(e.to_string())
    }
}

type CmdResult<T> = Result<T, CmdError>;

// 通用工具注册表(list_tools/run_tool/list_file_tools:供前端动态渲染与执行)

#[derive(serde::Serialize)]
pub struct ParamSpecDto {
    pub key: String,
    pub kind: &'static str,
    pub label: String,
    pub default: Option<String>,
    pub options: Vec<String>,
    pub placeholder: Option<String>,
    pub multiple: bool,
}

#[derive(serde::Serialize)]
pub struct ToolMetaDto {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub group: String,
    pub params: Vec<ParamSpecDto>,
    pub needs_main_input: bool,
    pub output_kind: String,
}

fn param_kind_str(k: ParamKind) -> &'static str {
    match k {
        ParamKind::Text => "text",
        ParamKind::Textarea => "textarea",
        ParamKind::Select => "select",
        ParamKind::Number => "number",
        ParamKind::Password => "password",
        ParamKind::Bool => "bool",
        ParamKind::File => "file",
    }
}

fn output_kind_str(k: OutputKind) -> String {
    match k {
        OutputKind::Text => "text".into(),
        OutputKind::Highlight(lang) => (*lang).into(),
        OutputKind::Svg => "svg".into(),
    }
}

/// 静态元数据 → DTO(owned,供 serde 序列化回前端)
fn tool_meta_to_dto(m: &ToolMeta) -> ToolMetaDto {
    ToolMetaDto {
        id: m.id.into(),
        name: m.name.into(),
        desc: m.desc.into(),
        group: m.group.into(),
        params: m
            .params
            .iter()
            .map(|p| ParamSpecDto {
                key: p.key.into(),
                kind: param_kind_str(p.kind),
                label: p.label.into(),
                default: p.default.map(Into::into),
                options: p.options.iter().map(|&s| s.into()).collect(),
                placeholder: p.placeholder.map(Into::into),
                multiple: p.multiple,
            })
            .collect(),
        needs_main_input: m.needs_main_input,
        output_kind: output_kind_str(m.output_kind),
    }
}

/// 文本工具元数据(前端动态渲染工具列表)
#[tauri::command]
pub fn list_tools() -> Vec<ToolMetaDto> {
    nextool_core::tools()
        .iter()
        .map(|t| tool_meta_to_dto(t.meta()))
        .collect()
}

/// 通用文本工具执行器:按 id 查注册表分发
#[tauri::command]
pub fn run_tool(id: String, input: String, args: Vec<(String, String)>) -> CmdResult<String> {
    let tool = nextool_core::find_tool(&id).ok_or_else(|| CmdError(format!("未知工具: {id}")))?;
    let args = nextool_core::ToolArgs::new(&args);
    Ok(tool.run(&input, &args)?)
}

/// 文件工具元数据(前端动态渲染文件转换工具列表)
///
/// 文件工具签名各异,无法经 run_tool 统一执行,但元数据可复用同一渲染逻辑。
#[tauri::command]
pub fn list_file_tools() -> Vec<ToolMetaDto> {
    FILE_TOOLS.iter().map(tool_meta_to_dto).collect()
}

/// 文件工具静态元数据:与 tools.ts 的 fileconv 段对齐
///
/// output_kind 统一为 Text(产物为路径或路径列表,无需语法高亮);
/// params 的 key 用 camelCase,经 Tauri 映射到 Rust 命令的 snake_case 形参。
static FILE_TOOLS: &[ToolMeta] = &[
    ToolMeta {
        id: "archive_list",
        name: "归档列表",
        desc: "列出归档内文件(zip/tar/gz)",
        group: "fileconv",
        params: &[ParamSpec {
            key: "path",
            kind: ParamKind::File,
            label: "归档文件",
            default: None,
            options: &[],
            placeholder: None,
            multiple: false,
        }],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "archive_extract",
        name: "解压归档",
        desc: "解压到源文件旁目录",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "归档文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "outputDir",
                kind: ParamKind::Text,
                label: "输出目录",
                default: None,
                options: &[],
                placeholder: Some("默认源文件旁"),
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "archive_compress",
        name: "压缩文件",
        desc: "创建归档(zip/tar/gz)",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "paths",
                kind: ParamKind::File,
                label: "文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: true,
            },
            ParamSpec {
                key: "format",
                kind: ParamKind::Select,
                label: "格式",
                default: Some("zip"),
                options: &["zip", "tar", "targz", "gz", "7z"],
                placeholder: None,
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "archive_convert",
        name: "归档转换",
        desc: "归档格式互转",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "归档文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "targetFormat",
                kind: ParamKind::Select,
                label: "目标格式",
                default: Some("zip"),
                options: &["zip", "tar", "targz", "gz", "7z"],
                placeholder: None,
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "image_convert",
        name: "图像转换",
        desc: "图像格式互转(png/jpg/gif/bmp/webp/tiff/ico)",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "图像文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "target",
                kind: ParamKind::Select,
                label: "目标格式",
                default: Some("png"),
                options: &["png", "jpg", "gif", "bmp", "webp", "tiff", "ico"],
                placeholder: None,
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "image_resize",
        name: "图像缩放",
        desc: "缩放(一维 0 等比)",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "图像文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "width",
                kind: ParamKind::Number,
                label: "宽",
                default: Some("0"),
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "height",
                kind: ParamKind::Number,
                label: "高",
                default: Some("0"),
                options: &[],
                placeholder: None,
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "pdf_split",
        name: "PDF 拆分",
        desc: "每页一个 PDF",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "PDF 文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "outputDir",
                kind: ParamKind::Text,
                label: "输出目录",
                default: None,
                options: &[],
                placeholder: Some("默认源文件旁"),
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "pdf_rotate",
        name: "PDF 旋转",
        desc: "所有页顺时针 90°",
        group: "fileconv",
        params: &[ParamSpec {
            key: "path",
            kind: ParamKind::File,
            label: "PDF 文件",
            default: None,
            options: &[],
            placeholder: None,
            multiple: false,
        }],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "pdf_encrypt",
        name: "PDF 加密",
        desc: "口令加密(AES)",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "PDF 文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "password",
                kind: ParamKind::Password,
                label: "口令",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "pdf_decrypt",
        name: "PDF 解密",
        desc: "口令解密",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "PDF 文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "password",
                kind: ParamKind::Password,
                label: "口令",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "av_convert",
        name: "音视频转换",
        desc: "ffmpeg 转码(音视频格式互转)",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "音视频文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "to",
                kind: ParamKind::Select,
                label: "目标格式",
                default: Some("mp3"),
                options: &[
                    "mp3", "wav", "aac", "flac", "ogg", "m4a", "mp4", "mkv", "webm", "mov", "avi",
                ],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "output",
                kind: ParamKind::Text,
                label: "输出路径",
                default: None,
                options: &[],
                placeholder: Some("默认源文件旁"),
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "office_to_pdf",
        name: "Office 转 PDF",
        desc: "LibreOffice(docx/xlsx/pptx→pdf)",
        group: "fileconv",
        params: &[ParamSpec {
            key: "path",
            kind: ParamKind::File,
            label: "Office 文档",
            default: None,
            options: &[],
            placeholder: None,
            multiple: false,
        }],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "ebook_convert",
        name: "电子书转换",
        desc: "calibre(epub/mobi/pdf 互转)",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "电子书文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "to",
                kind: ParamKind::Select,
                label: "目标格式",
                default: Some("epub"),
                options: &["epub", "mobi", "pdf", "txt", "azw3"],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "output",
                kind: ParamKind::Text,
                label: "输出路径",
                default: None,
                options: &[],
                placeholder: Some("默认源文件旁"),
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "markup_convert",
        name: "标记语言转换",
        desc: "pandoc(md/html/rst/adoc/org/tex 互转)",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "标记文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "to",
                kind: ParamKind::Select,
                label: "目标格式",
                default: Some("html"),
                options: &["html", "md", "rst", "adoc", "org", "tex", "docx"],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "output",
                kind: ParamKind::Text,
                label: "输出路径",
                default: None,
                options: &[],
                placeholder: Some("默认源文件旁"),
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "pdf_compress",
        name: "PDF 压缩",
        desc: "Ghostscript 优化(输出 _converted.pdf)",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "PDF 文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "output",
                kind: ParamKind::Text,
                label: "输出路径",
                default: None,
                options: &[],
                placeholder: Some("默认源文件旁 _converted.pdf"),
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "ocr",
        name: "OCR 识别",
        desc: "tesseract 图片转文本",
        group: "fileconv",
        params: &[ParamSpec {
            key: "path",
            kind: ParamKind::File,
            label: "图片文件",
            default: None,
            options: &[],
            placeholder: None,
            multiple: false,
        }],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
];

// 文件转换命令:归档/图像/PDF(接收路径,委托 fileconv 落盘,返回路径或路径列表)

/// 列出归档内文件(每行 `路径\t大小`)
#[tauri::command]
pub fn archive_list(path: String) -> CmdResult<String> {
    let data = std::fs::read(&path).map_err(|e| CmdError(e.to_string()))?;
    Ok(nextool_core::archive_list(&data)?)
}

/// 解压归档到目录(默认源文件旁 `_extracted`,碰撞追加 `(n)`);返回写出的文件路径列表
#[tauri::command]
pub fn archive_extract(path: String, output_dir: Option<String>) -> CmdResult<Vec<String>> {
    let (_out_dir, written) = nextool_core::extract_to_dir(&path, output_dir.as_deref())?;
    Ok(written)
}

/// 压缩文件为归档(默认输出到第一个文件旁);返回产物路径
#[tauri::command]
pub fn archive_compress(
    paths: Vec<String>,
    format: String,
    output: Option<String>,
) -> CmdResult<String> {
    let fmt = format
        .parse::<ArchiveFormat>()
        .map_err(|e| CmdError(e.to_string()))?;
    Ok(nextool_core::compress_files(
        &paths,
        fmt,
        output.as_deref(),
    )?)
}

/// 归档格式互转(默认输出到源文件旁);返回产物路径
#[tauri::command]
pub fn archive_convert(
    path: String,
    target_format: String,
    output: Option<String>,
) -> CmdResult<String> {
    let fmt = target_format
        .parse::<ArchiveFormat>()
        .map_err(|e| CmdError(e.to_string()))?;
    Ok(nextool_core::convert_file(&path, fmt, output.as_deref())?)
}

/// 图像格式转换(默认输出到源文件旁);返回产物路径
#[tauri::command]
pub fn image_convert(path: String, target: String, output: Option<String>) -> CmdResult<String> {
    let fmt = target
        .parse::<ImageFormat>()
        .map_err(|e| CmdError(e.to_string()))?;
    Ok(nextool_core::convert_image_file(
        &path,
        fmt,
        output.as_deref(),
    )?)
}

/// 图像缩放(默认输出到源文件旁,同格式);返回产物路径
///
/// `width`/`height` 一维为 0 时按另一维等比缩放。
#[tauri::command]
pub fn image_resize(
    path: String,
    width: u32,
    height: u32,
    output: Option<String>,
) -> CmdResult<String> {
    Ok(nextool_core::resize_image_file(
        &path,
        width,
        height,
        output.as_deref(),
    )?)
}

/// 拆分 PDF:每页一个独立 PDF,返回产物路径列表
#[tauri::command]
pub fn pdf_split(path: String, output_dir: Option<String>) -> CmdResult<Vec<String>> {
    Ok(nextool_core::split_pdf(&path, output_dir.as_deref())?)
}

/// 旋转 PDF 所有页 90 度(顺时针);返回产物路径
#[tauri::command]
pub fn pdf_rotate(path: String, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::rotate_pdf(&path, output.as_deref())?)
}

/// 加密 PDF(--password);返回产物路径
#[tauri::command]
pub fn pdf_encrypt(path: String, password: String, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::encrypt_pdf(
        &path,
        &password,
        output.as_deref(),
    )?)
}

/// 解密 PDF(--password);返回产物路径
#[tauri::command]
pub fn pdf_decrypt(path: String, password: String, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::decrypt_pdf(
        &path,
        &password,
        output.as_deref(),
    )?)
}

/// 音视频转换(ffmpeg,运行时探测);返回产物路径
#[tauri::command]
pub fn av_convert(input: String, to: String, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::engine_convert_file(
        &input,
        nextool_core::Engine::Ffmpeg,
        &to,
        output.as_deref(),
        &nextool_core::SubprocessRunner,
    )?)
}

/// Office 文档转 PDF(LibreOffice,输出源文件旁);返回产物路径
#[tauri::command]
pub fn office_to_pdf(input: String) -> CmdResult<String> {
    Ok(nextool_core::engine_convert_file(
        &input,
        nextool_core::Engine::LibreOffice,
        "pdf",
        None,
        &nextool_core::SubprocessRunner,
    )?)
}

/// 电子书转换(calibre,运行时探测);返回产物路径
#[tauri::command]
pub fn ebook_convert(input: String, to: String, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::engine_convert_file(
        &input,
        nextool_core::Engine::Calibre,
        &to,
        output.as_deref(),
        &nextool_core::SubprocessRunner,
    )?)
}

/// 标记语言转换(pandoc,运行时探测);返回产物路径
#[tauri::command]
pub fn markup_convert(input: String, to: String, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::engine_convert_file(
        &input,
        nextool_core::Engine::Pandoc,
        &to,
        output.as_deref(),
        &nextool_core::SubprocessRunner,
    )?)
}

/// PDF 压缩优化(Ghostscript,输出源文件旁 _converted.pdf);返回产物路径
#[tauri::command]
pub fn pdf_compress(input: String, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::engine_convert_file(
        &input,
        nextool_core::Engine::Ghostscript,
        "pdf",
        output.as_deref(),
        &nextool_core::SubprocessRunner,
    )?)
}

/// OCR 图片转文本(tesseract,输出源文件旁 .txt);返回产物路径
#[tauri::command]
pub fn ocr(input: String) -> CmdResult<String> {
    Ok(nextool_core::engine_convert_file(
        &input,
        nextool_core::Engine::Tesseract,
        "txt",
        None,
        &nextool_core::SubprocessRunner,
    )?)
}
