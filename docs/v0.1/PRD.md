# NexToolkit v0.1 — 版本级 PRD

> 版本级需求细则。产品级见 `docs/PRD.md`;技术选型/结构/命令/代码风格见 `docs/v0.1/tech.md`。

## 版本目标

v0.1 交付完整可用的核心层:覆盖开发者高频四类工具(编码/转换、正则/格式化/Diff、加密/安全、网络/IP/时间),纯 Rust 实现,CLI 与 GUI 双入口行为一致。重引擎格式转换(音视频/Office/电子书)归后续版本,不在本版。

## 工具清单(纯 Rust,共 32 个)

| 分组 | 工具 | 说明 |
|---|---|---|
| Encoders/Decoders | base64、url、html、hex、jwt | 编解码;jwt 解析 header/payload 不验签 |
| Converters | json-yaml、json-toml、json-csv、md-html、numbase | 结构化互转;numbase 进制转换 |
| Formatters | json-fmt、sql-fmt、xml-fmt、css-min | 美化与压缩 |
| Generators | hash、hmac、uuid、password、lorem、qr | hash 含 MD5/SHA1/SHA256/SHA512;uuid v4/v7;qr 生成 SVG |
| Text | case、sort-dedup、reverse、regex、diff | case 含 snake/camel/kebab/title;diff 文本逐行 |
| Crypto | aes-gcm、rsa、kdf | AES-GCM 加解密;RSA 密钥生成/加解密;kdf 含 PBKDF2/Argon2 |
| Net/Time | ipcalc、timestamp、cron、dns | IP/子网计算;时间戳互转;cron 解析下次触发;DNS 查询 |

## 用户故事与验收(代表性,Given/When/Then)

### base64

- Given 输入 `Hello`,When `nextool encode base64 encode "Hello"`,Then 输出 `SGVsbG8=`。
- Given 输入 `SGVsbG8=`,When `nextool encode base64 decode`,Then 输出 `Hello`。
- Given 非法 Base64 `!!!!`,When decode,Then 非零退出码 + 错误信息到 stderr。

### json-yaml

- Given JSON `{"a":1,"b":[2,3]}`,When `nextool convert json-yaml to`,Then 输出合法 YAML。
- Given 非法 JSON,When 转换,Then 非零退出码 + 指出错误位置。

### regex

- Given 模式 `\d+` 与文本 `a12b3`,When `nextool text regex-match`,Then 输出 `12`、`3` 各一行。
- Given 模式 `\d+` 与替换 `#$0`,When `nextool text regex-replace`,Then 输出 `a#12b#3`。

### diff

- Given 两段文本,When `nextool text diff`,Then 输出 unified diff,新增 `+`、删除 `-`。

### aes-gcm

- Given 明文与口令,When `nextool crypto aes-encrypt --password <口令>`,Then 输出密文(含 nonce);When `aes-decrypt` 同口令,Then 还原明文。
- Given 错误口令,When decrypt,Then 非零退出码(认证失败)。

### ipcalc

- Given `192.168.1.5/24`,When `nextool net-time ipcalc`,Then 输出网络地址、广播、掩码、可用主机范围。

### timestamp

- Given Unix 时间戳 `1700000000`,When `nextool net-time ts-to-human 1700000000 --tz Asia/Shanghai`,Then 输出对应时区可读时间。

### 通用验收(全部工具)

- 每个工具支持 `--help`,输出用法与示例。
- 输入可来自参数或 stdin(管道友好)。
- 成功退出码 0,失败非零 + stderr 错误。
- CLI 与 GUI 同输入同输出(共享 `nextool-core`)。

## 交互设计(GUI)

- 左侧分组导航(七组),右侧工具面板;每个工具声明参数表单 schema,前端渲染。
- 统一输入区(文本框 / 文件拖拽 / 图片预览),统一输出区(可复制 / 下载)。
- 首屏显式承诺"100% 本地,文件不离本机"(对标 freeconvert 上云的差异化)。
- 常用转换快捷卡片(base64、hash、json-fmt、jwt 等)。
- i18n:中英双语,locale 文件结构预留扩展。

## 规则

1. **core 纯逻辑**:工具实现不依赖 Tauri/clap/stdin/stdout;输入输出为 `String`/`Vec<u8>`/结构体,错误用 `thiserror`。
2. **CLI 薄**:clap 解析后直接调 core,输出格式化归 CLI 层。
3. **GUI 薄**:`#[tauri::command]` 调 core,前端只渲染。
4. **真实数据测试**:禁止 mock;core 单测用真实输入输出;CLI 集成测试真起二进制。
5. **依赖纯 Rust**:TLS 用 rustls;不引入 ffmpeg/LibreOffice 等外部引擎。
6. **中文注释**:文件头 + 关键函数注释。

## 测试策略

- core:每工具 `#[cfg(test)]` 单测,≥3 用例(正常/边界/错误)。优先用最高 seam(纯逻辑单测),全仓 seam 最少。
- CLI:`test/cli/` 用 `assert_cmd` 真起 `nextool-cli`,覆盖子命令路由与 stdin/退出码。
- GUI:v0.1 仅冒烟(每个工具可调通),后续版本扩 E2E。
- 覆盖期望:core 单测行覆盖 >85%。

## 分发(v0.1)

- CLI 单二进制:三平台 release artifact。
- 便携 GUI:Windows exe(skip WebView2)、Linux AppImage、macOS .app。
- 安装版:Windows msi/nsis、Linux deb、macOS dmg。
- CI:GitHub Actions 三平台矩阵(`tauri-action`)。

## 边界(继承产品级,补充 v0.1)

- Never:v0.1 不引入外部引擎;不为赶进度跳过测试。
- Always:每个工具 core 单测与 CLI 子命令同步交付;提交前 `cargo test --workspace` 全绿。

## 非目标(继承产品级)

重引擎格式转换、Smart Detection、Recipe 流水线、移动端、账号/遥测、单位/时区换算、字体转换。
