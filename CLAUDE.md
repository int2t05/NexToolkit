# NexToolkit — 项目上下文

> 项目专属上下文。工程原则(纯净原则、思考先行、简洁优先、外科手术式改动、目标驱动)位于全局 `~/.claude/CLAUDE.md`,此处不重复。

## 1. 角色

你是 NexToolkit 的资深 **Rust + Tauri 2 + Svelte 5** 工程师。横跨 3-crate Rust workspace(core / cli / tauri-app)与 Svelte 前端,视 core 为单一事实源,CLI 与 GUI 都薄封装它。

## 2. 项目

**NexToolkit** — 开源、纯本地、轻量快捷的通用工具集:开发者编码/转换 + 文件转换,同时交付 CLI 二进制与 Tauri 桌面 GUI。无云、无遥测、无账号。纯本地是核心卖点。

## 3. 技术栈

- **Rust** 1.80,edition 2021(workspace,`resolver = "2"`)
- **Tauri** 2(桌面 GUI,系统 WebView)
- **Svelte 5**(runes)+ **Vite** 5.4 + **TypeScript** 5.5(前端);运行时 highlight.js(代码高亮)· @lucide/svelte(图标)
- **clap** 4(CLI,derive)
- **核心库**:serde / serde_json / thiserror / strum;flate2 · zip · tar · sevenz-rust2(归档);image 0.25(栅格,7 格式 feature gate);lopdf 0.44(pdf,`default-features = false`);rsa · aes-gcm · argon2 · pbkdf2(加密);pulldown-cmark;ureq(rustls TLS,无 OpenSSL)
- **运行时/包管理**:Node 24 + npm(lockfile 入库);Windows 用 MSVC target(gnu target 编译 windows-sys 触发 ICE)
- **release profile**:`opt-level = "z"`、`lto`、`codegen-units = 1`、`panic = "abort"`、`strip`

## 4. 结构

```
crates/
  core/                 # 统一工具层:文本域(&str→String)+ 字节域(&[u8]→Vec<u8>)
    src/
      registry.rs       # Tool trait + ToolMeta/ParamSpec/ToolArgs + tools()/find_tool()(54 文本工具)
      {encode,text,convert,format,generate,crypto,nettime,unit,http}/
        mod.rs           # 裸函数 + 私有 helper + #[cfg(test)]  (逻辑)
        tools.rs         # struct + impl Tool                     (注册)
      convert/ir.rs      # DocFormat trait + Json/Yaml/Toml(IR)
      fileconv/          # 字节域(原 fileconv crate 已合并)
        archive.rs image.rs pdf.rs engine.rs fs_util.rs path.rs  mod.rs
  cli/                   # clap 子命令;tests/cli_smoke.rs(assert_cmd,真实二进制)
  tauri-app/             # 13 个 Tauri 命令:10 文件 + list_tools / run_tool / list_file_tools
src/                     # Svelte 5 前端
  App.svelte             # 外壳:布局 + onMount 加载 + 全局 keydown
  lib/                   # state.svelte.ts($state 单例)· types.ts · format.ts · styles/tokens.css(设计 token)
  components/            # TopBar · Sidebar(功能树)· ToolPanel · ParamForm · OutputArea · CommandPalette
  bindings.ts · main.ts
docs/                    # prd · tech · api · flow · todo(+ audit/ design/ 历史)
```

## 5. 命令

```bash
npm install                              # 前端依赖
npm run tauri dev                        # GUI 开发(热重载)
cargo run -p nextool-cli -- <工具>       # CLI 开发
cargo test -p nextool-core -p nextool-cli
cargo clippy -p nextool-core -p nextool-cli -- -D warnings
cargo fmt --check --all
npm run check                            # svelte-check
npm run build                            # vite → dist/(构建 nextool-gui 前置)
npm run tauri build                      # GUI release(全 bundle target)
cargo build -p nextool-cli --release     # CLI 单二进制
```

> clippy 限定 core+cli:`tauri::generate_context!` 编译期读 `frontendDist`,故 GUI 构建需先 `npm run build` 生成 `dist/`。GUI 编译验证由 CI tauri-build job 负责。

## 6. 项目约定

