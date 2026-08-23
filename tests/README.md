# NexToolkit E2E 留痕测试

> 顶层 E2E 测试目录:调 `nextool` CLI 执行真实转换,产物落盘 `output/` 供人工检查。
> Rust 集成测试在 `crates/cli/tests/`(CI 自动跑),本目录是人工留痕 + 产物检查,两者互补。

## 运行

```bash
# 前置:构建 CLI 二进制
cargo build -p nextool-cli --release

# 跑所有 E2E
bash tests/run.sh
```

脚本自动探测 `target/release/nextool`(回退 `target/debug/nextool`),清空旧产物后重跑全部测试。

## 产物

```
tests/output/
  encode/          # 文本工具产物(base64/url/hex/jwt 等)
  convert/         # 转换/格式化/生成器/文本/加密产物
  fileconv/        # 文件转换产物(归档/图像/SVG/PDF/DOCX/EPUB 等)
  report.md        # 测试报告(命令 + 产物路径 + ✓/✗)
```

产物目录 gitignore(不入库),每次运行重新生成。打开 `output/report.md` 查看哪些测试通过/失败,到 `output/<category>/` 检查具体产物。

## 覆盖范围

- **文本工具**:encode(base64/url/hex/jwt/charset)、convert(json-yaml/json-toml/json-csv/md-html/numbase/unit)、format(json/sql)、generate(hash/uuid/password/qr)、text(case/sort/dedup/regex)、crypto(aes)
- **文件转换(纯 Rust)**:归档(zip/tar/list/extract)、图像(convert/resize)、SVG→PNG、PDF(拆分/旋转/加密/解密/删除页/提取页/元数据/页码/合并)、XLSX↔JSON、DOCX→TXT
- **文件转换(引擎)**:MD→DOCX(pandoc)、DOCX→PDF(LibreOffice)、HTML→EPUB(calibre)、PDF 压缩(Ghostscript)— 引擎未装时自动跳过

## 引擎依赖

引擎测试在运行时探测,未装的引擎自动跳过(不计为失败)。如需跑全部引擎测试,确保以下引擎在 PATH:

- pandoc(MD→DOCX/HTML/PDF)
- soffice / LibreOffice(DOCX→PDF)
- ebook-convert / calibre(HTML→EPUB)
- gswin64c / Ghostscript(PDF 压缩)

## fixtures/

输入素材(小文本文件,入库):
- `sample.md` — Markdown(含中文)
- `sample-ascii.md` — ASCII Markdown(pandoc→PDF 需 pdflatex,不支持 CJK)
- `sample.json` / `sample.csv` / `sample.html` / `sample.txt` — 结构化数据

二进制 fixture(PNG/PDF/DOCX)由 `run.sh` 从文本 fixture 现场生成,不入库。
