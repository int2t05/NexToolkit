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

// 引擎状态(运行时探测,供前端展示哪些引擎已装)

#[derive(serde::Serialize)]
pub struct EngineStatusDto {
    pub binary: String,
    pub desc: String,
    pub available: bool,
}

/// 探测所有引擎可用性(ffmpeg/LibreOffice/calibre/pandoc/Ghostscript/tesseract)
#[tauri::command]
pub fn list_engines() -> Vec<EngineStatusDto> {
    nextool_core::engine_statuses(&nextool_core::SubprocessRunner)
        .into_iter()
        .map(|s| EngineStatusDto {
            binary: s.binary.into(),
            desc: s.desc.into(),
            available: s.available,
        })
        .collect()
}

/// 文件工具静态元数据(list_file_tools 返回,前端动态渲染)
///
/// output_kind 统一为 Text(产物为路径或路径列表,无需语法高亮);
/// params 的 key 用 camelCase,经 Tauri 映射到 Rust 命令的 snake_case 形参。
static FILE_TOOLS: &[ToolMeta] = &[
    ToolMeta {
        id: "convert_file",
        name: "文件转换",
        desc: "通用格式转换(Office/MD/电子书/PDF → 任意格式)",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "源文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "target",
                kind: ParamKind::Select,
                label: "目标格式",
                default: Some("pdf"),
                options: &[
                    "pdf", "docx", "doc", "xlsx", "xls", "pptx", "odt", "ods", "odp", "rtf",
                    "html", "md", "txt", "rst", "adoc", "org", "tex", "wiki", "epub", "mobi",
                    "azw3", "csv", "json", "yaml", "xml", "png", "jpg", "webp",
                ],
                placeholder: None,
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "archive_list",
        name: "归档列表",
        desc: "列出归档内文件(zip/tar/gz/7z/bz2/xz/zst)",
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
                options: &["zip", "tar", "targz", "gz", "7z", "bz2", "xz", "zst"],
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
                options: &["zip", "tar", "targz", "gz", "7z", "bz2", "xz", "zst"],
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
                options: &[
                    "png", "jpg", "gif", "bmp", "webp", "tiff", "ico", "dds", "farbfeld", "hdr",
                    "exr", "pnm", "qoi", "tga",
                ],
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
        id: "image_crop",
        name: "图像裁剪",
        desc: "区域裁剪(同格式输出)",
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
                key: "x",
                kind: ParamKind::Number,
                label: "起点 X",
                default: Some("0"),
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "y",
                kind: ParamKind::Number,
                label: "起点 Y",
                default: Some("0"),
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "width",
                kind: ParamKind::Number,
                label: "裁剪宽",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "height",
                kind: ParamKind::Number,
                label: "裁剪高",
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
        id: "image_flip",
        name: "图像翻转",
        desc: "水平/垂直镜像(同格式输出)",
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
                key: "direction",
                kind: ParamKind::Select,
                label: "方向",
                default: Some("h"),
                options: &["h", "v"],
                placeholder: None,
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "image_filter",
        name: "图像滤镜",
        desc: "灰度/反相/棕褐/模糊(同格式输出)",
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
                key: "filter",
                kind: ParamKind::Select,
                label: "滤镜",
                default: Some("grayscale"),
                options: &["grayscale", "invert", "sepia", "blur"],
                placeholder: None,
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "image_adjust",
        name: "亮度/对比度",
        desc: "调整亮度与对比度(同格式输出)",
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
                key: "brightness",
                kind: ParamKind::Number,
                label: "亮度(-255..255)",
                default: Some("0"),
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "contrast",
                kind: ParamKind::Number,
                label: "对比度(0.0-3.0)",
                default: Some("1.0"),
                options: &[],
                placeholder: None,
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "image_compress_jpeg",
        name: "JPEG 压缩",
        desc: "按质量压缩为 JPEG",
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
                key: "quality",
                kind: ParamKind::Number,
                label: "质量(1-100)",
                default: Some("80"),
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
        desc: "所有页旋转 90/180/270°",
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
                key: "degrees",
                kind: ParamKind::Number,
                label: "角度(90/180/270)",
                default: Some("90"),
                options: &[],
                placeholder: None,
                multiple: false,
            },
        ],
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
        id: "pdf_split_ranges",
        name: "PDF 范围拆分",
        desc: "按范围拆分(如 1-3,5)",
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
                key: "ranges",
                kind: ParamKind::Text,
                label: "范围",
                default: None,
                options: &[],
                placeholder: Some("如 1-3,5,7-10"),
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
        id: "pdf_split_every_n",
        name: "PDF 每 N 页拆分",
        desc: "每 N 页一段",
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
                key: "n",
                kind: ParamKind::Number,
                label: "每段页数",
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
        id: "pdf_split_parity",
        name: "PDF 奇偶页拆分",
        desc: "分离奇/偶页",
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
                key: "parity",
                kind: ParamKind::Select,
                label: "奇偶",
                default: Some("odd"),
                options: &["odd", "even"],
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
        id: "pdf_merge",
        name: "PDF 合并",
        desc: "多个 PDF 顺序拼接",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "paths",
                kind: ParamKind::File,
                label: "PDF 文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: true,
            },
            ParamSpec {
                key: "output",
                kind: ParamKind::Text,
                label: "输出路径",
                default: None,
                options: &[],
                placeholder: Some("默认第一个文件旁"),
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "pdf_delete_pages",
        name: "PDF 删除页",
        desc: "删除指定页",
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
                key: "pages",
                kind: ParamKind::Text,
                label: "页号",
                default: None,
                options: &[],
                placeholder: Some("如 2,4,6"),
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
        id: "pdf_extract_pages",
        name: "PDF 提取页",
        desc: "保留指定页,删其余",
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
                key: "pages",
                kind: ParamKind::Text,
                label: "页号",
                default: None,
                options: &[],
                placeholder: Some("如 1,3,5"),
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
        id: "pdf_set_metadata",
        name: "PDF 元数据",
        desc: "设置标题/作者/主题/关键词",
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
                key: "title",
                kind: ParamKind::Text,
                label: "标题",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "author",
                kind: ParamKind::Text,
                label: "作者",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "subject",
                kind: ParamKind::Text,
                label: "主题",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "keywords",
                kind: ParamKind::Text,
                label: "关键词",
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
                placeholder: Some("默认源文件旁"),
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "pdf_add_page_numbers",
        name: "PDF 页码",
        desc: "每页右下角加页码",
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
                placeholder: Some("默认源文件旁"),
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "font_convert",
        name: "字体转换",
        desc: "TTF/OTF↔WOFF 互转",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "字体文件",
                default: None,
                options: &[],
                placeholder: None,
                multiple: false,
            },
            ParamSpec {
                key: "target",
                kind: ParamKind::Select,
                label: "目标格式",
                default: Some("woff"),
                options: &["ttf", "woff"],
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
        id: "font_meta",
        name: "字体元数据",
        desc: "查看名称/版权/字重/UPM",
        group: "fileconv",
        params: &[ParamSpec {
            key: "path",
            kind: ParamKind::File,
            label: "字体文件",
            default: None,
            options: &[],
            placeholder: None,
            multiple: false,
        }],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "svg_convert",
        name: "SVG 栅格化",
        desc: "SVG→PNG/JPG(resvg)",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "SVG 文件",
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
                options: &["png", "jpg"],
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
    ToolMeta {
        id: "xlsx_to_json",
        name: "XLSX 转 JSON",
        desc: "电子表格转二维数组 JSON",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "XLSX 文件",
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
                placeholder: Some("默认源文件旁 .json"),
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "json_to_xlsx",
        name: "JSON 转 XLSX",
        desc: "二维数组 JSON 写电子表格",
        group: "fileconv",
        params: &[
            ParamSpec {
                key: "path",
                kind: ParamKind::File,
                label: "JSON 文件",
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
                placeholder: Some("默认源文件旁 .xlsx"),
                multiple: false,
            },
        ],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
    ToolMeta {
        id: "pdf_to_text",
        name: "PDF 提取文本",
        desc: "PDF 转纯文本",
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
        id: "docx_to_text",
        name: "DOCX 提取文本",
        desc: "Word 文档转纯文本",
        group: "fileconv",
        params: &[ParamSpec {
            key: "path",
            kind: ParamKind::File,
            label: "DOCX 文件",
            default: None,
            options: &[],
            placeholder: None,
            multiple: false,
        }],
        needs_main_input: false,
        output_kind: OutputKind::Text,
    },
];

// 文件转换命令(接收路径,委托 fileconv 落盘,返回路径或路径列表)

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

/// 图像裁剪(默认输出到源文件旁,同格式);返回产物路径
#[tauri::command]
pub fn image_crop(
    path: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    output: Option<String>,
) -> CmdResult<String> {
    Ok(nextool_core::crop_image_file(
        &path,
        x,
        y,
        width,
        height,
        output.as_deref(),
    )?)
}

/// 图像翻转(默认输出到源文件旁,同格式);返回产物路径
#[tauri::command]
pub fn image_flip(path: String, direction: String, output: Option<String>) -> CmdResult<String> {
    let dir = direction
        .parse::<nextool_core::FlipDirection>()
        .map_err(|e| CmdError(e.to_string()))?;
    Ok(nextool_core::flip_image_file(
        &path,
        dir,
        output.as_deref(),
    )?)
}

/// 图像滤镜(默认输出到源文件旁,同格式);返回产物路径
#[tauri::command]
pub fn image_filter(path: String, filter: String, output: Option<String>) -> CmdResult<String> {
    let f = filter
        .parse::<nextool_core::FilterKind>()
        .map_err(|e| CmdError(e.to_string()))?;
    Ok(nextool_core::filter_image_file(
        &path,
        f,
        output.as_deref(),
    )?)
}

/// 亮度/对比度调整(默认输出到源文件旁,同格式);返回产物路径
#[tauri::command]
pub fn image_adjust(
    path: String,
    brightness: i32,
    contrast: f32,
    output: Option<String>,
) -> CmdResult<String> {
    Ok(nextool_core::adjust_image_file(
        &path,
        brightness,
        contrast,
        output.as_deref(),
    )?)
}

/// JPEG 压缩(输出 JPEG 格式,默认源文件旁 .jpg);返回产物路径
#[tauri::command]
pub fn image_compress_jpeg(path: String, quality: u8, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::compress_jpeg_image_file(
        &path,
        quality,
        output.as_deref(),
    )?)
}

