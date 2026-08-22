# 接口契约

> Tauri command 契约:前端经 `@tauri-apps/api/core` 的 `invoke` 调用。无 HTTP API(桌面工具集)。
> 命令在 `crates/tauri-app/src/commands.rs` 以 `#[tauri::command]` 声明,薄封装 `nextool-core`,共 38 个:
> 4 通用入口(`list_tools`/`run_tool`/`list_file_tools`/`list_engines`)+ 34 文件命令(归档 4 / 图像 7 / PDF 12 / 字体 2 / SVG 1 / XLSX 2 / 引擎 6)。
> 文本工具(107 个)不再各自独立 command,统一经 `run_tool(id, input, args)` 分发。
> 前端参数以 camelCase 传入(Tauri 默认),Rust 侧 snake_case 接收。

## 调用方式

```ts
import { invoke } from '@tauri-apps/api/core'
// 文本工具:经 run_tool 通用入口
const out = await invoke<string>('run_tool', {
  id: 'base64_encode',
  input: 'Hello',
  args: [] as [string, string][],
})
// 文件工具:独立 command
const path = await invoke<string>('archive_list', { path: '/tmp/a.zip' })
// 元数据:onMount 拉取动态渲染
const tools = await invoke<ToolMetaDto[]>('list_tools')
```

## 约定

- **成功**:返回字符串。
- **失败**:reject 并返回 `CmdError`(序列化为字符串),前端 `catch` 展示。
- **布尔/枚举参数**:经 `run_tool` 的 `args` 传入(字符串),Rust 侧 `ToolArgs` 解析。布尔用 `"true"`/`"false"`(select 控件);枚举(哈希算法/大小写模式/归档格式/图像格式)经 strum `FromStr` 解析,非法值返回 `Parse`。
- **主输入**:`needs_main_input=true` 的工具,主输入经 `run_tool` 的 `input` 参数(GUI 主 textarea);`needs_main_input=false` 的生成类工具(`uuid_v4`/`uuid_v7`/`password_generate`/`lorem_ipsum`/`rsa_keygen`/`http_probe`/`unit_convert`)无主输入,`input` 传空串即可。

## 通用入口

文本工具(107 个)统一经 `run_tool` 分发,不再各自独立 command。前端 `onMount` 调 `list_tools()` 拉取 `Vec<ToolMetaDto>` 动态渲染工具列表与参数表单,执行时调 `run_tool`。

```ts
interface ParamSpecDto {
  key: string
  kind: 'text' | 'textarea' | 'select' | 'number' | 'password' | 'bool' | 'file'
  label: string
  default: string | null
  options: string[]
  placeholder: string | null
  multiple: boolean
}
interface ToolMetaDto {
  id: string
  name: string
  desc: string
  group: string
  params: ParamSpecDto[]
  needs_main_input: boolean
  output_kind: string // "text" | highlight.js 语言名("json"/"sql"/"xml"/"yaml"/"toml"/"css") | "svg"
}
```

- `list_tools()`:返回全部文本工具元数据(107 项),前端按 `group` 分组渲染。
- `list_file_tools()`:返回 34 个文件工具元数据(与独立 command 对齐),复用同一渲染逻辑。
- `list_engines()`:返回 6 个引擎的运行时探测状态(`EngineStatusDto`,已装/未装),供前端提示。
- `run_tool(id, input, args)`:按 `id` 查 registry 分发,`args` 为 `Vec<(String,String)>`(前端传 `[[key,val],...]`)。未知 `id` 返回 `CmdError("未知工具: {id}")`。
- `output_kind`:前端据此决定渲染——`text` 纯文本;`json`/`sql`/`xml`/`yaml`/`toml`/`css` 经 highlight.js 高亮;`svg` 直接渲染为 SVG。

## 文本工具 id 清单

文本工具 id、参数 schema、`output_kind`、错误语义**以 `list_tools()` 返回为准**(107 项,分组:`encode` 26 / `convert` 25 / `format` 6 / `generate` 7 / `text` 15 / `crypto` 21 / `nettime` 6 / `http` 1 / `unit` 1)。前端 `onMount` 拉取元数据动态渲染,不维护静态清单,避免文档与代码漂移。

