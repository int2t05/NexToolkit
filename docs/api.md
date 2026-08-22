# 接口契约

> Tauri command 契约:前端经 `@tauri-apps/api/core` 的 `invoke` 调用。无 HTTP API(桌面工具集)。
> 命令在 `crates/tauri-app/src/commands.rs` 以 `#[tauri::command]` 声明,薄封装 `nextool-core`,共 49 个。
> 前端参数以 camelCase 传入(Tauri 默认),Rust 侧 snake_case 接收。

## 调用方式

```ts
import { invoke } from '@tauri-apps/api/core'
const out = await invoke<string>('base64_encode', { input: 'Hello' })
```

## 约定

- **成功**:返回字符串。
- **失败**:reject 并返回 `CmdError`(序列化为字符串),前端 `catch` 展示。
- **布尔参数**:前端传字符串 `"true"`/`"false"`(select 控件),Rust 侧 `parse_bool` 解析。
- **枚举参数**:哈希算法 `"md5"|"sha1"|"sha256"|"sha512"`;大小写模式 `"upper"|"lower"|"title"|"snake"|"camel"|"kebab"`;DNS 类型 `"A"|"AAAA"|"MX"|"TXT"`。
- **主输入**:文本类工具的主输入为 `input` 参数(GUI 主 textarea);生成类工具(`uuid_v4`/`uuid_v7`/`password_generate`/`lorem_ipsum`/`rsa_keygen`)无主输入。

## 编解码

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `base64_encode` | `input: string` | base64 字符串 | — |
| `base64_decode` | `input: string` | 解码字符串 | `Base64`/`Utf8` |
| `url_encode` | `input: string` | percent-encoded | — |
| `url_decode` | `input: string` | 解码字符串 | `Parse` |
| `html_encode` | `input: string` | HTML 实体 | — |
| `html_decode` | `input: string` | 解码字符串 | — |
| `hex_encode` | `input: string` | 十六进制 | — |
| `hex_decode` | `input: string` | 解码字符串 | `Parse`/`Utf8` |
| `jwt_decode` | `input: string` | header/payload pretty JSON | `InvalidInput`/`Base64`/`Json` |
| `jwt_verify` | `input: string`, `key: string` | payload pretty JSON | `InvalidInput`(格式/alg 不支持/验签失败)/`Other` |

## 转换

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `json_to_yaml` | `input: string` | YAML | `Json`/`Other` |
| `yaml_to_json` | `input: string` | pretty JSON | `Other`/`Json` |
| `json_to_toml` | `input: string` | TOML | `Json`/`InvalidInput`(null)/`Other` |
| `toml_to_json` | `input: string` | pretty JSON | `Other`/`Json` |
| `json_to_csv` | `input: string` | CSV | `Json`/`InvalidInput`(非数组/空/非对象)/`Other` |
| `csv_to_json` | `input: string` | JSON 数组 | `Other`/`Json` |
| `md_to_html` | `input: string` | HTML | — |
| `numbase_convert` | `input: string`, `from: u32`, `to: u32` | 数字字符串 | `InvalidInput`(进制 ∉ 2..36 / 解析失败) |
| `unit_convert` | `value: f64`, `from: string`, `to: string` | 数字字符串 | `InvalidInput`(未知单位/跨类) |

## 格式化

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `json_format` | `input: string` | 美化 JSON(2 空格) | `Json` |
| `json_minify` | `input: string` | 压缩 JSON | `Json` |
| `sql_format` | `input: string` | 关键字大写 SQL | `EmptyInput` |
| `xml_format` | `input: string` | 美化 XML | `Other`(标签不匹配等) |
| `xml_minify` | `input: string` | 压缩 XML | `Other` |
| `css_minify` | `input: string` | 压缩 CSS | `Other` |

## 生成器

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `uuid_v4` | — | UUID v4 | — |
| `uuid_v7` | — | UUID v7 | — |
| `hash` | `input: string`, `algo: string` | 十六进制摘要 | `InvalidInput`(未知算法) |
| `hmac_compute` | `input: string`, `algo: string`, `key: string` | 十六进制 HMAC | `InvalidInput`(未知算法) |
| `password_generate` | `length: usize`, `upper/lower/digits/symbols: string` | 密码 | `InvalidInput`(长度<1 / 字符集全 false) |
| `lorem_ipsum` | `paragraphs: usize` | Lorem 文本 | `InvalidInput`(段数为 0) |
| `qr_svg` | `input: string` | SVG 字符串 | `EmptyInput`/`Other`(数据过长) |

