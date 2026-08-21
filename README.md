# NexToolkit

> 开源、纯本地、轻量快捷的通用工具集。Tauri 桌面 GUI + 独立 CLI,跨平台,纯 Rust 核心。

## 为什么造

市面无单一本地开源工具能同时覆盖"开发者编码工具 + 全格式转换":编码工具箱(DevToys/CyberChef)强编码弱媒体,格式引擎(FFmpeg/Pandoc)强媒体弱编码;在线服务(freeconvert)以数据上云换一站式,代价是隐私。NexToolkit 用 Tauri+Rust 自研填补此缺口。

## 特性

- **纯本地** — 文件不离本机,无网络上报,无大小限制。
- **轻量** — CLI 单二进制 <8MB,GUI 便携包 <20MB,秒启动。
- **双入口** — CLI(脚本/CI 友好,headless)+ GUI(可视化)共享同一核心库,行为一致。
- **跨平台** — Windows / macOS / Linux。

## 工具矩阵(共 32 个)

| 分组 | 工具 |
|---|---|
| Encoders/Decoders | base64 · url · html · hex · jwt |
| Converters | json-yaml · json-toml · json-csv · md-html · numbase |
| Formatters | json-fmt · sql-fmt · xml-fmt · css-min |
| Generators | hash · hmac · uuid · password · lorem · qr |
| Text | case · sort-dedup · reverse · regex · diff |
| Crypto | aes-gcm · rsa · kdf |
| Net/Time | ipcalc · timestamp · cron · dns |

## 使用(CLI)

```bash
nextool encode base64 encode "Hello"          # SGVsbG8=
echo -n "Man" | nextool encode base64 encode  # TWFu(stdin)
nextool encode base64 decode "SGVsbG8="       # Hello
nextool generate hash sha256 abc
nextool crypto aes-encrypt --password pw "Secret"
nextool --help
```

## 使用(GUI)

下载 Release 的安装包/便携包,或本地启动:

```bash
npm install
npm run tauri dev    # 开发模式(热重载)
npm run tauri build  # 生产构建(产出安装包与便携包)
```

## 构建

```bash
cargo build -p nextool-cli --release   # CLI 单二进制(~4.5MB)
npm install && npm run tauri build     # GUI(需 Rust + Node)
cargo test --workspace                 # 测试(真实数据,无 mock)
cargo clippy --workspace -- -D warnings
```

## 文档

- [docs/PRD.md](docs/PRD.md) — 产品需求
- [docs/v0.1/PRD.md](docs/v0.1/PRD.md) — v0.1 版本需求
- [docs/v0.1/tech.md](docs/v0.1/tech.md) — 技术选型与设计依据
- [docs/research/](docs/research/) — 竞品调研与对标
- [docs/audit/](docs/audit/) — 纯净审计报告
- [CHANGELOG.md](CHANGELOG.md) — 更新日志

## 状态

v0.2.0:CLI + GUI 双入口可用,32 工具,180 测试通过,三平台 Release 产物已发布。重格式转换(音视频/Office/电子书)归后续引擎层。

## 协议

MIT
