# 更新日志

本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。版本演进归此处,正文只陈述当前事实。

## [Unreleased]

## [0.4.0] — 2026-08-23

### 新增

- 引擎管理子系统:便携版自动安装(ffmpeg/pandoc 下载解压到 `%APPDATA%/NexToolkit/engines/`)+ CLI `engine check`/`engine install` + GUI 安装按钮。
- 字符编码转换工具:UTF-8/GBK/GB2312/GB18030/Big5/Shift_JIS/EUC-JP/EUC-KR/ISO-8859-1/Windows-1252(encoding_rs,纯 Rust)。
- 主题系统:明暗 + 跟随系统三态(TopBar Sun/Moon/Monitor 切换)。
- 通用文件转换命令:`file-conv convert <input> <target>` 按源格式自动路由引擎(Office→LibreOffice/MD→pandoc/电子书→calibre/PDF→ghostscript)。
- LibreOffice Windows 路径回退(`resolve_binary` 查 Program Files 默认安装路径)+ `CREATE_NO_WINDOW` 隐藏子进程 cmd 窗口。
- Tesseract Windows 路径回退(`C:\Program Files\Tesseract-OCR\`)。
- 前端架构重构:9 组按使用频率排序 + 子分类精简 46→30 + 哈希归 crypto + nettime 拆 network/time + BIDIRECTIONAL 扩展(chacha20/aes_gcm)。
- GUI 图标设计:logo.svg(六边形+N 字)+ app icon PNG 序列 + icon.ico 重生成。
- E2E 留痕目录:`tests/run.sh` 49 个测试 + 入库 fixture + report.md。
- PDF 测试覆盖:split/rotate/encrypt/ranges/delete/extract/metadata/pagenum/merge。

### 变更

- 分组重命名:convert→"数据转换"、text→"文本处理"、crypto→"加密与哈希"(消除"转换/格式化"歧义)。
- 删 5 个冗余 Tauri 命令(office_to_pdf/ebook_convert/markup_convert/pdf_compress/pdf_to_text),统一走 `convert_file` 通用入口。
- 字号统一 token 体系:group 12px / sub+tool 13px。
- 删 docs/audit/ + docs/design/(历史快照)。
- GUI 组件:ToolTabs 删除(左侧树承担切换)+ EngineManager 面板化。

### 修复

- LibreOffice `-env:UserInstallation=...` 参数用 `=` 连接(之前拆两个参数导致退出码 1)。
- Sidebar 收起后当前选中工具的 subgroup 保持展开。
- toggle 按钮统一展开/收起(含 group + subgroup)。
- `convert_pdf_to_text_with_content` 测试改 ASCII 内容(pdflatex 不支持 CJK)。

## [0.3.0] — 2026-08-22

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

### 新增

- 核心库 `nextool-core`:32 工具,7 模块(encode/convert/format/generate/text/crypto/nettime),纯 Rust,无 UI 依赖。
- CLI `nextool`:7 一级子命令,参数/stdin 输入,错误非零退出码。
- 180 单元+集成测试通过(真实数据,无 mock);clippy `-D warnings` 零警告。
- 纯净审计通过(docs/audit/2026-08-21-core-tools.md)。
- Windows CLI 单二进制 4.5MB release。
