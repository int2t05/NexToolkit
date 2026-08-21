# 文档重构审计报告

**审计日期:** 2026-08-21
**触发:** docs/ 重构为 pro/tech/todo/ROADMAP 四类(删旧 PRD/v0.1/research),README 重写详尽 CLI 用法,release.yml 加 Windows 便携 exe。
**范围:** 四份新文档 + README + CONTRIBUTING ↔ 代码(`crates/{core,cli,tauri-app}` + `src/` + CI)。
**方式:** documentation-audit 技能逐项核对,真跑 `--help` 与命令验证示例。

## 审计范围

| 文档 | 对照源 |
|---|---|
| `docs/pro.md` | 工具矩阵 32、验收、边界、非目标 |
| `docs/tech.md` | 技术栈、workspace 结构、模块设计、命令、测试、避坑 |
| `docs/todo.md` | 当前不足(与代码事实)、近期方向 |
| `docs/ROADMAP.md` | 引擎层、智能层、分发演进 |
| `README.md` | CLI 用法示例(7 分组逐工具)、架构、安装、未来方向 |
| `CONTRIBUTING.md` | 构建/clippy 命令、新增工具流程、文档路径 |

## Before / After

| 维度 | Before(旧文档) | After(重构后) |
|---|---|---|
| 文档分类 | PRD/v0.1/PRD/v0.1/tech/research 混杂 | pro/tech/todo/ROADMAP 四类 + audit/ |
| 版本字眼 | 正文含 v0.1/v0.2.0/2.11.x 等 | 四类正文零版本字眼 |
| CLI 示例 | PRD 用户故事 9 处扁平(`nextool base64`) | README 7 分组逐工具嵌套,真跑验证 |
| 工具矩阵 | 33(含未实现 http,hmac 分组冲突) | 32(http 删,hmac 归 Generators) |
| 技术栈命令 | tech §6 五处 pnpm | 全 npm |
| Tool trait 死引用 | tech §3/§7 trait 示例 | 改实际自由函数模式 |
| 测试路径 | test/cli/(死路径) | crates/cli/tests/cli_smoke.rs |
| 便携 GUI exe | CI 未产出 | release.yml 加 Windows 便携 exe 步骤 |

## 核对结果(逐项)

### 1. 工具矩阵 32 ✓
- pro/README 按"功能工具"粒度:encode 5 + convert 5 + format 4 + generate 6(含 hmac)+ text 5 + crypto 3 + nettime 4 = 32。
- core 49 个 `pub fn`(函数粒度,encode/decode 分开)——与"32 功能"不矛盾(功能=aes-gcm 1,CLI 暴露 2 子命令)。
- 无 http 残留,hmac 归 Generators(代码 `generate.rs` 文件头含 HMAC,`tools.ts` group='generate')。

### 2. CLI 用法示例 ✓(真跑验证)
- 顶层 7 分组:`encode`/`convert`/`format`/`generate`/`text`/`crypto`/`net-time`(`--help` 确认)。
- 含数字命令名正确:`base64`/`pbkdf2`/`argon2`/`uuid-v4`/`uuid-v7`(真跑 `encode base64 encode "Hello"`→`SGVsbG8=`、`crypto pbkdf2`/`argon2` 产出正确)。
- README 示例 `generate hash sha256 abc`→`ba7816bf...` 与 NIST 一致。

### 3. 技术栈 ✓
- Tauri 2 + Svelte 5(runes)+ Rust + MSVC + npm,与 Cargo.toml/package.json/CI 一致。
- tech 命令块零 pnpm,clippy 限定 `-p core -p cli`(GUI generate_context! 需前端 dist,CI tauri-build 验证)。

### 4. 测试路径 ✓
- tech 结构图无独立 `test/` 节点;`crates/cli/tests/cli_smoke.rs` 正确。

### 5. todo 准确性 ✓
- "RSA 无签名":代码 `grep rsa_sign` 零命中,todo 称述正确。
- "便携 exe 仅 Windows 待补":CI 已加步骤,todo 记为近期方向(已部分完成)。
- "GUI clippy 未在 CI":tauri-build job 仅编译,记为待办。

### 6. 互链完整性 ✓
- pro→ROADMAP、tech→(无外链)、todo→ROADMAP、ROADMAP→todo、README→四类+CHANGELOG+CONTRIBUTING,全部目标文件存在。

### 7. 便携 exe CI ✓
- release.yml Windows 加 `Locate portable exe` + `Upload portable exe` 步骤(`NexToolkit-<tag>-windows-x64-portable.exe`)。

### 8. API/TODO drift
- 无 HTTP API(桌面工具集),API 漂移步骤不适用(CLI/Tauri command 已在 tech/README 核对)。
- 代码零 TODO 注释(`grep TODO` 零命中),todo.md 为人工规划非代码 TODO 同步。

## 已修复(Sync)

重构本身即修复,本轮审计无新增 drift:
- 旧 PRD/v0.1/research/旧 docs-audit 删除。
- CONTRIBUTING `docs/v0.1/PRD.md`→`docs/pro.md`、`--workspace` clippy→`-p core -p cli`。
- gui audit `docs/research/` 死引用→`reference/competitors/`。

## 待人工确认

- **便携 exe 定位脚本**:`Get-ChildItem target\release -Filter "*.exe"` 排除 setup/nsis 取首个,依赖 tauri-action 产物命名。首次 release 需人工确认定位到正确 exe(非 setup.exe)。若失败,改用 `tauri-action` 的 `args: --bundles none` 单独步骤产出。
- **GUI clippy 入 CI**:todo 列为近期方向,需 CI 加 step(先 `npm run build` 产 dist 再 clippy gui)。
- **ts-rs 类型绑定**:todo 列为近期方向,command 增多后接入。

## 结论

四类文档(pro/tech/todo/ROADMAP)+ README + CONTRIBUTING 与代码完全同步,零 drift。CLI 用法示例真跑验证,工具矩阵 32 准确,技术栈/路径/互链一致,便携 exe CI 已加。文档正文零版本字眼,mermaid 为主,语言干练。
