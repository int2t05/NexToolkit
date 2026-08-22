//! 文本提取:PDF/DOCX → 纯文本(字节域,纯内存)
//!
//! [`pdf_to_text`] 遍历 PDF 页面内容流提取 BT/ET 文本块中的字符串(简单实现,
//! 不解析 CID 字体编码与复杂布局);[`docx_to_text`] 解压 OOXML zip 提取
//! word/document.xml 的 w:t 文本节点。输入输出为 `&[u8]`/`String`,不碰文件系统。

use crate::{ToolError, ToolResult};

/// PDF → 纯文本:遍历每页内容流,提取 BT/ET 文本块中的字符串
///
/// 简单实现:仅提取 Latin 文本字符串字面量 `(...)` 与十六进制 `<...>`,
/// 不解析 CID 字体编码、字距(TJ 数组 kerning)与布局坐标。适用于简单 Latin 文本 PDF;
/// 扫描件(无文本层)或 CID 编码(中日韩)PDF 可能提取不到文本。
#[cfg(feature = "pdf")]
pub fn pdf_to_text(data: &[u8]) -> ToolResult<String> {
    let doc = lopdf::Document::load_mem(data)
        .map_err(|e| ToolError::InvalidInput(format!("PDF 加载失败: {e}")))?;
    let pages = doc.get_pages();
    if pages.is_empty() {
        return Err(ToolError::InvalidInput("PDF 无页面".into()));
    }
    let mut out = String::new();
    for page_id in pages.values() {
        let content = doc.get_page_content(*page_id);
        extract_pdf_text(&content, &mut out);
        out.push('\n');
    }
    let trimmed = out.trim();
    if trimmed.is_empty() {
        return Err(ToolError::InvalidInput(
            "PDF 未提取到文本(可能为扫描件或使用 CID 字体编码)".into(),
        ));
    }
    Ok(trimmed.to_string())
}

/// DOCX → 纯文本:解压 zip 读取 word/document.xml,提取 w:t 文本节点
#[cfg(feature = "archive")]
pub fn docx_to_text(data: &[u8]) -> ToolResult<String> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| ToolError::InvalidInput(format!("DOCX(zip) 解析失败: {e}")))?;
    let mut document_xml = String::new();
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| ToolError::Other(e.to_string()))?;
        if file.name() == "word/document.xml" {
            use std::io::Read;
            file.read_to_string(&mut document_xml)?;
            break;
        }
    }
    if document_xml.is_empty() {
        return Err(ToolError::InvalidInput("DOCX 无 word/document.xml".into()));
    }
    extract_docx_text(&document_xml)
}

// ---- PDF 内容流文本提取 ----

/// 遍历内容流:BT 进入文本模式,ET 退出;文本模式内提取 `()` 与 `<>` 字符串
#[cfg(feature = "pdf")]
fn extract_pdf_text(content: &[u8], out: &mut String) {
    let mut i = 0;
    let mut in_text = false;
    while i < content.len() {
        if is_pdf_op(content, i, b"BT") {
            in_text = true;
            i += 2;
            continue;
        }
        if is_pdf_op(content, i, b"ET") {
            in_text = false;
            i += 2;
            continue;
        }
        if in_text {
            match content[i] {
                b'(' => {
                    let (s, next) = parse_pdf_literal_string(content, i);
                    out.push_str(&s);
                    i = next;
                }
                b'<' => {
                    if i + 1 < content.len() && content[i + 1] == b'<' {
                        // 字典开始 <<,跳过(非文本)
                        i += 1;
                    } else {
                        let (s, next) = parse_pdf_hex_string(content, i);
                        out.push_str(&s);
                        i = next;
                    }
                }
                _ => i += 1,
            }
        } else {
            i += 1;
        }
    }
}

/// 判断位置 i 是否为操作符 op(前后为空白或边界)
#[cfg(feature = "pdf")]
fn is_pdf_op(content: &[u8], i: usize, op: &[u8]) -> bool {
    if i + op.len() > content.len() || &content[i..i + op.len()] != op {
        return false;
    }
    let before_ok = i == 0 || matches!(content[i - 1], b' ' | b'\n' | b'\r' | b'\t');
    let after = i + op.len();
    let after_ok = after >= content.len() || matches!(content[after], b' ' | b'\n' | b'\r' | b'\t');
    before_ok && after_ok
}

