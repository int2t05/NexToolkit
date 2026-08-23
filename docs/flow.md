# 业务流程

CLI 与 GUI 共享 `nextool-core`,数据流以 core 为中心。

## CLI 管道

```mermaid
flowchart LR
  ARG["参数 / stdin"] --> IO["read_input"]
  IO --> CLAP["clap 子命令"]
  CLAP --> CORE["core 函数"]
  CORE -->|"Ok"| OUT["stdout · exit 0"]
  CORE -->|"Err"| ERR["stderr · exit 1"]
```

`read_input` 优先取位置参数,缺省读 stdin。子命令调 core 同名函数,`Ok` 打印 stdout 退出 0,`Err` 经 `to_string()` 进 stderr 退出 1。

## GUI invoke

```mermaid
flowchart LR
  MOUNT["onMount<br/>list_tools()"] --> META["ToolMetaDto[]"]
  META --> UI["渲染表单"]
  UI -->|"文本工具"| RT["invoke('run_tool',<br/>{id, input, args})"]
  UI -->|"文件工具"| FC["invoke(cmd, {path,...})"]
  RT --> CORE["core 工具"]
  FC --> CORE
  CORE -->|"Ok"| OUT["输出区<br/>按 output_kind 渲染"]
  CORE -->|"Err"| CE["CmdError → reject"]
```

前端 `onMount` 拉取元数据动态渲染。文本工具经 `run_tool` 通用入口,文件工具签名各异保留独立 command。`output_kind` 决定渲染:高亮/SVG/纯文本。`Err` 经 `CmdError` 序列化 reject,前端 catch 展示。

## 通用文件转换路由

```mermaid
flowchart TD
  INPUT["convert_file(input, target)"] --> DETECT["detect_family(input_ext)"]
  DETECT -->|"Office"| LO["LibreOffice<br/>--convert-to target"]
  DETECT -->|"Markup"| PAN["pandoc -o output"]
  DETECT -->|"Ebook"| CAL["calibre input output"]
  DETECT -->|"PDF"| PDF{"target?"}
  PDF -->|"pdf"| GS["Ghostscript 压缩"]
  PDF -->|"txt"| RUST["纯 Rust lopdf 提取"]
  PDF -->|"其他"| LO2["LibreOffice"]
  DETECT -->|"Image/Archive/Font/SVG"| REJ["InvalidInput: 用专项命令"]
```

`convert_any` 按源文件扩展名判定 `FormatFamily`,路由到对应引擎或纯 Rust 函数。同扩展名用 `_converted` 后缀避免覆盖源。Image/Archive/Font/Svg 族返回错误提示用专项命令。

## 引擎安装流程

```mermaid
flowchart TD
  CHECK["resolve_binary()"] -->|"PATH/安装目录/常见路径"| FOUND["已装"]
  CHECK -->|"未找到"| PORT{"is_portable?"}
  PORT -->|"是"| INSTALL["install_engine()<br/>下载 zip → 解压到 install_dir"]
  PORT -->|"否"| MANUAL["提示下载安装包<br/>用户手动装"]
  INSTALL --> DETECT2["resolve_binary() 再探测"]
  DETECT2 --> FOUND
```

便携版(ffmpeg/pandoc)自动下载解压到 `%APPDATA%/NexToolkit/engines/<engine>/`。安装包引擎(LibreOffice/calibre/Ghostscript/tesseract)提供下载链接,用户手动安装后重启应用。`resolve_binary` 查找优先级:PATH → 安装目录 → Windows 常见路径。

## 主题切换

```mermaid
flowchart LR
  BTN["TopBar 切换按钮"] --> STATE["state.theme<br/>light/dark/auto"]
  STATE --> APPLY["applyTheme()<br/>document.dataset.theme"]
  APPLY --> CSS["tokens.css<br/>:root[data-theme] 变量"]
  STATE --> LS["localStorage 持久化"]
```

三态切换:`auto` 跟随系统(`@media prefers-color-scheme`)、`light` 强制明色、`dark` 强制暗色。`localStorage` 持久化用户选择。

## AES-256-GCM 加解密

```mermaid
flowchart TD
  subgraph 加密["aes_gcm_encrypt(plaintext, password)"]
    R["OsRng salt(16B) + nonce(12B)"] --> K["PBKDF2 100k 轮 → 32B key"]
    K --> E["Aes256Gcm 加密"]
    E --> B["salt‖nonce‖ciphertext‖tag → base64"]
  end
  subgraph 解密["aes_gcm_decrypt(b64, password)"]
    D["base64 解码"] --> S["切出 salt/nonce/ciphertext"]
    S --> K2["PBKDF2 派生 key"]
    K2 --> D2["Aes256Gcm 认证解密"]
  end
```

随机 salt+nonce 使同明文产生不同密文。密文自包含 salt/nonce,解密只需口令。认证失败返回 `InvalidInput`。

## RSA 密钥与加解密

```mermaid
flowchart TD
  KG["rsa_keygen(bits)"] --> GEN["RsaPrivateKey::new"]
  GEN --> PEM["PKCS#1 PEM: 私钥 + 公钥"]
  ENC["rsa_encrypt(input, pub_pem)"] --> OE["OAEP-SHA256 加密 → base64"]
  DEC["rsa_decrypt(b64, priv_pem)"] --> OD["OAEP-SHA256 解密"]
  SIGN["rsa_sign(input, priv_pem)"] --> SG["PKCS1v15-SHA256 签名 → base64"]
  VERIFY["rsa_verify(input, pub_pem, sig)"] --> VF["验签 → 成功/InvalidInput"]
```

密钥对输出 PKCS#1 PEM。加解密用 OAEP-SHA256,签名用 PKCS1v15-SHA256。PEM 解析兼容 PKCS#8/SPKI 与 PKCS#1。

## 错误传播

```mermaid
flowchart LR
  CORE["core 返回 ToolError"] -->|"CLI"| CLIERR["to_string() → stderr · exit 1"]
  CORE -->|"GUI"| FROM["From<ToolError> → CmdError"]
  FROM --> SER["serde 序列化"]
  SER --> REJ["invoke reject → 前端 catch"]
```

core 统一 `ToolError`(12 变体)。CLI 直接 `to_string` 进 stderr 非零退出;GUI 经 `From` 转字符串序列化 reject。两入口错误信息一致。
