# 接口契约

> Tauri command 契约:前端经 `@tauri-apps/api/core` 的 `invoke` 调用。无 HTTP API(桌面工具集)。
> 命令在 `crates/tauri-app/src/commands.rs` 以 `#[tauri::command]` 声明,薄封装 `nextool-core`,共 13 个:
> 3 通用入口(`list_tools`/`run_tool`/`list_file_tools`)+ 10 文件命令(归档/图像/PDF)。
> 文本工具(54 个)不再各自独立 command,统一经 `run_tool(id, input, args)` 分发。
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

文本工具(54 个)统一经 `run_tool` 分发,不再各自独立 command。前端 `onMount` 调 `list_tools()` 拉取 `Vec<ToolMetaDto>` 动态渲染工具列表与参数表单,执行时调 `run_tool`。

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

- `list_tools()`:返回全部文本工具元数据(54 项),前端按 `group` 分组渲染。
- `list_file_tools()`:返回 10 个文件工具元数据(与独立 command 对齐),复用同一渲染逻辑。
- `run_tool(id, input, args)`:按 `id` 查 registry 分发,`args` 为 `Vec<(String,String)>`(前端传 `[[key,val],...]`)。未知 `id` 返回 `CmdError("未知工具: {id}")`。
- `output_kind`:前端据此决定渲染——`text` 纯文本;`json`/`sql`/`xml`/`yaml`/`toml`/`css` 经 highlight.js 高亮;`svg` 直接渲染为 SVG。

## 文本工具 id 清单

下表按 `group` 列出全部 54 个文本工具 id。`主` 列标记是否需要主输入(`input`);`args` 列列出参数 key(类型);`输出` 列为 `output_kind`;错误语义列于备注。参数完整 schema(label/default/options/placeholder)以 `list_tools()` 返回为准。

### encode(10)

| id | 主 | args | 输出 | 错误 |
|---|---|---|---|---|
| `base64_encode` | ✓ | — | text | — |
| `base64_decode` | ✓ | — | text | `Base64`/`Utf8` |
| `url_encode` | ✓ | — | text | — |
| `url_decode` | ✓ | — | text | `Parse` |
| `html_encode` | ✓ | — | text | — |
| `html_decode` | ✓ | — | text | — |
| `hex_encode` | ✓ | — | text | — |
| `hex_decode` | ✓ | — | text | `Parse`/`Utf8` |
| `jwt_decode` | ✓ | — | json | `InvalidInput`(格式)/`Base64`/`Json` |
| `jwt_verify` | ✓ | `key`(text) | json | `InvalidInput`(格式/alg 不支持/验签失败)/`Other` |

### convert(9)

| id | 主 | args | 输出 | 错误 |
|---|---|---|---|---|
| `json_to_yaml` | ✓ | — | yaml | `Json`/`Yaml` |
| `yaml_to_json` | ✓ | — | json | `Yaml`/`Json` |
| `json_to_toml` | ✓ | — | toml | `Json`/`InvalidInput`(null)/`Toml` |
| `toml_to_json` | ✓ | — | json | `Toml`/`Json` |
| `json_to_csv` | ✓ | — | text | `Json`/`InvalidInput`(非数组/空/非对象)/`Csv` |
| `csv_to_json` | ✓ | — | json | `Csv`/`Json` |
| `md_to_html` | ✓ | — | xml | — |
| `numbase_convert` | ✓ | `from`(number), `to`(number) | text | `InvalidInput`(进制 ∉ 2..36 / 解析失败) |
| `unit_convert` | ✗ | `value`(number), `from`(text), `to`(text) | text | `InvalidInput`(未知单位/跨类) |

### format(6)

| id | 主 | args | 输出 | 错误 |
|---|---|---|---|---|
| `json_format` | ✓ | — | json | `Json` |
| `json_minify` | ✓ | — | json | `Json` |
| `sql_format` | ✓ | — | sql | `EmptyInput` |
| `xml_format` | ✓ | — | xml | `Other`(标签不匹配等) |
| `xml_minify` | ✓ | — | xml | `Other` |
| `css_minify` | ✓ | — | css | `Other` |

### generate(7)

| id | 主 | args | 输出 | 错误 |
|---|---|---|---|---|
| `uuid_v4` | ✗ | — | text | — |
| `uuid_v7` | ✗ | — | text | — |
| `hash` | ✓ | `algo`(select: md5/sha1/sha256/sha512) | text | `Parse`(未知算法) |
| `hmac_compute` | ✓ | `algo`(select), `key`(text) | text | `Parse`(未知算法) |
| `password_generate` | ✗ | `length`(number), `upper`/`lower`/`digits`/`symbols`(select: true/false) | text | `InvalidInput`(长度<1) |
| `lorem_ipsum` | ✗ | `paragraphs`(number) | text | `InvalidInput`(段数为 0) |
| `qr_svg` | ✓ | — | svg | `EmptyInput`/`Other`(数据过长) |

> `password_generate` 默认规则下沉 core:`PasswordOpts::default()`(大写+小写+数字,不含符号);四项全 `false` 时用默认字符集,CLI/GUI 行为统一。

### text(7)