/// PDF 字符串字面量 `(...)`:处理转义与嵌套括号
#[cfg(feature = "pdf")]
fn parse_pdf_literal_string(content: &[u8], start: usize) -> (String, usize) {
    let mut out = String::new();
    let mut i = start + 1;
    let mut depth = 1;
    while i < content.len() {
        let b = content[i];
        match b {
            b'\\' => {
                if i + 1 < content.len() {
                    let n = content[i + 1];
                    match n {
                        b'n' => {
                            out.push('\n');
                            i += 2;
                        }
                        b'r' => {
                            out.push('\r');
                            i += 2;
                        }
                        b't' => {
                            out.push('\t');
                            i += 2;
                        }
                        b'\\' => {
                            out.push('\\');
                            i += 2;
                        }
                        b'(' => {
                            out.push('(');
                            i += 2;
                        }
                        b')' => {
                            out.push(')');
                            i += 2;
                        }
                        b'0'..=b'7' => {
                            let mut oct = String::new();
                            let mut j = i + 1;
                            while j < content.len()
                                && j < i + 4
                                && (b'0'..=b'7').contains(&content[j])
                            {
                                oct.push(content[j] as char);
                                j += 1;
                            }
                            if let Ok(n) = u8::from_str_radix(&oct, 8) {
                                out.push(n as char);
                            }
                            i = j;
                        }
                        _ => {
                            i += 2;
                        }
                    }
                } else {
                    i += 1;
                }
            }
            b'(' => {
                depth += 1;
                out.push('(');
                i += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    i += 1;
                    break;
                }
                out.push(')');
                i += 1;
            }
            _ => {
                out.push(b as char);
                i += 1;
            }
        }
    }
    (out, i)
}

/// PDF 十六进制字符串 `<...>`:每两位为字节,奇数补 0
#[cfg(feature = "pdf")]
fn parse_pdf_hex_string(content: &[u8], start: usize) -> (String, usize) {
    let mut hex = String::new();
    let mut i = start + 1;
    while i < content.len() && content[i] != b'>' {
        if content[i].is_ascii_hexdigit() {
            hex.push(content[i] as char);
        }
        i += 1;
    }
    if i < content.len() {
        i += 1; // 跳过 >
    }
    if hex.len() % 2 == 1 {
        hex.push('0');
    }
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap_or(0))
        .collect();
    (String::from_utf8_lossy(&bytes).into_owned(), i)
}

// ---- DOCX XML 文本提取 ----