调用约定:`needs_main_input=true` 的工具主输入经 `input` 参数;`needs_main_input=false` 的生成类工具(`uuid_v4`/`uuid_v7`/`password_generate`/`lorem_ipsum`/`rsa_keygen`/`ed25519_keygen`/`http_probe`/`unit_convert`)无主输入,`input` 传空串。`args` 为 `Vec<(String,String)>`,布尔用 `"true"`/`"false"`,枚举经 strum `FromStr` 解析,非法值返回 `Parse`。`output_kind` 取值:`text` / highlight.js 语言名(`json`/`sql`/`xml`/`yaml`/`toml`/`css`)/ `svg`。

## 文件转换

字节域独立 command(接收文件路径,后端 `std::fs` 读写,返回路径或路径列表)。`format`/`targetFormat`/`target`/`direction`/`filter`/`parity` 等参数为 `String`,Rust 侧经 strum `FromStr` 解析为对应枚举,非法值返回 `CmdError`。

- **归档格式**(`ArchiveFormat`):`"zip"|"tar"|"targz"|"gz"|"7z"|"bz2"|"xz"|"zst"`(7z/bz2/xz/zst 支持解压,zip/tar/targz/gz/zst 支持压缩)。
- **图像格式**(`ImageFormat`):`"png"|"jpg"|"gif"|"bmp"|"webp"|"tiff"|"ico"|"dds"|"ff"|"hdr"|"exr"|"pnm"|"qoi"|"tga"`(14 格式,DDS 仅解码)。
- **字体格式**(`FontFormat`):`"ttf"|"woff"`。
- **SVG 格式**(`SvgFormat`):`"png"|"jpg"`。

### 归档(4)

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `archive_list` | `path: string` | 文件列表(每行 `路径\t大小`) | `Io`/`InvalidInput`(格式不识别) |
| `archive_extract` | `path: string`, `outputDir?: string` | 写出文件路径列表(`string[]`) | `Io`/`InvalidInput`(路径遍历/格式) |
| `archive_compress` | `paths: string[]`, `format: string`, `output?: string` | 产物路径 | `InvalidInput`(未知格式/GZ 多文件)/`Io` |
| `archive_convert` | `path: string`, `targetFormat: string`, `output?: string` | 产物路径 | `InvalidInput`/`Io` |

### 图像(7)

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `image_convert` | `path`, `target: string`, `output?: string` | 产物路径 | `InvalidInput`(未知格式/解码失败)/`Other`(编码失败)/`Io` |
| `image_resize` | `path`, `width: u32`, `height: u32`, `output?: string` | 产物路径 | `InvalidInput`(宽高同 0/解码失败)/`Io` |
| `image_crop` | `path`, `x: u32`, `y: u32`, `width: u32`, `height: u32`, `output?: string` | 产物路径 | `InvalidInput`/`Io` |
| `image_flip` | `path`, `direction: string`(`h`/`v`), `output?: string` | 产物路径 | `InvalidInput`(未知方向)/`Io` |
| `image_filter` | `path`, `filter: string`(`grayscale`/`invert`/`sepia`/`blur`), `output?: string` | 产物路径 | `InvalidInput`(未知滤镜)/`Io` |
| `image_adjust` | `path`, `brightness: i32`, `contrast: f32`, `output?: string` | 产物路径 | `Io` |
| `image_compress_jpeg` | `path`, `quality: u8`, `output?: string` | 产物路径(JPEG) | `InvalidInput`(quality>100)/`Io` |

