# 贡献指南

## 开发环境

- Rust(stable,Windows 用 MSVC target)
- Node.js 24 + npm
- Tauri 2 prerequisites(见 https://tauri.app/start/prerequisites/)

## 开发流程

```bash
git checkout -b feat/<功能>
npm install
cargo test -p nextool-core -p nextool-cli   # 测试必须全绿(真实数据,无 mock)
cargo clippy -p nextool-core -p nextool-cli --all-targets -- -D warnings
cargo fmt --check --all
npm run check                   # 前端类型检查
```

## 准则

- **纯净**:修复根因不留症状补丁;无 TODO/FIXME 残留;注释中文,解释 why。
- **测试**:真实调用真实数据,禁止 mock。新工具 ≥3 用例(正常/边界/错误)。
- **简洁**:最小代码解决问题,抽象须挣复杂度。
- **设计有据**:UI/架构改动须有 reference 参考(爬取同类开源实现),标注依据来源。
- **分支**:`main` 只放可运行版本;开发在 `feat/*` 分支,CI 全绿后合并。

## 新增工具

1. 文本域:`crates/core/src/<域>.rs` 实现纯逻辑函数 + 单测;`lib.rs` 加 `pub use <域>::*;`。字节域(文件转换):`crates/core/src/fileconv/<域>.rs`,纯内存逻辑 + 单测,`fileconv/mod.rs` 加 `pub mod`。
2. `crates/cli/src/<域>_cmd.rs` 加子命令;`main.rs` 注册。
3. `crates/tauri-app/src/commands.rs` 加 `#[tauri::command]`;`lib.rs` 的 `use` 与 `generate_handler!` 注册。
4. `src/tools.ts` 加工具元数据(分组/参数 schema)。
5. 更新 `docs/prd.md` 工具矩阵与 `docs/api.md` 命令契约。

## 提交

- Conventional Commits:`feat:`/`fix:`/`docs:`/`chore:`。
- 中文描述,说明 what + why。
- push 需人工确认(不自动 push 到 main)。
