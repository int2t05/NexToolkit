# 核心工具层纯净审计报告

> 审计日期:2026-08-21 ｜ 范围:`crates/core/src/` 七模块(core v0.1.0,33 工具)
> 审计方式:子 agent 独立只读审计 + 集成者复核,对照 A-F 残留检查表

## 审计范围

`crates/core/src/` 下 8 文件:lib.rs / encode.rs / convert.rs / format.rs / generate.rs / text.rs / crypto.rs / nettime.rs。共 ~2500 行,33 个工具函数,168 单测(+2 DNS 网络测试 ignored)。

## 结论

**可提交。** 修复 2 类轻微残留(6 行)后状态纯净。

## 发现与修复

| # | 类别 | 文件:行 | 问题 | 修复 |
|---|---|---|---|---|
| 1 | A2/C9 溯源·版本号 | nettime.rs:60 | 注释嵌 `chrono 0.4` 版本号,依赖升级后过时 | 删版本号,保留 API 说明 |
| 2 | A2/C9 溯源·版本号 | nettime.rs:107 | 注释嵌 `hickory-resolver 0.24` 版本号 | 同上 |
| 3 | B4 装饰·横幅 | crypto.rs:10,17,70,143 | `// ==== xxx ====` 装饰边框,信息已被函数文档覆盖;且其余 6 文件无此样式,不一致 | 退化为无装饰单行注释(`// AES-256-GCM` 等) |

## 无问题项(逐条确认)

- **A1/A3 溯源**:无标签式溯源,分组按功能域非批次。
- **B5/B6/B7 装饰冗余**:无 benchmark 角标;match 多臂是类型系统必然非冗余;无 `// HACK`/`**除非**` 特例 guard。
- **C8 死指令**:`lib.rs` "各模块只写自己文件" 指令被遵守。
- **C10 死字段**:`PasswordOpts`/`HashAlgo`/`CaseMode` 全字段与变体均被消费。
- **D11 校验对应**:bits<2048、length<1、字符集空、n==0、salt<8、JWT 段数、进制越界 等校验与 Err 动作一一对应。
- **D13 共享结构**:ToolError 单一定义于 lib.rs,无 drift。
- **E15/E16 语言平台**:注释全中文(标识符/协议名除外);无 unsafe/asm/平台私有 API。
- **TODO 残留**:零命中。
- **手写 crypto**:无。PBKDF2 用官方 `pbkdf2::pbkdf2_hmac`(非手写),AES-GCM/RSA/Argon2/HMAC/Hash 均成熟 crate。
- **死代码**:无 `allow(dead_code)`;私有辅助函数均被调用。
- **注释与实现不符**:无。AES/JWT/XML/DNS/进制 文档与实现逐条核对一致。
- **错误处理**:统一 ToolError(8 变体);非测试 panic 仅 2 处不变式(`expect`/`unreachable`),均成立;2 处 `unwrap_or` 安全回退。
- **IO 泄漏**:core 零 println/stdin/stdout,纯逻辑。

## 验证证据

- `cargo test --workspace`:168 + 12 = 180 通过,0 失败,2 ignored(DNS 真实网络)。
- `cargo clippy --workspace --all-targets -- -D warnings`:零警告。
- CLI 端到端冒烟:base64/url/hex/jwt/json-yaml/numbase/json-fmt/hash/uuid/password/case/regex/ipcalc/timestamp/aes-gcm/rsa/pbkdf2 全部正确。

## 关键修正记录(集成阶段)

集成时对子 agent 产出的 4 处修正:

1. **crypto.rs PBKDF2**:子 agent 因 `Cargo.toml` 误配 `pbkdf2 = { default-features = false }` 关掉 `hmac` feature,手写了 PBKDF2-HMAC-SHA256。集成者改为启用 `hmac` feature 用官方 `pbkdf2::pbkdf2_hmac`,删除手写实现(避坑第 7 条:不手写 crypto)。
2. **generate.rs sha1**:子 agent 因缺 `sha1` crate 留 TODO 占位。集成者补 `sha1 = "0.10"` 依赖,实现 Sha1 哈希与 HMAC-SHA1,测试改用 RFC 2202 正向向量。
3. **generate.rs hash_sha512 测试**:子 agent 笔误写错期望向量末尾(`54b49bb`→应为 `54ca49f`)。集成者修正为 NIST 正确向量。
4. **encode.rs/url_decode、generate.rs/uuid_v7、nettime.rs/ipcalc**:依赖版本差异致 3 处 API 不匹配(`urlencoding::decode` 返回 Result 非 Option;`uuid` 1.24 `new_v7` 需参数改 `now_v7`;`IpNet::from_str` 类型推断),集成者修正。

## 待后续处理

- Tauri GUI 在 `x86_64-pc-windows-gnu` target 的链接兼容性(webview2-com FFI)待 GUI 集成阶段验证。
- Windows 页面文件不足(os error 1455)致并行编译 ICE,已用 `cargo --jobs 1` 规避;CI 矩阵环境内存充足不受影响,本机开发记此约束。