| id | 主 | args | 输出 | 错误 |
|---|---|---|---|---|
| `case_convert` | ✓ | `mode`(select: upper/lower/title/snake/camel/kebab) | text | `Parse`(未知模式) |
| `sort_lines` | ✓ | — | text | — |
| `dedup_lines` | ✓ | — | text | — |
| `reverse_text` | ✓ | — | text | — |
| `regex_match` | ✓ | `pattern`(text) | text | `Regex`(非法正则) |
| `regex_replace` | ✓ | `pattern`(text), `replacement`(text) | text | `Regex` |
| `diff_text` | ✓ | `other`(textarea) | text | — |

### crypto(9)

| id | 主 | args | 输出 | 错误 |
|---|---|---|---|---|
| `aes_gcm_encrypt` | ✓ | `password`(password) | text | `Other` |
| `aes_gcm_decrypt` | ✓ | `password`(password) | text | `Base64`/`InvalidInput`(口令错或数据损坏) |
| `rsa_keygen` | ✗ | `bits`(number, 默认 2048) | text | `InvalidInput`(<2048)/`Other` |
| `rsa_encrypt` | ✓ | `pubPem`(textarea) | text | `InvalidInput`(无效 PEM)/`Other` |
| `rsa_decrypt` | ✓ | `privPem`(textarea) | text | `Base64`/`InvalidInput`/`Other` |
| `rsa_sign` | ✓ | `privPem`(textarea) | text | `InvalidInput`(无效 PEM)/`Other` |
| `rsa_verify` | ✓ | `pubPem`(textarea), `signature`(text) | text | `InvalidInput`(无效 PEM/签名/验签失败) |
| `pbkdf2` | ✓ | `salt`(text), `iterations`(number, 默认 100000) | text | — |
| `argon2` | ✓ | `salt`(text, ≥8 字节) | text | `InvalidInput`(salt <8 字节)/`Other` |

> `rsa_verify` 成功返回 `"验签成功"`。`rsa_sign`/`rsa_verify` 用 PKCS1v15/SHA256;加解密用 OAEP-SHA256。

### nettime(6)

| id | 主 | args | 输出 | 错误 |
|---|---|---|---|---|
| `ipcalc` | ✓ | — | text | `Parse`(非法 CIDR) |
| `timestamp_to_human` | ✓ | `tz`(text, 默认 UTC) | text | `Parse`(非整数 / 无效时区 / 超范围) |
| `timestamp_from_human` | ✓ | `tz`(text, 默认 UTC) | text | `Parse`(时间格式 / 时区 / 夏令时歧义) |
| `cron_next` | ✓ | `count`(number, 默认 3) | text | `InvalidInput`(count=0)/`Parse`(非法 cron) |
| `dns_lookup` | ✓ | `rtype`(select: A/AAAA/MX/TXT) | text | `InvalidInput`(未知类型)/`Io`/`Other` |
| `http_probe` | ✗ | `url`(text) | text | `InvalidInput`(URL 非法)/`Other`(请求失败) |

## 文件转换

字节域独立 command(接收文件路径,后端 `std::fs` 读写,返回路径或路径列表)。`format`/`targetFormat`/`target` 参数为 `String`,Rust 侧经 strum `FromStr` 解析为 `ArchiveFormat`/`ImageFormat` 枚举,非法值返回 `CmdError`。归档格式:`"zip"|"tar"|"targz"|"gz"|"7z"`(7z 仅解压);图像格式:`"png"|"jpg"|"gif"|"bmp"|"webp"|"tiff"|"ico"`。

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `archive_list` | `path: string` | 文件列表(每行 `路径\t大小`) | `Io`/`InvalidInput`(格式不识别) |
| `archive_extract` | `path: string`, `outputDir?: string` | 写出文件路径列表(`string[]`) | `Io`/`InvalidInput`(路径遍历/格式) |
| `archive_compress` | `paths: string[]`, `format: string`, `output?: string` | 产物路径 | `InvalidInput`(未知格式/GZ 多文件)/`Io` |
| `archive_convert` | `path: string`, `targetFormat: string`, `output?: string` | 产物路径 | `InvalidInput`/`Io` |
| `image_convert` | `path: string`, `target: string`, `output?: string` | 产物路径 | `InvalidInput`(未知格式/解码失败)/`Other`(编码失败)/`Io` |
| `image_resize` | `path: string`, `width: u32`, `height: u32`, `output?: string` | 产物路径 | `InvalidInput`(宽高同 0/解码失败)/`Io` |
| `pdf_split` | `path: string`, `outputDir?: string` | 拆分件路径列表(`string[]`) | `InvalidInput`(格式)/`Io` |
| `pdf_rotate` | `path: string`, `output?: string` | 产物路径 | `InvalidInput`(格式)/`Other`/`Io` |
| `pdf_encrypt` | `path: string`, `password: string`, `output?: string` | 产物路径 | `InvalidInput`(空口令/格式)/`Other`/`Io` |
| `pdf_decrypt` | `path: string`, `password: string`, `output?: string` | 产物路径 | `InvalidInput`(口令错/未加密/格式)/`Io` |

产物默认落源文件所在目录:解压/拆分到 `{stem}_extracted/`,压缩/转换到 `{stem}.{ext}`,碰撞追加 `_converted`→`(1)`→`(2)`(`create_new` 原子检查,不覆盖)。`output`/`outputDir` 省略时用默认。图像缩放 `width`/`height` 一维为 0 时按另一维等比;产物同源格式。PDF 旋转所有页顺时针 90°;加密用 AES(owner=user 同口令)。



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