/// 拆分 PDF:每页一个独立 PDF,返回产物路径列表
#[tauri::command]
pub fn pdf_split(path: String, output_dir: Option<String>) -> CmdResult<Vec<String>> {
    Ok(nextool_core::split_pdf(&path, output_dir.as_deref())?)
}

/// 旋转 PDF 所有页指定角度(90/180/270);返回产物路径
#[tauri::command]
pub fn pdf_rotate(path: String, degrees: u32, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::rotate_pdf(&path, degrees, output.as_deref())?)
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

/// 按范围拆分 PDF(返回产物路径列表)
#[tauri::command]
pub fn pdf_split_ranges(
    path: String,
    ranges: String,
    output_dir: Option<String>,
) -> CmdResult<Vec<String>> {
    let r = nextool_core::parse_page_ranges(&ranges)?;
    Ok(nextool_core::split_pdf_ranges(
        &path,
        &r,
        output_dir.as_deref(),
    )?)
}

/// 每 N 页拆分 PDF(返回产物路径列表)
#[tauri::command]
pub fn pdf_split_every_n(
    path: String,
    n: u32,
    output_dir: Option<String>,
) -> CmdResult<Vec<String>> {
    Ok(nextool_core::split_pdf_every_n(
        &path,
        n,
        output_dir.as_deref(),
    )?)
}

