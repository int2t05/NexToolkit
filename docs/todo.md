# 待办

当前系统不足与未来方向。

## 当前不足

### 文本工具

- **编码**:零宽字符隐写对非 ASCII 支持有限;JWT 仅 HS256/RS256,缺 EC。
- **转换**:CSV/TSV 大文件流式处理未实现;单位换算缺货币(需实时汇率,违背纯本地)。
- **加密**:RSA 密钥对仅 PKCS#1 输出,缺 PKCS#8;无 PGP 兼容。

### 文件转换

- **PDF**:加密用 RC4-128(lopdf 限制),非 AES-256;解密检测加密标志有已知限制。
- **图像**:DDS 仅解码;RAW/HEIC 需 C 绑定(非目标)。
- **归档**:7z 创建未实现(仅解压);RAR 创建非目标(专有格式)。
- **字体**:仅 TTF↔WOFF,缺 OTF→WOFF 独立路径(OTF 经 TTF 处理)。

### 引擎局限

- **便携版覆盖**:仅 ffmpeg/pandoc 有便携版自动安装;LibreOffice/calibre/Ghostscript/tesseract 需手动下载安装。
- **macOS 便携版**:未实现(macOS 引擎多走 brew)。
- **LibreOffice 首次启动**:headless 冷启动慢(数秒),未实现常驻进程优化。

### GUI

- **状态管理**:`state.svelte.ts` 单例承载全部状态,接近单文件上限,可拆分。
- **大输出性能**:highlight.js 大文件高亮卡顿,未实现虚拟滚动。
- **i18n**:仅中英,缺日韩。

## 未来方向

### 智能层

- **Smart Detection**:剪贴板自动识别内容(JSON/URL/base64/PDF 路径)并推荐工具。
- **Recipe 流水线**:工具链式组合(如 base64 解码 → JSON 格式化 → 保存文件)。

### 格式扩展

- **PDF AES-256 加密**:替换 lopdf 的 RC4-128,或集成 Ghostscript 做加密。
- **Office 互转**:经 LibreOffice `--convert-to` 覆盖 doc↔docx/xls↔xlsx 等互转对。
- **图像处理**:批量转换(目录递归)+ 更多滤镜(锐化/边缘检测)。

### 引擎便携版

- **calibre 便携版**:评估 calibre-portable 可行性(当前仅安装包)。
- **tesseract 便携版**:评估 UB-Mannheim zip 解压可行性。
- **LibreOffice 常驻进程**:`--accept` socket 模式消除冷启动开销。

### i18n

- 日韩支持;错误消息本地化(当前混合中英)。

### 生态

- **插件系统**:用户自定义工具注册(经 Lua/WASM 脚本)。
- **CI 增强**:macOS/Linux 引擎测试矩阵(当前仅 Windows 本地验证)。