/// 遍历 document.xml:收集 w:t 文本,w:p 结束换行,w:br 换行,w:tab 制表
#[cfg(feature = "archive")]
fn extract_docx_text(xml: &str) -> ToolResult<String> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().check_end_names = true;
    let mut buf = Vec::new();
    let mut out = String::new();
    let mut in_w_t = false;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"w:t" => in_w_t = true,
                b"w:br" => out.push('\n'),
                _ => {}
            },
            Ok(Event::Empty(e)) => match e.name().as_ref() {
                b"w:br" => out.push('\n'),
                b"w:tab" => out.push('\t'),
                _ => {}
            },
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"w:t" => in_w_t = false,
                b"w:p" => out.push('\n'),
                _ => {}
            },
            Ok(Event::Text(e)) => {
                if in_w_t {
                    let text = e.unescape().map_err(|x| ToolError::Other(x.to_string()))?;
                    out.push_str(&text);
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => continue,
            Err(e) => return Err(ToolError::Other(e.to_string())),
        }
    }
    Ok(out.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- PDF 文本提取 ----

    #[cfg(feature = "pdf")]
    fn escape_pdf_literal(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)")
    }

    /// 构造含文本内容流的单页 PDF(BT/Tj/ET)
    #[cfg(feature = "pdf")]
    fn make_text_pdf(text: &str) -> Vec<u8> {
        use lopdf::{dictionary, Object};
        use std::io::Cursor;

        let mut doc = lopdf::Document::with_version("1.4");
        let pages_id = doc.new_object_id();
        let escaped = escape_pdf_literal(text);
        let content = format!("BT /F1 12 Tf ({escaped}) Tj ET");
        let stream = lopdf::Stream::new(lopdf::Dictionary::new(), content.into_bytes());
        let content_id = doc.add_object(Object::Stream(stream));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => content_id,
        });
        let kids: Vec<Object> = vec![Object::Reference(page_id)];
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => kids,
                "Count" => 1,
            }),
        );
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        let mut buf = Vec::new();
        doc.save_to(&mut Cursor::new(&mut buf)).unwrap();
        buf
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn pdf_text_basic() {
        let data = make_text_pdf("Hello World");
        let text = pdf_to_text(&data).unwrap();
        assert!(text.contains("Hello World"), "应提取到文本: {text}");
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn pdf_text_special_chars() {
        let data = make_text_pdf("Line (with) parens");
        let text = pdf_to_text(&data).unwrap();
        assert!(text.contains("Line (with) parens"), "括号应保留: {text}");
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn pdf_text_invalid_input() {
        assert!(pdf_to_text(b"not a pdf").is_err());
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn pdf_text_no_text_layer() {
        use lopdf::{dictionary, Object};
        use std::io::Cursor;

        let mut doc = lopdf::Document::with_version("1.4");
        let pages_id = doc.new_object_id();
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        });
        let kids: Vec<Object> = vec![Object::Reference(page_id)];
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => kids,
                "Count" => 1,
            }),
        );
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        let mut buf = Vec::new();
        doc.save_to(&mut Cursor::new(&mut buf)).unwrap();
        assert!(pdf_to_text(&buf).is_err(), "无文本层应报错");
    }

    // ---- DOCX 文本提取 ----

    #[cfg(feature = "archive")]
    fn escape_xml_text(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    /// 构造含多段落的 DOCX(zip + word/document.xml)
    #[cfg(feature = "archive")]
    fn make_docx(paragraphs: &[&str]) -> Vec<u8> {
        use std::io::Write;

        let options = zip::write::SimpleFileOptions::default();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        zip.start_file("word/document.xml", options).unwrap();
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#,
        );
        for p in paragraphs {
            let escaped = escape_xml_text(p);
            xml.push_str(&format!("<w:p><w:r><w:t>{escaped}</w:t></w:r></w:p>"));
        }
        xml.push_str("</w:body></w:document>");
        zip.write_all(xml.as_bytes()).unwrap();
        let buf = zip.finish().unwrap();
        buf.into_inner()
    }

    #[cfg(feature = "archive")]
    #[test]
    fn docx_text_basic() {
        let data = make_docx(&["Hello World"]);
        let text = docx_to_text(&data).unwrap();
        assert_eq!(text, "Hello World");
    }

    #[cfg(feature = "archive")]
    #[test]
    fn docx_text_multi_paragraph() {
        let data = make_docx(&["First paragraph", "Second paragraph"]);
        let text = docx_to_text(&data).unwrap();
        assert!(text.contains("First paragraph"), "{text}");
        assert!(text.contains("Second paragraph"), "{text}");
        assert!(text.contains('\n'), "段落间应有换行");
    }

    #[cfg(feature = "archive")]
    #[test]
    fn docx_text_xml_entities() {
        let data = make_docx(&["a < b & c > d"]);
        let text = docx_to_text(&data).unwrap();
        assert_eq!(text, "a < b & c > d", "XML 实体应还原");
    }

    #[cfg(feature = "archive")]
    #[test]
    fn docx_text_invalid_input() {
        assert!(docx_to_text(b"not a zip").is_err());
    }

    #[cfg(feature = "archive")]
    #[test]
    fn docx_text_no_document_xml() {
        use std::io::Write;

        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        zip.start_file("other.txt", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"hello").unwrap();
        let buf = zip.finish().unwrap();
        assert!(
            docx_to_text(&buf.into_inner()).is_err(),
            "无 document.xml 应报错"
        );
    }
}