/// 按奇偶页拆分 PDF(返回产物路径列表,单元素)
#[tauri::command]
pub fn pdf_split_parity(
    path: String,
    parity: String,
    output_dir: Option<String>,
) -> CmdResult<Vec<String>> {
    let p = parity
        .parse::<nextool_core::Parity>()
        .map_err(|e| CmdError(e.to_string()))?;
    Ok(nextool_core::split_pdf_parity(
        &path,
        p,
        output_dir.as_deref(),
    )?)
}

/// 合并多个 PDF(默认输出第一个文件旁);返回产物路径
#[tauri::command]
pub fn pdf_merge(paths: Vec<String>, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::merge_pdfs(&paths, output.as_deref())?)
}

/// 删除 PDF 指定页(返回产物路径)
#[tauri::command]
pub fn pdf_delete_pages(path: String, pages: String, output: Option<String>) -> CmdResult<String> {
    let nums = parse_pages(&pages)?;
    Ok(nextool_core::delete_pdf_pages(
        &path,
        &nums,
        output.as_deref(),
    )?)
}

/// 提取 PDF 指定页(返回产物路径)
#[tauri::command]
pub fn pdf_extract_pages(path: String, pages: String, output: Option<String>) -> CmdResult<String> {
    let nums = parse_pages(&pages)?;
    Ok(nextool_core::extract_pdf_pages(
        &path,
        &nums,
        output.as_deref(),
    )?)
}

