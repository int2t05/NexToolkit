//! 格式化模块:JSON/SQL/XML/CSS 美化与压缩

use crate::{ToolError, ToolResult};

/// JSON 美化:解析后以 2 空格缩进重新序列化
pub fn json_format(s: &str) -> ToolResult<String> {
    let value: serde_json::Value = serde_json::from_str(s)?;
    Ok(serde_json::to_string_pretty(&value)?)
}

/// JSON 压缩:解析后以紧凑形式重新序列化,去除多余空白
pub fn json_minify(s: &str) -> ToolResult<String> {
    let value: serde_json::Value = serde_json::from_str(s)?;
    Ok(serde_json::to_string(&value)?)
}

/// SQL 美化:关键字大写、2 空格缩进;空输入返回 EmptyInput
pub fn sql_format(s: &str) -> ToolResult<String> {
    if s.trim().is_empty() {
        return Err(ToolError::EmptyInput);
    }
    let options = sqlformat::FormatOptions {
        indent: sqlformat::Indent::Spaces(2),
        uppercase: Some(true),
        ..Default::default()
    };
    Ok(sqlformat::format(
        s,
        &sqlformat::QueryParams::None,
        &options,
    ))
}

/// XML 美化:quick-xml 解析后按 2 空格缩进重输出,空白文本节点忽略
pub fn xml_format(s: &str) -> ToolResult<String> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(s);
    reader.config_mut().check_end_names = true;
    let mut buf = Vec::new();
    let mut out = String::new();
    let mut depth: usize = 0;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = utf8(e.name().into_inner())?;
                push_indent(&mut out, depth);
                out.push('<');
                out.push_str(name);
                write_attrs(&mut out, e.attributes())?;
                out.push_str(">\n");
                depth += 1;
            }
            Ok(Event::End(e)) => {
                depth = depth.saturating_sub(1);
                let name = utf8(e.name().into_inner())?;
                push_indent(&mut out, depth);
                out.push_str("</");
                out.push_str(name);
                out.push_str(">\n");
            }
            Ok(Event::Empty(e)) => {
                let name = utf8(e.name().into_inner())?;
                push_indent(&mut out, depth);
                out.push('<');
                out.push_str(name);
                write_attrs(&mut out, e.attributes())?;
                out.push_str("/>\n");
            }
            Ok(Event::Text(e)) => {
                let text = utf8(&e)?;
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    continue;
                }
                push_indent(&mut out, depth);
                out.push_str(trimmed);
                out.push('\n');
            }
            Ok(Event::Comment(e)) => {
                let text = utf8(&e)?;
                push_indent(&mut out, depth);
                out.push_str("<!--");
                out.push_str(text);
                out.push_str("-->\n");
            }
            Ok(Event::CData(e)) => {
                let text = utf8(&e)?;
                push_indent(&mut out, depth);
                out.push_str("<![CDATA[");
                out.push_str(text);
                out.push_str("]]>\n");
            }
            Ok(Event::Decl(d)) => {
                let text = utf8(&d)?;
                out.push_str("<?");
                out.push_str(text);
                out.push_str("?>\n");
            }
            Ok(Event::PI(p)) => {
                let text = utf8(&p)?;
                out.push_str("<?");
                out.push_str(text);
                out.push_str("?>\n");
            }
            Ok(Event::DocType(e)) => {
                let text = utf8(&e)?;
                push_indent(&mut out, depth);
                out.push_str("<!DOCTYPE ");
                out.push_str(text.trim());
                out.push_str(">\n");
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ToolError::Other(e.to_string())),
        }
    }
    if out.ends_with('\n') {
        out.pop();
    }
    Ok(out)
}

/// XML 压缩:去除标签间空白与换行,保留有效文本内容
pub fn xml_minify(s: &str) -> ToolResult<String> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(s);
    reader.config_mut().check_end_names = true;
    let mut buf = Vec::new();
    let mut out = String::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = utf8(e.name().into_inner())?;
                out.push('<');
                out.push_str(name);
                write_attrs(&mut out, e.attributes())?;
                out.push('>');
            }
            Ok(Event::End(e)) => {
                let name = utf8(e.name().into_inner())?;
                out.push_str("</");
                out.push_str(name);
                out.push('>');
            }
            Ok(Event::Empty(e)) => {
                let name = utf8(e.name().into_inner())?;
                out.push('<');
                out.push_str(name);
                write_attrs(&mut out, e.attributes())?;
                out.push_str("/>");
            }
            Ok(Event::Text(e)) => {
                let text = utf8(&e)?;
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    continue;
                }
                out.push_str(trimmed);
            }
            Ok(Event::Comment(e)) => {
                let text = utf8(&e)?;
                out.push_str("<!--");
                out.push_str(text);
                out.push_str("-->");
            }
            Ok(Event::CData(e)) => {
                let text = utf8(&e)?;
                out.push_str("<![CDATA[");
                out.push_str(text);
                out.push_str("]]>");
            }
            Ok(Event::Decl(d)) => {
                let text = utf8(&d)?;
                out.push_str("<?");
                out.push_str(text);
                out.push_str("?>");
            }
            Ok(Event::PI(p)) => {
                let text = utf8(&p)?;
                out.push_str("<?");
                out.push_str(text);
                out.push_str("?>");
            }
            Ok(Event::DocType(e)) => {
                let text = utf8(&e)?;
                out.push_str("<!DOCTYPE ");
                out.push_str(text.trim());
                out.push('>');
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ToolError::Other(e.to_string())),
        }
    }
    Ok(out)
}