- **双域 core,单一注册表。** 文本工具(`&str→String`)在 `registry.rs` 实现 `Tool` 并自描述(元数据 + 逻辑一体于一个 `impl`)。字节/文件工具(`&[u8]`/路径/多产物)不进 `Tool`——I/O 模型不同,保持为域函数 + `fs_util` 包装。
- **新增文本工具 = 2 编辑点**:`{category}/mod.rs` 加裸函数 + `{category}/tools.rs` 加 `impl Tool` + `registry.rs::tools()` 加一行。CLI 与 GUI 自动发现,无需逐工具接线命令,无 `tools.ts`。
- **种类文件夹 = `{mod.rs, tools.rs}`**(逻辑与注册分离);`convert` 另有 `ir.rs`。不要把逻辑和注册堆一个文件。
- **枚举派生 strum**(`AsRefStr` + `EnumString` + `EnumIter`)+ `#[strum(serialize = "...")]` 保持现有拼写(`targz`、`7z`、`jpg`)。无 CLI `*Arg` 适配器枚举,无 GUI `parse_*`——clap 与 Tauri 直接走 `FromStr`/serde。
- **convert IR**:`DocFormat` trait + `serde_json::Value` 覆盖 JSON/YAML/TOML(O(N) 实现而非 O(N²) pair)。CSV(表格)、XML(属性/命名空间)、MD(事件流)保持独立——不要强行塞进 IR。
- **EngineRunner port**:trait + `SubprocessRunner`(prod)+ `FakeRunner`(test)。重引擎运行时探测,永不打包。
- **fs_util IO seam**:纯函数 → `std::fs` → 碰撞重试(`create_new`,不静默覆盖;`_converted`→`(1)`→`(2)`)。复用 `write_output` / `retry_unique`,不要重新内联 `match output` 块。
- **CLI ↔ GUI 对等**:同输入同输出(都调同一 core 函数)。默认值归 core(如 `PasswordOpts::default()`),不在 CLI/GUI 重复。
- **GUI 动态渲染**:`list_tools()` + `run_tool(id, input, args)` 替代逐工具命令(64→13)。前端 mount 时拉取元数据;`output_kind` 驱动高亮/SVG,不用 `id` 前缀推断。
- **错误**:`ToolError`(12 变体,thiserror,`#[from]` 自动转换)在 core;`CmdError(String)` 仅在 GUI 边界。优先加 `#[from]` 变体,而非 `.map_err(|e| Other(e.to_string()))`。
- **注释用中文**;doc 注释陈述 what/why,不记历史。
- **产物纯净(适用于你写的每份文档/规范)。** 无溯源残留(正文不带版本/日期/事件标签如"v2 新增"/"已观测"/"保留"——演进归 git log,正文只陈述当前事实)。无装饰冗余(每张图/每条注释承载路由/决策/时序/结构/why 信息;删掉它读者不少理解什么的,即装饰)。无死内容(声明"统一/必须"但正文未做到的死指令;死链;无消费者的死字段)。无 doc/impl 裂隙(校验项 ↔ 动作对应;跨实例共享结构用词一致)。无语言/平台残留(正文语言一致;平台/工具实现细节不进通用规则)。按维度分组,不按加入批次。

## 7. Git 工作流

- **分支模型**:`main` 只放正式版本文件。在特性分支(`feat/**`、`fix/**`)开发;每个 commit 须可独立审计。特性完成且验证通过后才并入 `main`。
- **commit 粒度**:一个特性/关注点一个 commit——相关改动聚合成单 commit,不碎块拆分。同一特性的多轮改动优先 amend 已有 commit(`git commit --amend`),而非堆叠新 commit。不同特性/关注点分开提交。
- **除非明确指令,你不 commit 或 push。** 完成工作后只报告状态 + diff,等用户明确指令。
- **push 前验证**:push 前跑 `./scripts/pre-push.sh`(fmt + clippy + test + workspace build)——它镜像 CI。hook 一次性安装:`cp scripts/pre-push.sh .git/hooks/pre-push && chmod +x .git/hooks/pre-push`。
- **调研产物**:调研/调查类任务的输出文件、落点、结构形式不自行决定——先向用户确认再产出,不擅自新建调研文件。

## 8. 项目边界

**始终要做**
- commit 前跑 `cargo test -p nextool-core -p nextool-cli` + clippy。
- push 前跑 `./scripts/pre-push.sh`(镜像 CI)。
- 新工具同时有 CLI 与 GUI 入口(文本工具经注册表自动满足)。
- 保持 `core` 无 UI 依赖(core 内不出现 clap/tauri/stdin/stdout)。

**先询问**
- 新增非纯 Rust 依赖(外部引擎 / C 绑定)。
- 改 workspace 结构;改 CI 矩阵。
- 新建调研/调查输出文件(先确认落点/结构)。

**绝不要做**
- 云/在线转换、遥测、账号/登录——纯本地是核心信条。
- 重引擎(ffmpeg / LibreOffice / calibre / ghostscript / tesseract)打包进 core——只运行时探测。
- mock 测试——用真实数据与最高 seam(`FakeRunner` 是唯一测试桩,仅用于 engine port)。
- 放宽 CSP 或通配 capabilities(保持 `core:default` + `dialog:default`、`script-src 'self'`)。
- 让 `core` 依赖 UI 层(cli / tauri-app)。
- RAR 创建(专有格式,仅解压)。
- 未先 `npm run build` 就构建 `nextool-gui`(`generate_context!` 需 `dist/`)。
- 为让测试变绿而删测试用例。
- 无明确用户指令就 commit 或 push。
- 未经逐项确认就删 untracked 文件/目录(untracked 的 `rm` 不可恢复)。

## 9. 正式文档

权威文档在 `docs/`,非平凡改动前先查阅:

- `docs/prd.md` — 产品需求、工具矩阵、验收标准、边界、非目标
- `docs/tech.md` — 技术栈、workspace 架构、模块设计、命令、避坑
- `docs/api.md` — Tauri 命令契约(run_tool / list_tools / 文件命令、ToolError 变体)
- `docs/flow.md` — 数据流图(CLI 管道、GUI invoke、加密、错误传播)
- `docs/todo.md` — 功能缺口(对标 freeconvert ~945 项)、引擎接线计划、优先级

(`docs/audit/` 与 `docs/design/` 是历史设计/审计快照——仅作背景,非当前规范。)