/// 设置 PDF 元数据(返回产物路径)
#[tauri::command]
pub fn pdf_set_metadata(
    path: String,
    title: Option<String>,
    author: Option<String>,
    subject: Option<String>,
    keywords: Option<String>,
    output: Option<String>,
) -> CmdResult<String> {
    Ok(nextool_core::set_pdf_metadata(
        &path,
        title.as_deref(),
        author.as_deref(),
        subject.as_deref(),
        keywords.as_deref(),
        output.as_deref(),
    )?)
}

/// 为 PDF 每页添加页码(返回产物路径)
#[tauri::command]
pub fn pdf_add_page_numbers(path: String, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::add_pdf_page_numbers(
        &path,
        output.as_deref(),
    )?)
}

/// 解析逗号分隔页号列表 "2,4,6" → Vec<u32>
fn parse_pages(s: &str) -> CmdResult<Vec<u32>> {
    s.split(',')
        .map(|p| {
            p.trim()
                .parse::<u32>()
                .map_err(|e| CmdError(format!("页号解析失败: {e}")))
        })
        .collect()
}

/// 字体格式互转(默认输出到源文件旁);返回产物路径
#[tauri::command]
pub fn font_convert(path: String, target: String, output: Option<String>) -> CmdResult<String> {
    let fmt = target
        .parse::<nextool_core::FontFormat>()
        .map_err(|e| CmdError(e.to_string()))?;
    Ok(nextool_core::convert_font_file(
        &path,
        fmt,
        output.as_deref(),
    )?)
}

/// 读取字体元数据(名称/版权/字重/UPM);返回格式化文本
#[tauri::command]
pub fn font_meta(path: String) -> CmdResult<String> {
    Ok(nextool_core::read_font_meta_file(&path)?)
}

/// SVG 栅格化为 PNG/JPG(默认输出到源文件旁);返回产物路径
#[tauri::command]
pub fn svg_convert(path: String, target: String, output: Option<String>) -> CmdResult<String> {
    let fmt = target
        .parse::<nextool_core::SvgFormat>()
        .map_err(|e| CmdError(e.to_string()))?;
    Ok(nextool_core::convert_svg_file(
        &path,
        fmt,
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

/// XLSX → JSON(首个 sheet 转二维数组,输出源文件旁 .json);返回产物路径
#[tauri::command]
pub fn xlsx_to_json(input: String, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::xlsx_to_json_file(&input, output.as_deref())?)
}

/// JSON → XLSX(二维数组写首个 sheet,输出源文件旁 .xlsx);返回产物路径
#[tauri::command]
pub fn json_to_xlsx(input: String, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::json_to_xlsx_file(&input, output.as_deref())?)
}

/// PDF → TXT(纯文本提取,输出源文件旁 .txt);返回产物路径
#[tauri::command]
pub fn pdf_to_text(input: String, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::pdf_to_text_file(&input, output.as_deref())?)
}

/// DOCX → TXT(Word 文档文本提取,输出源文件旁 .txt);返回产物路径
#[tauri::command]
pub fn docx_to_text(input: String, output: Option<String>) -> CmdResult<String> {
    Ok(nextool_core::docx_to_text_file(&input, output.as_deref())?)
}

/// 通用文件转换:按源格式自动路由(Office/MD/电子书/PDF → 任意目标格式)
#[tauri::command]
pub fn convert_file(input: String, target: String) -> CmdResult<String> {
    Ok(nextool_core::convert_any(
        &input,
        &target,
        &nextool_core::SubprocessRunner,
    )?)
}
