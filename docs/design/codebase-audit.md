# 代码库架构审计

> 本文档是**重构前的架构审计快照**,记录"为何这样设计"的决策依据,不是当前实现说明(那是 `tech.md` 的职责)。
>
> **历史语境**:审计时 `fileconv` 是独立 crate(后已并入 `core` 作为 `core::fileconv` 子模块)。本文保留当时的 crate 边界描述,作为设计决策记录,不回填合并后状态。

## 背景与问题

NexToolkit 目标是 945+ 功能的本地工具集(对标 freeconvert)。重构前每新增一个功能需 **6-9 个编辑点跨 3 个 crate**:新增文本工具要在 core 写逻辑 + CLI 加子命令 match 臂 + GUI 改 4 处(commands 两函数 + lib.rs 命令名写两遍 + tools.ts schema);新增图像格式要在 fileconv 改 5 处 + CLI 双重枚举 + GUI 2 处。

审计识别出 **7 个深化机会**,归为三条接缝(适配器接缝 / fileconv 边界接缝 / engine 接缝)。每条接缝的核心是:**纯逻辑已存在且正确,但边界处反复手写适配样板,阻碍规模化**。

---

## 候选目录

| # | 候选 | 接缝 | 收益 | 风险 |
|---|---|---|---|---|
| 1 | 枚举字符串映射移入枚举 | A | 删 CLI 3 种 `*Arg` + GUI 5 个 `parse_*` | 低(机械替换) |
| 2 | fs_util 补全 IO seam | B | 消 CLI/GUI inline `std::fs::read` | 低(纯包装) |
| 3 | 密码默认下沉 core | A | CLI/GUI 行为统一,删三重 `!upper && !lower...` | 低 |
| 4 | CLI 文本域 helper | A | 折叠 ~30 臂 `read→map_err→println` 样板 | 低 |
| 5 | EngineRunner port | C | 引擎可 mock,接线前定设计 | 中(当前孤儿,需 P4 接线落地) |
| 6 | archive_list 结构化 | B | presentation 移适配器,GUI 不再解析字符串 | 中(向后兼容) |
| 7 | 合并碰撞重试 | B | DRY `write_bytes_safe`/`resolve_extract_dir` | 低 |

---

## 接缝 A · 适配器接缝(候选 #1 #3 #4)

适配器接缝指**枚举/配置在 core 与 CLI/GUI 之间的字符串映射**。core 已有正确枚举,但 CLI 为兼容 clap 又包一层 `*Arg` 适配器,GUI 为兼容 Tauri 字符串又写一层 `parse_*`。同一映射三处实现。

### A1 · 枚举字符串映射移入枚举

**现状**:4 个枚举各自手写映射,每个枚举在 CLI 和 GUI 各有一份适配器。

| 枚举 | core 位置 | CLI 适配器 | GUI 适配器 |
|---|---|---|---|
| `HashAlgo` | `generate.rs` | `HashAlgoArg` + `From` | `parse_hash_algo` |
| `CaseMode` | `text.rs` | `CaseModeArg` + `From` | `parse_case_mode` |
| `ArchiveFormat` | `archive.rs` | `ArchiveFormatArg` + `to_format` | `parse_archive_format` |
| `ImageFormat` | `image.rs` | `ImageFormatArg` + `to_format` | `parse_image_format` |

**重构**:枚举加 `FromStr` + `Display`(或等价派生),字符串↔枚举映射成为枚举自身的职责。CLI 删 `*Arg` 适配器,clap 字段直接用 core 枚举;GUI 删 `parse_*`,Tauri 反序列化直接走 `FromStr`。

#### ADR-1 · 用 FromStr 而非 clap ValueEnum

- **决策**:枚举实现 `FromStr` + `Display`,不依赖 clap `ValueEnum` 派生。
- **理由**:`ValueEnum` 强制 kebab-case 序列化,而现有拼写是自定义的(如 `"sha256"`、`"snake"`)。`FromStr` 保留任意拼写,且非 clap 专属(同样服务 GUI 反序列化与未来注册表)。
- **代价**:clap 需通过 `value_parser` 接入 `FromStr`,比 `ValueEnum` 略多一行声明。
- **结论**:拼写自由 > 框架便利,采用 `FromStr`。

### A2 · 密码默认字符集规则移入 core

**现状**:`PasswordOpts` 无 `Default`,CLI 在 `generate_cmd.rs` 内手写三重 `!upper && !lower && !digits && !symbols` 判空后回退默认;GUI 各 bool 字段独立默认。两处默认规则可能漂移。

