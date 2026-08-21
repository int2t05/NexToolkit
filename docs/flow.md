# 业务流程

> 各业务 mermaid 流程与数据流描述。CLI 与 GUI 共享 `nextool-core`,数据流以 core 为中心。

## CLI 管道

```mermaid
flowchart LR
  ARG["参数 / stdin"] --> IO["io::read_input"]
  IO --> CLAP["clap 子命令<br/>encode/convert/.../net-time"]
  CLAP --> CORE["nextool-core 函数"]
  CORE -->|"Ok"| OUT["stdout 打印"]
  CORE -->|"Err"| ERR["eprintln + exit 1"]
```

**数据流:** `read_input` 优先取位置参数;缺省则读 stdin(管道友好)。子命令模块(`*_cmd.rs`)解析 clap 参数后调 core 同名函数,`Ok` 打印到 stdout 退出 0,`Err` 经 `e.to_string()` 打到 stderr 退出 1。RSA 的 `--pub-pem`/`--priv-pem` 既可内联 PEM 也可文件路径(`load_pem` 按是否以 `-----BEGIN` 开头判定)。

## GUI invoke

```mermaid
flowchart LR
  TS["tools.ts 元数据<br/>分组/参数 schema"] --> UI["工具面板渲染表单"]
  UI -->|"invoke(cmd, args)"| BIND["bindings.ts<br/>@tauri-apps/api/core"]
  BIND --> CMD["commands.rs<br/>#[tauri::command]"]
  CMD --> CORE["nextool-core 函数"]
  CORE -->|"Ok(String)"| RESP["前端输出区 + 复制"]
  CORE -->|"Err"| CE["CmdError(String)"]
  CE -->|"reject"| CATCH["前端 catch 展示"]
```

**数据流:** `tools.ts` 声明式 schema 驱动 UI,主输入为 textarea(传 `input`),参数表单按 `params` 渲染。前端 camelCase 参数经 Tauri 转 snake_case 进入 command;command 仅转发到 core(枚举/布尔经 `parse_*` 辅助解析)。`Ok` 返回字符串渲染输出;`Err` 经 `CmdError` 序列化 reject,前端 catch 展示。生成类工具(`needsMainInput=false`)不渲染主输入区。

## AES-256-GCM 加解密

```mermaid
flowchart TD
  subgraph 加密["aes_gcm_encrypt(plaintext, password)"]
    R["OsRng 生成 salt(16B) + nonce(12B)"] --> K["PBKDF2-HMAC-SHA256<br/>password+salt · 100_000 轮 → 32B key"]
    K --> E["Aes256Gcm 加密 plaintext"]
    E --> B["blob = salt‖nonce‖ciphertext‖tag"]
    B --> B64["base64 输出"]
  end
  subgraph 解密["aes_gcm_decrypt(b64, password)"]
    D1["base64 解码"] --> S["切出 salt(16) / nonce(12) / ciphertext"]
    S --> K2["PBKDF2 派生 key"]
    K2 --> D2["Aes256Gcm 认证解密"]
    D2 -->|"认证失败"| F["InvalidInput"]
    D2 -->|"成功"| U["UTF-8 → String"]
  end
```

**数据流:** 随机 salt+nonce 使同一明文+口令产生不同密文(非确定性),但均可解回。密文自包含 salt/nonce,解密只需口令。认证失败(口令错或数据损坏)统一返回 `InvalidInput("解密失败:口令错误或数据损坏")`;数据不足 28 字节(salt+nonce)直接 `InvalidInput`。

## RSA 密钥与加解密

```mermaid
flowchart TD
  KG["rsa_keygen(bits)"] -->|"bits<2048"| REJ["InvalidInput"]
  KG -->|"≥2048"| GEN["RsaPrivateKey::new"]
  GEN --> PEM["PKCS#1 PEM:私钥块 + 空行 + 公钥块"]
  ENC["rsa_encrypt(input, pub_pem)"] --> PE["parse 公钥(PKCS#8/SPKI 或 PKCS#1)"]
  PE --> OE["OAEP-SHA256 加密"]
  OE --> EB["base64 输出"]
  DEC["rsa_decrypt(b64, priv_pem)"] --> PD["parse 私钥(PKCS#8 或 PKCS#1)"]
  PD --> OD["OAEP-SHA256 解密"]
  OD --> U2["UTF-8 → String"]
```

