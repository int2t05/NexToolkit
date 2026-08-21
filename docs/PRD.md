# NexToolkit — 产品需求文档

> 开源、纯本地、轻量快捷的通用工具集。本文为产品级共享真相源,只定"做什么 + 验收什么";
> 交互细节、技术选型、阈值下沉至 `docs/v0.1/PRD.md` 与 `docs/v0.1/tech.md`。

## 目标

市面无单一本地开源工具能同时覆盖"开发者编码工具 + 全格式转换":编码工具箱(DevToys/CyberChef)强编码弱媒体,格式引擎(FFmpeg/Pandoc)强媒体弱编码;在线服务(freeconvert)以数据上云换一站式,代价是隐私(依据见 `docs/research/2026-08-21-local-oss-toolbox.md` §1/§6)。NexToolkit 用 Tauri+Rust 自研填补此缺口,核心层纯 Rust 保证轻量,重格式转换归后续引擎层,卖点为**零上传、无大小限制、无网络依赖、编码/加密能力**。

**用户故事:**

- 开发者:本地一键完成 Base64/Hash/JWT/正则等高频操作,文件不离本机。
- 终端用户:`nextool <工具> <参数>` 在脚本/CI 调用同一套工具,无 GUI 与 WebView 依赖。
- 隐私敏感用户:所有处理纯本地,无任何网络上报。

## 验收标准

| 指标 | 目标 | 验证方式 |
|---|---|---|
| 跨平台三形态分发 | Win/macOS/Linux 均出 CLI 单二进制、便携 GUI、安装版 | 三平台 CI 矩阵产物 |
| 双入口行为一致 | CLI 与 GUI 同输入同输出(共享 `nextool-core`) | 共享 core 的集成测试 |
| CLI 冷启动 | <200ms(空参数 help) | `hyperfine 'nextool --help'` |
| GUI 冷启动 | <1s(首屏可交互) | 三平台人工计时 + CI 冒烟 |
| CLI 二进制体积 | <8MB(release,strip) | release 资产体积测量 |
| 便携 GUI 包体积 | <20MB(Win,skip WebView2) | zip 体积测量 |
| 测试 | core 单测 + CLI 集成测试全绿,真实数据无 mock | `cargo test --workspace` |

## 边界

- **Always**:提交前跑 `cargo test --workspace` + `cargo clippy`;core 不引入 Tauri/clap 依赖;新工具同步加 CLI 与 GUI 入口;中文注释。
- **Ask first**:新增非纯 Rust 依赖(外部引擎);改 workspace 结构;改 CI 矩阵;引入非 `skip` 的 WebView2 安装模式。
- **Never**:提交密钥;放宽 CSP 或通配 capabilities;为通过测试删测试用例;core 依赖 UI 层;mock 测试。

## 非目标

- 不做在线/云转换(纯本地是核心卖点)。
- v0.1 不做重引擎格式转换(音视频/Office↔PDF/电子书)——归引擎层后续。
- v0.1 不做 Smart Detection(剪贴板自动选工具)、Recipe 流水线(操作链)——远期路线图。
- 不做移动端、账号/登录/遥测。
- 不做单位/时区换算、字体转换(与定位无关,依据见调研 §6)。

## 待决问题

1. 引擎层(重格式转换)引入时机与分发方式:独立可选包 vs 运行时按需下载?(后续版本启动时定)
2. macOS 是否申请 Apple Developer 签名公证($99/年),还是开源初期不签名 + 文档告知右键打开?(首版 macOS 分发前定)
3. 自动更新(tauri-updater)是否启用?(开源工具集首版可不做)