/// CSS 压缩:移除注释与多余空白,保留必要分号与结构
pub fn css_minify(s: &str) -> ToolResult<String> {
    let minified = minifier::css::minify(s).map_err(|e| ToolError::Other(e.to_string()))?;
    Ok(minified.to_string())
}

/// 字节切片转 &str,UTF-8 错误归入 Other
fn utf8(bytes: &[u8]) -> ToolResult<&str> {
    std::str::from_utf8(bytes).map_err(|e| ToolError::Other(e.to_string()))
}

/// 写入 2 空格缩进(每层 depth 一组)
fn push_indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

/// 将属性列表追加为 ` key="value"` 形式,值保持原始转义
fn write_attrs(
    out: &mut String,
    attrs: quick_xml::events::attributes::Attributes<'_>,
) -> ToolResult<()> {
    for attr in attrs {
        let attr = attr.map_err(|e| ToolError::Other(e.to_string()))?;
        out.push(' ');
        out.push_str(utf8(attr.key.as_ref())?);
        out.push_str("=\"");
        out.push_str(utf8(attr.value.as_ref())?);
        out.push('"');
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- json_format ----

    #[test]
    fn json_format_basic() {
        assert_eq!(json_format("{\"a\":1}").unwrap(), "{\n  \"a\": 1\n}");
    }

    #[test]
    fn json_format_nested_indent() {
        let out = json_format("{\"a\":{\"b\":1}}").unwrap();
        assert!(out.contains("    \"b\": 1"), "嵌套层级应 4 空格缩进");
        assert!(out.contains('\n'));
    }

    #[test]
    fn json_format_scalar_and_empty_array() {
        assert_eq!(json_format("42").unwrap(), "42");
        assert_eq!(json_format("[]").unwrap(), "[]");
    }

    #[test]
    fn json_format_invalid() {
        assert!(json_format("{bad").is_err());
        assert!(json_format("").is_err());
    }

    // ---- json_minify ----

    #[test]
    fn json_minify_basic() {
        assert_eq!(json_minify("{ \"a\" : 1 }").unwrap(), "{\"a\":1}");
    }

    #[test]
    fn json_minify_array_and_empty() {
        assert_eq!(json_minify("  [ 1 , 2 ]  ").unwrap(), "[1,2]");
        assert_eq!(json_minify("[]").unwrap(), "[]");
    }

    #[test]
    fn json_minify_invalid() {
        assert!(json_minify("[1,").is_err());
    }

    // ---- sql_format ----

    #[test]
    fn sql_format_keywords_uppercase() {
        let out = sql_format("select*from t where a=1").unwrap();
        assert!(out.contains("SELECT"), "关键字应大写");
        assert!(out.contains("FROM"));
        assert!(out.contains('\n'), "应含换行");
    }

    #[test]
    fn sql_format_single_keyword() {
        let out = sql_format("select 1").unwrap();
        assert!(out.contains("SELECT"));
    }

    #[test]
    fn sql_format_empty_input() {
        assert!(sql_format("").is_err());
        assert!(sql_format("   \n  ").is_err());
    }

    // ---- xml_format ----

    #[test]
    fn xml_format_indents_nested() {
        let out = xml_format("<a><b>1</b></a>").unwrap();
        assert!(out.contains("\n  <b>"), "子元素应 2 空格缩进");
        assert!(out.ends_with("</a>"));
    }

    #[test]
    fn xml_format_self_closing() {
        let out = xml_format("<root><empty/></root>").unwrap();
        assert!(out.contains("  <empty/>"));
    }

    #[test]
    fn xml_format_empty_input() {
        assert_eq!(xml_format("").unwrap(), "");
    }

    #[test]
    fn xml_format_mismatched_tags() {
        assert!(xml_format("<a><b></a>").is_err());
    }

    // ---- xml_minify ----

    #[test]
    fn xml_minify_removes_whitespace() {
        assert_eq!(
            xml_minify("<a>\n  <b>1</b>\n</a>").unwrap(),
            "<a><b>1</b></a>"
        );
    }

    #[test]
    fn xml_minify_self_closing_and_empty() {
        assert_eq!(xml_minify("<a/>").unwrap(), "<a/>");
        assert_eq!(xml_minify("").unwrap(), "");
    }

    #[test]
    fn xml_minify_mismatched_tags() {
        assert!(xml_minify("<a><b></a>").is_err());
    }

    // ---- css_minify ----

    #[test]
    fn css_minify_basic() {
        let css = "\n    .foo > p {\n        color: red;\n    }";
        assert_eq!(css_minify(css).unwrap(), ".foo>p{color:red;}");
    }

    #[test]
    fn css_minify_strips_comments() {
        assert_eq!(
            css_minify(".x { /* note */ color: red; }").unwrap(),
            ".x{color:red;}"
        );
    }

    #[test]
    fn css_minify_unclosed_comment() {
        assert!(css_minify("/* 未闭合").is_err());
    }
}