**重构**:`PasswordOpts` 加 `impl Default`(全 false → 字母数字不含符号)。CLI 删判空逻辑,GUI bool 字段改 Rust `bool`(Tauri 反序列化 JSON bool),默认值由 core 单点提供。

### A3 · CLI 文本域样板 helper

**现状**:文本域子命令每个臂是 `read_input → map_err → 调核心 → println` 四行样板。format_cmd 6 臂 × 4 行 = 24 行可压到 6 行。全 CLI ~58 处 `map_err` 多数来自此模式。

**重构**:`cli/src/io.rs` 加 `print_text(input, F)` helper,折叠 read→map_err→println。

#### ADR-2 · 用 helper 而非宏

- **决策**:用普通函数 helper,不引入宏。
- **理由**:闭包捕获足够表达当前所有臂,宏会引入 hygiene/调试难度,收益不抵成本。helper 是 Deep Module——小接口隐藏完整 IO 序列。
- **结论**:采用函数 helper。

---

## 接缝 B · fileconv 边界接缝(候选 #2 #6 #7)

fileconv 边界接缝指**文件 IO 与展示逻辑泄漏到 CLI/GUI**。fs_util 已封装大部分 IO,但仍有缺口(archive 列表、PDF 加密探测),且 archive_list 把展示格式写死在 core。

### B1 · 补全 fs_util IO seam

**现状**:`archive_list` 只接受 `&[u8]`(数据域),但 CLI(`archive_cmd.rs` 列表命令)和 GUI(`commands.rs` 列表命令)各自 inline `std::fs::read` 读文件再调。PDF 加密探测同理。

**重构**:fs_util 加两个包装:
- `list_archive_file(path) -> ToolResult<Vec<...>>` — 读文件 + 调 archive_list
- `is_pdf_encrypted_file(path) -> ToolResult<bool>` — 读文件 + 调 pdf_is_encrypted

CLI/GUI 调包装,不再 inline read。

### B2 · archive_list 返回结构化

**现状**:`archive_list(data) -> String` 直接返回格式化字符串(带对齐/缩进)。CLI 直接打印;GUI 要么原样显示要么解析字符串——presentation 与逻辑耦合在 core。

**重构**:拆为:
- `archive_list_entries(data) -> ToolResult<Vec<ArchiveListEntry>>` — 结构化(路径 + 大小)
- `archive_list(data) -> String` — 格式化,向后兼容 CLI

GUI 走结构化路径,presentation(对齐/排序/单位)在适配器层。

### B3 · 合并碰撞重试

**现状**:`write_bytes_safe`(L243)与 `resolve_extract_dir`(L259)实现同一碰撞重试算法(`0..100` 循环 + `AlreadyExists` 跳过),两份代码。

**重构**:抽 `retry_unique(path_fn, op, collision_msg)` helper,两处复用。

---

## 接缝 C · engine 接缝(候选 #5)

engine 接缝指**外部引擎子进程调用内联在 `engine_convert`**。当前 `detect_engine`(探测 PATH)与子进程 spawn 混在业务函数里,无法单测,且 todo.md P4 要接线 5 个引擎,内联模式会让每个引擎都复制 spawn 副作用。

### C1 · EngineRunner port

**现状**:`engine_convert` 直接 `Command::new(engine.binary())...status()`,PATH 探测 + spawn + 错误处理耦合。无测试桩,任何引擎接线改动都需真实安装引擎才能验证。

**重构**:抽 port:

```text
trait EngineRunner: Send + Sync {
    fn is_available(&self, engine: Engine) -> bool;
    fn run(&self, engine: Engine, input: &str, output: &str) -> ToolResult<()>;
}
SubprocessRunner   // prod:PATH 探测 + Command::new
FakeRunner         // test:记录 args,返回 canned Ok/Err
engine_convert(runner: &dyn EngineRunner, engine, input, output) -> ToolResult<String>
```

#### ADR-3 · 用 port 而非内联子进程

- **决策**:引擎执行经 `EngineRunner` trait 抽象,不内联在 `engine_convert`。
- **理由**:todo.md P4 明确要接线 5 个引擎(ffmpeg/LibreOffice/calibre/ghostscript/tesseract),内联模式每个引擎的测试都需真实安装,port 让 `FakeRunner` 证明 seam 可测。符合 Ports & Adapters:纯逻辑拥有完整接口,副作用隔离在适配器。
- **代价**:多一层 trait + 桩实现。但 `engine_convert_file`(fs_util 包装)是真实消费者,非孤儿投机。
- **结论**:采用 port。详见 `engine-feasibility.md`。

