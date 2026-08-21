# 产品需求

> 开源、纯本地、轻量快捷的通用工具集。Tauri 桌面 GUI + 独立 CLI,跨平台,纯 Rust 核心。

## 定位

市面无单一本地开源工具能同时覆盖"开发者编码工具 + 全格式转换":编码工具箱强编码弱媒体,格式引擎强媒体弱编码,在线服务以数据上云换一站式。NexToolkit 用 Tauri + Rust 自研填补此缺口,核心层纯 Rust 保证轻量,重格式转换归引擎层按需接入。

```mermaid
flowchart LR
  F["freeconvert<br/>上云 · 广度 + 一站式"] -.->|"无编码 / 数据上云"| NT
  C["DevToys / CyberChef<br/>编码强 · 弱媒体"] --> NT
  E["FFmpeg / Pandoc / Calibre<br/>媒体强 · 无编码"] -.->|"引擎层按需接入"| NT
  NT["NexToolkit<br/>纯本地 · 编码 + 轻量格式 · 双入口"]
```

**卖点:** 零上传(隐私)、无大小限制、无网络依赖、编码/加密能力。

## 工具矩阵(共 32,纯 Rust)

| 分组 | 工具 | 说明 |
|---|---|---|
| Encoders | base64 · url · html · hex · jwt | 编解码;jwt 解析 header/payload 不验签 |
| Converters | json-yaml · json-toml · json-csv · md-html · numbase | 结构化互转;numbase 进制转换 |
| Formatters | json-fmt · sql-fmt · xml-fmt · css-min | 美化与压缩 |
| Generators | hash · hmac · uuid · password · lorem · qr | hash 含 MD5/SHA1/SHA256/SHA512;uuid v4/v7;qr 生成 SVG |
| Text | case · sort-dedup · reverse · regex · diff | case 含 snake/camel/kebab/title;diff 文本逐行 |
| Crypto | aes-gcm · rsa · kdf | AES-GCM 加解密;RSA 密钥生成/加解密;kdf 含 PBKDF2/Argon2 |
| Net/Time | ipcalc · timestamp · cron · dns | IP/子网计算;时间戳互转;cron 下次触发;DNS 查询 |

## 验收

| 指标 | 目标 | 验证 |
|---|---|---|
| 跨平台三形态分发 | Win/macOS/Linux 各出 CLI 二进制、便携 GUI、安装版 | CI 三平台矩阵 |
| 双入口行为一致 | CLI 与 GUI 同输入同输出(共享 core) | 共享 core 集成测试 |
| CLI 冷启动 | <200ms | hyperfine 基准 |
| CLI 体积 | <8MB(release strip) | 资产测量 |
| 便携 GUI 体积 | <20MB(Win,skip WebView2) | 资产测量 |
| 测试 | core 单测 + CLI 集成测试全绿,真实数据无 mock | cargo test |

## 边界

- **Always**:提交前跑测试 + clippy;core 不引入 UI 依赖;新工具同步加 CLI 与 GUI 入口;中文注释。
- **Ask first**:新增非纯 Rust 依赖(外部引擎);改 workspace 结构;改 CI 矩阵。
- **Never**:提交密钥;放宽 CSP 或通配 capabilities;为通过测试删测试用例;core 依赖 UI 层;mock 测试。

## 非目标

- 不做在线/云转换(纯本地是核心卖点)。
- 不做移动端、账号/登录/遥测。
- 重引擎格式转换(音视频/Office↔PDF/电子书)不打包进核心,归引擎层按需接入(见 [todo.md](todo.md) 未来方向)。