## 文本

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `case_convert` | `input: string`, `mode: string` | 转换字符串 | `InvalidInput`(未知模式) |
| `sort_lines` | `input: string` | 排序文本 | — |
| `dedup_lines` | `input: string` | 去重保序 | — |
| `reverse_text` | `input: string` | 反转文本 | — |
| `regex_match` | `input: string`, `pattern: string` | 匹配行 | `Parse`(非法正则) |
| `regex_replace` | `input: string`, `pattern: string`, `replacement: string` | 替换文本 | `Parse` |
| `diff_text` | `input: string`, `other: string` | unified diff | — |

## 加密

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `aes_gcm_encrypt` | `input: string`, `password: string` | base64 密文 | `Other` |
| `aes_gcm_decrypt` | `input: string`, `password: string` | 明文 | `Base64`/`InvalidInput`(口令错或数据损坏) |
| `rsa_keygen` | `bits: usize` | PEM 密钥对 | `InvalidInput`(<2048)/`Other` |
| `rsa_encrypt` | `input: string`, `pub_pem: string` | base64 密文 | `InvalidInput`(无效 PEM)/`Other` |
| `rsa_decrypt` | `input: string`, `priv_pem: string` | 明文 | `Base64`/`InvalidInput`/`Other` |
| `rsa_sign` | `input: string`, `priv_pem: string` | base64 签名 | `InvalidInput`(无效 PEM)/`Other` |
| `rsa_verify` | `input: string`, `pub_pem: string`, `signature: string` | "签名验证通过" | `InvalidInput`(无效 PEM/签名/验签失败) |
| `pbkdf2` | `input: string`, `salt: string`, `iterations: u32` | 十六进制派生密钥(32B) | — |
| `argon2` | `input: string`, `salt: string` | 十六进制哈希(32B) | `InvalidInput`(salt <8 字节)/`Other` |

## 网络/时间

| 命令 | 参数 | 返回 | 错误 |
|---|---|---|---|
| `ipcalc` | `input: string` | 地址/网络/广播/掩码/主机范围/主机数 | `Parse`(非法 CIDR) |
| `timestamp_to_human` | `input: string`, `tz: string` | RFC3339 可读时间 | `Parse`(非整数 / 无效时区 / 超范围) |
| `timestamp_from_human` | `input: string`, `tz: string` | Unix 秒字符串 | `Parse`(时间格式 / 时区 / 夏令时歧义) |
| `cron_next` | `input: string`, `count: usize` | 下次触发列表(每行一个 RFC3339) | `InvalidInput`(count=0)/`Parse`(非法 cron) |
| `dns_lookup` | `input: string`, `rtype: string` | DNS 记录(每行一条) | `InvalidInput`(未知类型)/`Io`/`Other` |
| `http_probe` | `url: string` | 状态码/最终 URL/响应头多行 | `InvalidInput`(URL 非法)/`Other`(请求失败) |

## 文件转换

字节域命令(command 接收文件路径,后端 `std::fs` 读写,返回路径或路径列表)。归档格式枚举:`"zip"|"tar"|"targz"|"gz"|"7z"`(7z 仅解压);图像格式:`"png"|"jpg"|"gif"|"bmp"|"webp"|"tiff"|"ico"`。

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



`CmdError(String)` 序列化为前端可读字符串,源自 `nextool_core::ToolError`:

| 变体 | 触发 |
|---|---|
| `Utf8` | 字节→字符串失败(hex/base64 解码后非 UTF-8) |
| `Base64` | base64 解码失败 |
| `Json` | serde_json 错误 |
| `Io` | IO/DNS resolver 错误 |
| `EmptyInput` | 输入为空(sql/qr) |
| `InvalidInput` | 参数非法(未知算法/模式/salt 过短/位数不足/非数组) |
| `Parse` | 解析失败(YAML/TOML/CSV/正则/IP/时间/cron) |
| `Other` | 其他(YAML/TOML 序列化、加解密失败、XML 解析等) |