### PDF(12)

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `pdf_split` | `path`, `outputDir?: string` | 拆分件路径列表(`string[]`) | `InvalidInput`(格式)/`Io` |
| `pdf_split_ranges` | `path`, `ranges: string`(`"1-3,5,7-10"`), `outputDir?: string` | 拆分件路径列表(`string[]`) | `InvalidInput`(范围/格式)/`Io` |
| `pdf_split_every_n` | `path`, `n: u32`, `outputDir?: string` | 拆分件路径列表(`string[]`) | `InvalidInput`(n=0/格式)/`Io` |
| `pdf_split_parity` | `path`, `parity: string`(`odd`/`even`), `outputDir?: string` | 拆分件路径列表(`string[]`) | `InvalidInput`(parity/格式)/`Io` |
| `pdf_rotate` | `path`, `degrees: u32`(`90`/`180`/`270`), `output?: string` | 产物路径 | `InvalidInput`(格式)/`Other`/`Io` |
| `pdf_encrypt` | `path`, `password: string`, `output?: string` | 产物路径 | `InvalidInput`(空口令/格式)/`Other`/`Io` |
| `pdf_decrypt` | `path`, `password: string`, `output?: string` | 产物路径 | `InvalidInput`(口令错/未加密/格式)/`Io` |
| `pdf_merge` | `paths: string[]`, `output?: string` | 产物路径 | `InvalidInput`(空列表/格式)/`Io` |
| `pdf_delete_pages` | `path`, `pages: string`(`"1,3,5-7"`), `output?: string` | 产物路径 | `InvalidInput`(页码/格式)/`Io` |
| `pdf_extract_pages` | `path`, `pages: string`(`"1,3,5-7"`), `output?: string` | 产物路径 | `InvalidInput`(页码/格式)/`Io` |
| `pdf_set_metadata` | `path`, `title?/author?/subject?/keywords?: string`, `output?: string` | 产物路径 | `InvalidInput`(格式)/`Io` |
| `pdf_add_page_numbers` | `path`, `output?: string` | 产物路径 | `InvalidInput`(格式)/`Io` |

### 字体(2)

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `font_convert` | `path`, `target: string`(`ttf`/`woff`), `output?: string` | 产物路径 | `InvalidInput`(未知格式/解码失败)/`Io` |
| `font_meta` | `path: string` | 元数据文本(名称/版权/字重等) | `InvalidInput`(格式)/`Io` |

### SVG(1)

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `svg_convert` | `path`, `target: string`(`png`/`jpg`), `output?: string` | 产物路径(栅格化) | `InvalidInput`(未知格式/SVG 解析失败)/`Io` |

### XLSX(2)

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `xlsx_to_json` | `input: string`, `output?: string` | 产物路径(JSON) | `InvalidInput`(格式)/`Io` |
| `json_to_xlsx` | `input: string`, `output?: string` | 产物路径(XLSX) | `InvalidInput`(格式)/`Io` |

### 引擎(6,运行时探测系统已装)

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `av_convert` | `input`, `to: string`, `output?: string` | 产物路径 | `Other`(ffmpeg 未装/失败) |
| `office_to_pdf` | `input: string` | 产物路径(PDF) | `Other`(LibreOffice 未装/失败) |
| `ebook_convert` | `input`, `to: string`, `output?: string` | 产物路径 | `Other`(calibre 未装/失败) |
| `markup_convert` | `input`, `to: string`, `output?: string` | 产物路径 | `Other`(pandoc 未装/失败) |
| `pdf_compress` | `input: string`, `output?: string` | 产物路径(PDF) | `Other`(ghostscript 未装/失败) |
| `ocr` | `input: string` | 产物路径(TXT) | `Other`(tesseract 未装/失败) |

产物默认落源文件所在目录:解压/拆分到 `{stem}_extracted/`,压缩/转换到 `{stem}.{ext}`,碰撞追加 `_converted`→`(1)`→`(2)`(`create_new` 原子检查,不覆盖)。`output`/`outputDir` 省略时用默认。图像缩放 `width`/`height` 一维为 0 时按另一维等比;产物同源格式。PDF 加密用 AES(owner=user 同口令)。引擎命令统一经 `engine_convert_file` 落盘,未装引擎返回明确错误提示安装。



`CmdError(String)` 序列化为前端可读字符串,源自 `nextool_core::ToolError`(12 变体,`thiserror` 派生):

| 变体 | 触发 |
|---|---|
| `Utf8` | 字节→字符串失败(hex/base64 解码后非 UTF-8) |
| `Base64` | base64 解码失败 |
| `Json` | serde_json 错误 |
| `Io` | IO/DNS resolver 错误 |
| `Yaml` | serde_yaml 序列化/反序列化错误 |
| `Toml` | toml 反序列化错误 |
| `Csv` | csv 读写错误 |
| `Regex` | 正则编译错误(非法正则) |
| `EmptyInput` | 输入为空(sql/qr) |
| `InvalidInput` | 参数非法(未知算法/模式/salt 过短/位数不足/非数组) |
| `Parse` | 解析失败(IP/时间/cron/进制/strum 枚举) |
| `Other` | 其他(加解密失败、XML 解析、HMAC 初始化等) |
