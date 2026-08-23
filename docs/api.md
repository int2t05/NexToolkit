# 接口契约

Tauri command 契约,前端经 `invoke` 调用。36 个命令:4 通用入口 + 32 文件命令。

## 通用入口

### list_tools

```ts
invoke<ToolMetaDto[]>('list_tools')
```

返回 109 个文本工具元数据,前端按 `group` 分组渲染。

### run_tool

```ts
invoke<string>('run_tool', { id: string, input: string, args: [string, string][] })
```

按 `id` 查注册表分发。`args` 为 `Vec<(String,String)>`,前端传 `[[key,val],...]`。未知 `id` 返回 `CmdError("未知工具: {id}")`。

### list_file_tools

```ts
invoke<ToolMetaDto[]>('list_file_tools')
```

返回 32 个文件工具元数据,复用同一渲染逻辑。

### list_engines

```ts
invoke<EngineStatusDto[]>('list_engines')
```

```ts
interface EngineStatusDto {
  binary: string;        // "ffmpeg" | "soffice" | ...
  desc: string;          // "音视频转码" | ...
  available: boolean;    // 运行时探测
  resolved_path: string | null;  // 实际找到的路径
}
```

### engine_install_infos

```ts
invoke<EngineInstallInfoDto[]>('engine_install_infos')
```

```ts
interface EngineInstallInfoDto {
  binary: string;
  desc: string;
  available: boolean;
  is_portable: boolean;       // ffmpeg/pandoc = true
  download_url: string;       // 下载页 URL
  install_path: string | null;
}
```

### install_engine

```ts
invoke<string>('install_engine', { engine: string })
```

下载并解压便携版引擎到 `%APPDATA%/NexToolkit/engines/`,返回二进制路径。仅支持便携版(ffmpeg/pandoc),安装包引擎返回错误提示手动下载。

## 文件转换命令

### 归档(4)

| 命令 | 参数 | 返回 |
|---|---|---|
| `archive_list` | `path` | 文件列表(每行 `路径\t大小`) |
| `archive_extract` | `path`, `outputDir?` | `string[]` 写出文件路径 |
| `archive_compress` | `paths[]`, `format`, `output?` | 产物路径 |
| `archive_convert` | `path`, `targetFormat`, `output?` | 产物路径 |

归档格式:`zip`/`tar`/`targz`/`gz`/`7z`/`bz2`/`xz`/`zst`。

### 图像(7)

| 命令 | 参数 | 返回 |
|---|---|---|
| `image_convert` | `path`, `target`, `output?` | 产物路径 |
| `image_resize` | `path`, `width`, `height`, `output?` | 产物路径 |
| `image_crop` | `path`, `x`, `y`, `width`, `height`, `output?` | 产物路径 |
| `image_flip` | `path`, `direction`(h/v), `output?` | 产物路径 |
| `image_filter` | `path`, `filter`(grayscale/invert/sepia/blur), `output?` | 产物路径 |
| `image_adjust` | `path`, `brightness`, `contrast`, `output?` | 产物路径 |
| `image_compress_jpeg` | `path`, `quality`(1-100), `output?` | 产物路径(JPEG) |

图像格式:14 种(png/jpg/gif/bmp/webp/tiff/ico/dds/farbfeld/hdr/exr/pnm/qoi/tga)。

### PDF(12)

| 命令 | 参数 | 返回 |
|---|---|---|
| `pdf_split` | `path`, `outputDir?` | `string[]` 拆分件路径 |
| `pdf_split_ranges` | `path`, `ranges`(如 "1-3,5"), `outputDir?` | `string[]` |
| `pdf_split_every_n` | `path`, `n`, `outputDir?` | `string[]` |
| `pdf_split_parity` | `path`, `parity`(odd/even), `outputDir?` | `string[]` |
| `pdf_rotate` | `path`, `degrees`(90/180/270), `output?` | 产物路径 |
| `pdf_encrypt` | `path`, `password`, `output?` | 产物路径 |
| `pdf_decrypt` | `path`, `password`, `output?` | 产物路径 |
| `pdf_merge` | `paths[]`, `output?` | 产物路径 |
| `pdf_delete_pages` | `path`, `pages`(如 "2,4,6"), `output?` | 产物路径 |
| `pdf_extract_pages` | `path`, `pages`, `output?` | 产物路径 |
| `pdf_set_metadata` | `path`, `title?/author?/subject?/keywords?`, `output?` | 产物路径 |
| `pdf_add_page_numbers` | `path`, `output?` | 产物路径 |

### 字体(2)

| 命令 | 参数 | 返回 |
|---|---|---|
| `font_convert` | `path`, `target`(ttf/woff), `output?` | 产物路径 |
| `font_meta` | `path` | 元数据文本 |

### SVG(1)

| `svg_convert` | `path`, `target`(png/jpg), `output?` | 产物路径 |

### 电子表格(2)

| `xlsx_to_json` | `path`, `output?` | 产物路径(JSON) |
| `json_to_xlsx` | `path`, `output?` | 产物路径(XLSX) |

### 文本提取(1)

| `docx_to_text` | `path`, `output?` | 产物路径(TXT) |

### 引擎转换(2)

| `av_convert` | `path`, `to`, `output?` | 产物路径(ffmpeg) |
| `ocr` | `path` | 产物路径(TXT,tesseract) |

### 通用转换(1)

| `convert_file` | `path`, `target` | 产物路径 |

按源格式自动路由:Office→LibreOffice · Markup→pandoc · Ebook→calibre · PDF→ghostscript(压缩)/纯 Rust(提取文本)/LibreOffice(其他)。

## DTO

```ts
interface ParamSpecDto {
  key: string;
  kind: 'text' | 'textarea' | 'select' | 'number' | 'password' | 'file';
  label: string;
  default: string | null;
  options: string[];
  placeholder: string | null;
  multiple: boolean;
}

interface ToolMetaDto {
  id: string;
  name: string;
  desc: string;
  group: string;
  params: ParamSpecDto[];
  needs_main_input: boolean;
  output_kind: string;  // "text" | highlight.js 语言名 | "svg"
}
```

`output_kind` 驱动前端渲染:`text` 纯文本;`json`/`sql`/`xml`/`yaml`/`toml`/`css` 经 highlight.js 高亮;`svg` 直接渲染。

## 错误模型

`CmdError(String)` 序列化为前端可读字符串,源自 `ToolError`(12 变体):

| 变体 | 触发 |
|---|---|
| `Utf8` | 字节→字符串失败 |
| `Base64` | base64 解码失败 |
| `Json` | serde_json 错误 |
| `Io` | IO 错误 |
| `Yaml` | serde_yaml 错误 |
| `Toml` | toml 错误 |
| `Csv` | csv 读写错误 |
| `Regex` | 正则编译错误 |
| `EmptyInput` | 输入为空 |
| `InvalidInput` | 参数非法 |
| `Parse` | 解析失败(IP/时间/cron/进制/枚举) |
| `Other` | 其他(加解密失败、引擎执行失败等) |
