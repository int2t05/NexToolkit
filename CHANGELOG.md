# 更新日志

本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。版本演进归此处,正文只陈述当前事实。

## [Unreleased]

## [0.2.0] — 2026-08-21

### 新增

- Tauri 2 + Svelte 5 桌面 GUI:33 工具全暴露,左侧七组导航 + 搜索 + 参数表单 + 输出/复制,中英双语,暗色主题。
- 前端 51KB(gzip 19.6KB),`@tauri-apps/api` 官方 invoke,runes 模式。
- CI:三平台(Ubuntu/Windows/macOS)core-cli 测试 + 前端 check + Tauri 编译验证;tag 触发 release 三平台产物上传。
- 设计参考:爬取 it-tools + tauri2-svelte5-shadcn/OpenCovibe/comine 本地分析,设计决策标注依据。
- 纯净审计:GUI 代码对照 A-F 表,阻断 1 + 清理 4 全修复(docs/audit/2026-08-21-gui.md)。

### 变更

- Windows 工具链由 gnu 切 MSVC(gnu rustc 编译 windows-sys 0.59 栈溢出 ICE)。
- 前端包管理用 npm(pnpm 在本机 V8 OOM,npm 稳定)。

### 验证

- CI 全绿:frontend + core-cli(clippy+test)+ tauri-build 三平台编译。
- core 168 + cli 12 = 180 测试通过;clippy `-D warnings` 零警告。

## [0.1.0] — 2026-08-21

## [0.1.0] — 2026-08-21

### 新增

- 核心库 `nextool-core`:33 工具,7 模块(encode/convert/format/generate/text/crypto/nettime),纯 Rust,无 UI 依赖。
- CLI `nextool`:7 一级子命令,参数/stdin 输入,错误非零退出码。
- 180 单元+集成测试通过(真实数据,无 mock);clippy `-D warnings` 零警告。
- 纯净审计通过(docs/audit/2026-08-21-core-tools.md)。
- Windows CLI 单二进制 4.5MB release。