**数据流:** 密钥对输出 PKCS#1 PEM(`-----BEGIN RSA PRIVATE KEY-----` 与 `-----BEGIN RSA PUBLIC KEY-----`,空行分隔)。加解密用 OAEP-SHA256 填充。PEM 解析兼容 PKCS#8/SPKI 与 PKCS#1 两种格式。仅做加解密,无签名/验签(见 [todo.md](todo.md))。

## 错误传播

```mermaid
flowchart LR
  CORE["core 返回 ToolError"] -->|"CLI"| CLIERR["e.to_string() → stderr · exit 1"]
  CORE -->|"GUI"| FROM["From<ToolError> → CmdError"]
  FROM --> SER["serde 序列化 String"]
  SER --> REJ["invoke reject"]
```

**数据流:** core 统一 `ToolError`(8 变体,`thiserror` 派生)。CLI 直接 `to_string` 进 stderr 并非零退出;GUI 经 `From<ToolError> for CmdError` 转字符串序列化,invoke reject 由前端 catch。两入口错误信息一致(同一 `ToolError::to_string`)。

## 数据形态

- **文本工具**(编解码/转换/格式化/文本/生成/网络时间):输入输出均为 UTF-8 字符串,经 `String` 传递。
- **加密工具**:输入为字符串,密文/哈希以 base64 或十六进制字符串返回(字节流编码为可传输文本)。
- **DNS 查询**:唯一发真实网络请求的工具(hickory-resolver 同步阻塞),其余工具纯计算无 IO。
- **文件转换**(归档):字节域 `&[u8]→Vec<u8>`,core/fileconv 纯内存;磁盘读写留 CLI/GUI 边界(fs_util)。

## CLI 归档管道

```mermaid
flowchart LR
  PATH["文件路径参数"] --> READ["io::read_bytes_input"]
  READ --> FC["fileconv::fs_util"]
  FC --> ARCHIVE["archive 纯内存<br/>extract/create/convert"]
  ARCHIVE --> COLLIDE["compute_*_path<br/>+ create_new 碰撞"]
  COLLIDE --> WRITE["写产物到源目录"]
  WRITE --> STDOUT["stdout 报路径"]
```

**数据流:** CLI `file-conv archive` 子命令读文件路径→`read_bytes_input` 取字节→委托 `fileconv::fs_util`(extract_to_dir/compress_files/convert_file)。fs_util 调纯内存 `archive_*` 转换,再 `compute_output_path`/`compute_extract_dir` 算产物路径,`create_new` 原子写入源文件所在目录(碰撞迭代后缀)。list 命令仅 `archive_list` 纯内存查看,输出到 stdout。

## GUI 文件转换

```mermaid
flowchart LR
  DIALOG["dialog::open 选择文件"] --> PATH["路径"]
  PATH -->|"invoke(cmd, {path,...})"| CMD["commands.rs #[tauri::command]"]
  CMD --> FS["fileconv::fs_util<br/>std::fs 读写"]
  FS --> ARCHIVE["archive 纯内存"]
  ARCHIVE --> WRITE["落盘源目录 + 碰撞"]
  WRITE --> RET["返回产物路径"]
  RET -->|"string / string[]"| UI["前端展示路径"]
```

**数据流:** 前端 `@tauri-apps/plugin-dialog` 取文件路径(不经 invoke 传大字节)→ invoke 传路径→command 用 `std::fs::read` 取字节(Rust 后端不受 capabilities 约束)→委托 `fileconv::fs_util` 转换落盘→返回产物路径(`String` 或 `Vec<String>`)。前端归一化数组为换行文本展示。capabilities 仅 `dialog:default`(取路径),无 fs 插件,最小权限。


