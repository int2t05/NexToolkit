<!--
PR 前确认:分支 feat/* 已 CI 全绿;main 只放可运行版本。
中文描述,说明 what + why。
-->

## 变更
<!-- 关联 issue;简述改了什么、为何改 -->

## 类型
- [ ] feat 新工具 / 功能
- [ ] fix 缺陷修复
- [ ] docs 文档
- [ ] chore 工程 / CI

## 核对
- [ ] core 测试真实数据,无 mock
- [ ] `cargo clippy -p nextool-core -p nextool-cli -- -D warnings` 无 warning
- [ ] `cargo fmt --check --all` 通过
- [ ] 新工具同步加 CLI 入口 + GUI command + `tools.ts` + 文档(prd/api)
- [ ] 中文注释,无 TODO/FIXME 残留
- [ ] 设计有据(参考同类开源实现,标注来源)
