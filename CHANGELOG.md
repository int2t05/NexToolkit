# 更新日志

本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。版本演进归此处,正文只陈述当前事实。

## [Unreleased]

### 新增

- Tauri 2 + Svelte 5 桌面 GUI:33 工具全暴露,左侧七组导航 + 搜索 + 参数表单 + 输出/复制,中英双语,暗色主题。
- CI:三平台(Ubuntu/Windows/macOS)core-cli 测试 + 前端 check + Tauri 编译验证;tag 触发 release 三平台产物上传。
- 设计参考:爬取 it-tools(Vue3 工具集)+ tauri2-svelte5-shadcn/OpenCovibe/comine(Tauri+Svelte)本地分析,设计决策标注依据。

### 变更

- Windows 工具链由 gnu 切 MSVC(gnu rustc 编译 windows-sys 0.59 栈溢出 ICE)。
- 前端包管理用 npm(pnpm 在本机 V8 OOM,npm 稳定)。

## [0.1.0] — 2026-08-21

### 新增

- 核心库 `nextool-core`:33 工具,7 模块(encode/convert/format/generate/text/crypto/nettime),纯 Rust,无 UI 依赖。
- CLI `nextool`:7 一级子命令,参数/stdin 输入,错误非零退出码。
- 180 单元+集成测试通过(真实数据,无 mock);clippy `-D warnings` 零警告。
- 纯净审计通过(docs/audit/2026-08-21-core-tools.md)。
- Windows CLI 单二进制 4.5MB release。