---

## 文件架构职责表

| 文件 | 职责 | 重构动作 |
|---|---|---|
| `core/src/generate.rs` | HashAlgo 枚举 + 哈希逻辑 | A1:枚举加 FromStr/Display |
| `core/src/text.rs` | CaseMode + 文本工具 | A1:同上 |
| `core/src/lib.rs` | PasswordOpts | A2:加 Default |
| `fileconv/src/archive.rs` | ArchiveFormat + 归档 | A1 + B2:枚举映射 + 拆 list |
| `fileconv/src/image.rs` | ImageFormat + 图像 | A1:枚举映射 |
| `fileconv/src/fs_util.rs` | 文件 IO seam | B1/B3:补包装 + retry_unique |
| `fileconv/src/engine.rs` | 引擎子进程 | C1:EngineRunner port |
| `cli/src/io.rs` | CLI IO helper | A3:print_text |
| `cli/src/*_cmd.rs` | 子命令 | A1/A2/A3:删适配器 + 折叠臂 |
| `tauri-app/src/commands.rs` | Tauri 命令 | A1:删 parse_* |

---

## 测试策略:替换而非叠加

重构遵循 **replace don't layer** 原则:

- 适配器删除后,原有路径测试(map_err 转换、parse_* 拼写)由枚举 `FromStr` 单测覆盖,不保留旧适配器测试。
- `archive_list` 拆分后,String 版本保留向后兼容测试,新增结构化版本测试;GUI 字符串解析测试(若有)删除。
- `engine_convert` port 化后,`FakeRunner` 单测覆盖可用性检查 + 调用委托 + 错误传播;真实引擎测试标 `#[ignore]`(需本机安装)。
- 每步独立 commit,`cargo test` 全过才进下一步。

不新增"过渡兼容层"测试——重构即替换,旧代码删则旧测试删。

---

## 推荐顺序

| 步 | 动作 | 依赖 | 验证 |
|---|---|---|---|
| 1 | ToolError 扩展(高频变体) | 无 | 现有 `?` 自动转换替代 map_err |
| 2 | 枚举 FromStr/Display(A1) | 无 | `"sha256".parse()` 测试 |
| 3 | 密码 Default(A2) | 无 | CLI/GUI 默认一致 |
| 4 | fs_util 补全 + retry_unique(B1/B3) | 无 | 新包装单测 + 现有 fs_util 测试 |
| 5 | archive_list 结构化(B2) | 步 4 | 结构化 + 向后兼容测试 |
| 6 | EngineRunner port(C1) | 步 1 | FakeRunner 单测 |
| 7 | CLI print_text + 删 *Arg(A3/A1) | 步 2 | `nextool generate hash sha256 "abc"` 冒烟 |

顺序原则:无依赖的先做(步 1-5),有依赖的按链推进。每步独立可回退。

---

## 验收清单

- [ ] 4 枚举加 FromStr/Display,CLI 无 `*Arg` 适配器,GUI 无 `parse_*`
- [ ] `PasswordOpts::default()` 存在,CLI 无判空三重否定,GUI bool 字段
- [ ] `print_text` helper 存在,format_cmd 等臂为单行
- [ ] `list_archive_file`/`is_pdf_encrypted_file` 存在,CLI/GUI 无 inline `std::fs::read`
- [ ] `archive_list_entries` 结构化版本存在,String 版本向后兼容
- [ ] `retry_unique` 存在,`write_bytes_safe`/`resolve_extract_dir` 复用
- [ ] `EngineRunner` trait + `SubprocessRunner` + `FakeRunner` 存在,`engine_convert` 接受 `&dyn EngineRunner`
- [ ] `cargo test --workspace` 全过(除 `#[ignore]` 真实引擎测试)
- [ ] 新增一个文本工具的编辑点数 ≤ 2(理想:core 1 处 + 注册表 1 处)

---

## 关联文档

- `engine-feasibility.md` — 接缝 C 的引擎层前瞻设计(接线步骤 / 进度流 / 安装提示)
- `frontend-tree.md` — 前端结构设计(动态渲染 / run_tool 分流 / output_kind)
- `../todo.md` — 功能 backlog(P4 引擎接线需求来源)
