//! 格式转换模块:JSON/YAML/TOML 经 IR 互转 + CSV/Markdown/进制
//!
//! [`ir`] 为树形格式 IR(DocFormat),[`tools`] 为工具注册;本文件为公开转换函数。

mod ir;
mod tools;

pub use ir::*;
pub use tools::*;

use crate::{ToolError, ToolResult};

/// JSON → YAML
pub fn json_to_yaml(s: &str) -> ToolResult<String> {
    convert_format(&Json, &Yaml, s)
}

/// YAML → JSON(pretty)
pub fn yaml_to_json(s: &str) -> ToolResult<String> {
    convert_format(&Yaml, &Json, s)
}

/// JSON → TOML(TOML 不支持 null,遇到报错)
pub fn json_to_toml(s: &str) -> ToolResult<String> {
    convert_format(&Json, &Toml, s)
}

/// TOML → JSON(pretty)
pub fn toml_to_json(s: &str) -> ToolResult<String> {
    convert_format(&Toml, &Json, s)
}

/// JSON 数组(对象列表)→ CSV:首行表头取第一个对象的 key,后续每行对应一个对象
pub fn json_to_csv(s: &str) -> ToolResult<String> {
    json_to_delimited(s, b',')
}

/// CSV → JSON 数组(pretty):首行为表头,后续每行映射为对象(值均为字符串)
pub fn csv_to_json(s: &str) -> ToolResult<String> {
    delimited_to_json(s, b',')
}

/// JSON 数组 → TSV(制表符分隔)
pub fn json_to_tsv(s: &str) -> ToolResult<String> {
    json_to_delimited(s, b'\t')
}

/// TSV → JSON 数组(pretty)
pub fn tsv_to_json(s: &str) -> ToolResult<String> {
    delimited_to_json(s, b'\t')
}

/// CSV → TSV:分隔符替换(经 csv crate 解析再写,正确处理引号字段)
pub fn csv_to_tsv(s: &str) -> ToolResult<String> {
    delimited_convert(s, b',', b'\t')
}

/// TSV → CSV
pub fn tsv_to_csv(s: &str) -> ToolResult<String> {
    delimited_convert(s, b'\t', b',')
}

/// CSV → YAML(经 JSON 中转)
pub fn csv_to_yaml(s: &str) -> ToolResult<String> {
    let json = csv_to_json(s)?;
    convert_format(&Json, &Yaml, &json)
}

/// YAML → CSV(经 JSON 中转)
pub fn yaml_to_csv(s: &str) -> ToolResult<String> {
    let json = convert_format(&Yaml, &Json, s)?;
    json_to_csv(&json)
}

/// CSV → XML(经 JSON 中转)
pub fn csv_to_xml(s: &str) -> ToolResult<String> {
    let json = csv_to_json(s)?;
    convert_format(&Json, &Xml, &json)
}

/// XML → CSV(经 JSON 中转)
pub fn xml_to_csv(s: &str) -> ToolResult<String> {
    let json = convert_format(&Xml, &Json, s)?;
    json_to_csv(&json)
}

/// JSON → XML
pub fn json_to_xml(s: &str) -> ToolResult<String> {
    convert_format(&Json, &Xml, s)
}

/// XML → JSON(pretty)
pub fn xml_to_json(s: &str) -> ToolResult<String> {
    convert_format(&Xml, &Json, s)
}

/// YAML → TOML(经 JSON 中转)
pub fn yaml_to_toml(s: &str) -> ToolResult<String> {
    convert_format(&Yaml, &Toml, s)
}

/// TOML → YAML(经 JSON 中转)
pub fn toml_to_yaml(s: &str) -> ToolResult<String> {
    convert_format(&Toml, &Yaml, s)
}

/// YAML → XML(经 JSON 中转)
pub fn yaml_to_xml(s: &str) -> ToolResult<String> {
    convert_format(&Yaml, &Xml, s)
}

/// XML → YAML(经 JSON 中转)
pub fn xml_to_yaml(s: &str) -> ToolResult<String> {
    convert_format(&Xml, &Yaml, s)
}

/// TOML → XML(经 JSON 中转)
pub fn toml_to_xml(s: &str) -> ToolResult<String> {
    convert_format(&Toml, &Xml, s)
}

/// XML → TOML(经 JSON 中转)
pub fn xml_to_toml(s: &str) -> ToolResult<String> {
    convert_format(&Xml, &Toml, s)
}

/// JSON 数组 → 分隔文本(delimiter 指定分隔符:b',' 为 CSV,b'\t' 为 TSV)
fn json_to_delimited(s: &str, delimiter: u8) -> ToolResult<String> {
    let value: serde_json::Value = serde_json::from_str(s)?;
    let arr = match value {
        serde_json::Value::Array(a) => a,
        _ => return Err(ToolError::InvalidInput("JSON 必须为数组".into())),
    };
    if arr.is_empty() {
        return Err(ToolError::InvalidInput("JSON 数组为空,无法确定表头".into()));
    }
    let headers: Vec<String> = match &arr[0] {
        serde_json::Value::Object(obj) => obj.keys().cloned().collect(),
        _ => return Err(ToolError::InvalidInput("数组元素必须为对象".into())),
    };
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(Vec::new());
    wtr.write_record(&headers)?;
    for item in &arr {
        let obj = match item {
            serde_json::Value::Object(o) => o,
            _ => return Err(ToolError::InvalidInput("数组元素必须为对象".into())),
        };
        let row: Vec<String> = headers
            .iter()
            .map(|h| obj.get(h).map(json_value_to_csv_cell).unwrap_or_default())
            .collect();
        wtr.write_record(&row)?;
    }
    let bytes = wtr
        .into_inner()
        .map_err(|e| ToolError::Other(e.to_string()))?;
    Ok(String::from_utf8(bytes)?)
}

/// 分隔文本 → JSON 数组(delimiter 指定分隔符)
fn delimited_to_json(s: &str, delimiter: u8) -> ToolResult<String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_reader(s.as_bytes());
    let headers = reader.headers()?.clone();
    let mut arr = Vec::new();
    for result in reader.records() {
        let record = result?;
        let mut obj = serde_json::Map::new();
        for (i, header) in headers.iter().enumerate() {
            let val = record.get(i).unwrap_or("").to_string();
            obj.insert(header.to_string(), serde_json::Value::String(val));
        }
        arr.push(serde_json::Value::Object(obj));
    }
    Ok(serde_json::to_string_pretty(&arr)?)
}

/// 分隔文本互转(from → to 分隔符,经 csv crate 解析再写,正确处理引号)
fn delimited_convert(s: &str, from: u8, to: u8) -> ToolResult<String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(from)
        .from_reader(s.as_bytes());
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(to)
        .from_writer(Vec::new());
    let headers = reader.byte_headers()?.clone();
    wtr.write_byte_record(&headers)?;
    for result in reader.byte_records() {
        let r = result?;
        wtr.write_byte_record(&r)?;
    }
    let bytes = wtr
        .into_inner()
        .map_err(|e| ToolError::Other(e.to_string()))?;
    Ok(String::from_utf8(bytes)?)
}

/// Markdown → HTML:pulldown-cmark 解析,html::push_html 输出 HTML 字符串
pub fn md_to_html(s: &str) -> ToolResult<String> {
    let parser = pulldown_cmark::Parser::new(s);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    Ok(html_output)
}

/// Markdown → 纯文本:pulldown-cmark 解析后提取 Text/Code,块级元素结束补换行
pub fn md_to_txt(s: &str) -> ToolResult<String> {
    use pulldown_cmark::{Event, TagEnd};

    let parser = pulldown_cmark::Parser::new(s);
    let mut out = String::new();
    for event in parser {
        match event {
            Event::Text(t) => out.push_str(&t),
            Event::Code(c) => out.push_str(&c),
            Event::SoftBreak | Event::HardBreak => out.push('\n'),
            Event::Rule => out.push_str("\n---\n"),
            Event::End(TagEnd::Paragraph) => out.push_str("\n\n"),
            Event::End(TagEnd::Heading(_))
            | Event::End(TagEnd::CodeBlock)
            | Event::End(TagEnd::Item) => out.push('\n'),
            _ => {}
        }
    }
    Ok(out.trim().to_string())
}

/// 进制转换:按 from 进制解析字符串为 u128,输出 to 进制字符串;from/to ∈ 2..=36
pub fn numbase_convert(s: &str, from: u32, to: u32) -> ToolResult<String> {
    if !(2..=36).contains(&from) {
        return Err(ToolError::InvalidInput(format!(
            "源进制必须在 2..=36 范围内,得到 {from}"
        )));
    }
    if !(2..=36).contains(&to) {
        return Err(ToolError::InvalidInput(format!(
            "目标进制必须在 2..=36 范围内,得到 {to}"
        )));
    }
    let n: u128 = u128::from_str_radix(s, from)
        .map_err(|e| ToolError::InvalidInput(format!("进制解析失败({from} 进制):{e}")))?;
    Ok(to_base(n, to))
}

/// JSON 值转 CSV 单元格:字符串取原值,null 为空串,其余取 to_string
fn json_value_to_csv_cell(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        _ => v.to_string(),
    }
}

/// u128 转指定进制字符串(小写字母);0 返回 "0"
fn to_base(mut n: u128, base: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let radix = base as u128;
    let mut buf = Vec::new();
    while n > 0 {
        let rem = (n % radix) as u32;
        buf.push(char::from_digit(rem, base).expect("余数必在进制范围内"));
        n /= radix;
    }
    buf.reverse();
    buf.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- json_to_yaml ----

    #[test]
    fn json_to_yaml_basic() {
        let yaml = json_to_yaml(r#"{"a":1,"b":[2,3]}"#).unwrap();
        assert!(yaml.contains("a: 1"));
        assert!(yaml.contains("b:"));
    }

    #[test]
    fn json_to_yaml_nested() {
        let yaml = json_to_yaml(r#"{"x":{"y":true}}"#).unwrap();
        assert!(yaml.contains("y: true"));
    }

    #[test]
    fn json_to_yaml_invalid() {
        assert!(json_to_yaml("{").is_err());
        assert!(json_to_yaml("not json").is_err());
    }

    #[test]
    fn json_yaml_roundtrip() {
        let json = r#"{"a":1,"b":"hello","c":[1,2,3]}"#;
        let yaml = json_to_yaml(json).unwrap();
        let json2 = yaml_to_json(&yaml).unwrap();
        let v1: serde_json::Value = serde_json::from_str(json).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&json2).unwrap();
        assert_eq!(v1, v2);
    }

    // ---- yaml_to_json ----

    #[test]
    fn yaml_to_json_basic() {
        let json = yaml_to_json("a: 1\nb: hello").unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"], "hello");
    }

    #[test]
    fn yaml_to_json_array() {
        let json = yaml_to_json("- 1\n- 2\n- 3").unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v[0], 1);
        assert_eq!(v[2], 3);
    }

    #[test]
    fn yaml_to_json_invalid() {
        assert!(yaml_to_json("{a").is_err());
        assert!(yaml_to_json("[1, 2").is_err());
    }

    // ---- json_to_toml ----

    #[test]
    fn json_to_toml_basic() {
        let toml = json_to_toml(r#"{"a":1,"b":"hello"}"#).unwrap();
        assert!(toml.contains("a = 1"));
        assert!(toml.contains("b = \"hello\""));
    }

    #[test]
    fn json_to_toml_nested() {
        let toml = json_to_toml(r#"{"outer":{"inner":42}}"#).unwrap();
        assert!(toml.contains("[outer]"));
        assert!(toml.contains("inner = 42"));
    }

    #[test]
    fn json_to_toml_invalid_json() {
        assert!(json_to_toml("{").is_err());
        assert!(json_to_toml("not json").is_err());
    }

    #[test]
    fn json_to_toml_null_rejected() {
        assert!(json_to_toml(r#"{"a":null}"#).is_err());
        assert!(json_to_toml(r#"{"a":[1,null,3]}"#).is_err());
        assert!(json_to_toml(r#"{"a":{"b":null}}"#).is_err());
    }

    // ---- toml_to_json ----

    #[test]
    fn toml_to_json_basic() {
        let json = toml_to_json("a = 1\nb = \"hello\"").unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"], "hello");
    }

    #[test]
    fn toml_to_json_table() {
        let json = toml_to_json("[section]\nkey = 42").unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["section"]["key"], 42);
    }

    #[test]
    fn toml_to_json_invalid() {
        assert!(toml_to_json("a = ").is_err());
        assert!(toml_to_json("[unclosed").is_err());
    }

    #[test]
    fn json_toml_roundtrip() {
        let json = r#"{"a":1,"b":"hello","c":true}"#;
        let toml = json_to_toml(json).unwrap();
        let json2 = toml_to_json(&toml).unwrap();
        let v1: serde_json::Value = serde_json::from_str(json).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&json2).unwrap();
        assert_eq!(v1, v2);
    }

    // ---- json_to_csv ----

    #[test]
    fn json_to_csv_basic() {
        let csv = json_to_csv(r#"[{"a":1,"b":2},{"a":3,"b":4}]"#).unwrap();
        assert!(csv.contains("a,b"));
        assert!(csv.contains("1,2"));
        assert!(csv.contains("3,4"));
    }

    #[test]
    fn json_to_csv_with_string_values() {
        let csv = json_to_csv(r#"[{"name":"Alice","age":30}]"#).unwrap();
        assert!(csv.contains("Alice"));
        assert!(csv.contains("30"));
    }

    #[test]
    fn json_to_csv_non_array() {
        assert!(json_to_csv(r#"{"a":1}"#).is_err());
        assert!(json_to_csv(r#""hello""#).is_err());
    }

    #[test]
    fn json_to_csv_empty_array() {
        assert!(json_to_csv("[]").is_err());
    }

    #[test]
    fn json_to_csv_non_object_element() {
        assert!(json_to_csv("[1,2,3]").is_err());
        assert!(json_to_csv(r#"[{"a":1},"hello"]"#).is_err());
    }

    // ---- csv_to_json ----

    #[test]
    fn csv_to_json_basic() {
        let json = csv_to_json("a,b\n1,2\n3,4").unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v.as_array().unwrap().len(), 2);
        assert_eq!(v[0]["a"], "1");
        assert_eq!(v[0]["b"], "2");
        assert_eq!(v[1]["a"], "3");
        assert_eq!(v[1]["b"], "4");
    }

    #[test]
    fn csv_to_json_single_row() {
        let json = csv_to_json("a,b\n1,2").unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v.as_array().unwrap().len(), 1);
        assert_eq!(v[0]["a"], "1");
    }

    #[test]
    fn csv_to_json_header_only() {
        let json = csv_to_json("a,b").unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v.as_array().unwrap().len(), 0);
    }

    #[test]
    fn csv_to_json_quoted_fields() {
        let json = csv_to_json("name,desc\nAlice,\"hello, world\"").unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v[0]["name"], "Alice");
        assert_eq!(v[0]["desc"], "hello, world");
    }

    // ---- md_to_html ----

    #[test]
    fn md_to_html_heading() {
        let html = md_to_html("# Hello").unwrap();
        assert!(html.contains("<h1>Hello</h1>"));
    }

    #[test]
    fn md_to_html_bold() {
        let html = md_to_html("**bold**").unwrap();
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn md_to_html_list() {
        let html = md_to_html("- a\n- b").unwrap();
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>a</li>"));
        assert!(html.contains("<li>b</li>"));
    }

    #[test]
    fn md_to_html_empty() {
        let html = md_to_html("").unwrap();
        assert!(html.is_empty() || html.trim().is_empty());
    }

    #[test]
    fn md_to_html_paragraph() {
        let html = md_to_html("hello world").unwrap();
        assert!(html.contains("<p>hello world</p>"));
    }

    // ---- numbase_convert ----

    #[test]
    fn numbase_basic() {
        assert_eq!(numbase_convert("ff", 16, 10).unwrap(), "255");
        assert_eq!(numbase_convert("255", 10, 16).unwrap(), "ff");
        assert_eq!(numbase_convert("12", 3, 10).unwrap(), "5");
    }

    #[test]
    fn numbase_zero() {
        assert_eq!(numbase_convert("0", 10, 2).unwrap(), "0");
        assert_eq!(numbase_convert("0", 2, 16).unwrap(), "0");
    }

    #[test]
    fn numbase_uppercase_input() {
        assert_eq!(numbase_convert("FF", 16, 10).unwrap(), "255");
        assert_eq!(numbase_convert("FF", 16, 2).unwrap(), "11111111");
    }

    #[test]
    fn numbase_max_u128() {
        let hex_max = format!("{:x}", u128::MAX);
        let dec_max = u128::MAX.to_string();
        assert_eq!(numbase_convert(&hex_max, 16, 10).unwrap(), dec_max);
        // roundtrip
        assert_eq!(numbase_convert(&dec_max, 10, 16).unwrap(), hex_max);
    }

    #[test]
    fn numbase_invalid_digit() {
        assert!(numbase_convert("9", 2, 10).is_err());
        assert!(numbase_convert("g", 16, 10).is_err());
        assert!(numbase_convert("xyz", 10, 2).is_err());
    }

    #[test]
    fn numbase_overflow() {
        // 40 位十六进制远超 u128(约 34 位十六进制)
        let huge = "f".repeat(40);
        assert!(numbase_convert(&huge, 16, 10).is_err());
    }

    #[test]
    fn numbase_invalid_base() {
        assert!(numbase_convert("10", 1, 10).is_err());
        assert!(numbase_convert("10", 37, 10).is_err());
        assert!(numbase_convert("10", 10, 1).is_err());
        assert!(numbase_convert("10", 10, 37).is_err());
    }

    #[test]
    fn numbase_empty_input() {
        assert!(numbase_convert("", 10, 2).is_err());
    }

    #[test]
    fn numbase_same_base() {
        assert_eq!(numbase_convert("123", 10, 10).unwrap(), "123");
        assert_eq!(numbase_convert("abc", 16, 16).unwrap(), "abc");
    }

    // ---- csv_to_tsv / tsv_to_csv ----

    #[test]
    fn csv_to_tsv_basic() {
        let tsv = csv_to_tsv("a,b\n1,2\n3,4").unwrap();
        assert!(tsv.contains("a\tb"));
        assert!(tsv.contains("1\t2"));
        assert!(tsv.contains("3\t4"));
    }

    #[test]
    fn csv_to_tsv_quoted_field() {
        let tsv = csv_to_tsv("name,desc\nAlice,\"hello, world\"").unwrap();
        assert!(tsv.contains("Alice"));
        assert!(tsv.contains("hello, world"));
        assert!(!tsv.contains('"'), "TSV 无需引号包裹");
    }

    #[test]
    fn tsv_to_csv_basic() {
        let csv = tsv_to_csv("a\tb\n1\t2").unwrap();
        assert!(csv.contains("a,b"));
        assert!(csv.contains("1,2"));
    }

    #[test]
    fn csv_tsv_roundtrip() {
        let csv = "a,b\n1,2\n3,4";
        let tsv = csv_to_tsv(csv).unwrap();
        let csv2 = tsv_to_csv(&tsv).unwrap();
        assert_eq!(csv2.trim_end(), csv);
    }

    // ---- csv_to_yaml / yaml_to_csv ----

    #[test]
    fn csv_to_yaml_basic() {
        let yaml = csv_to_yaml("a,b\n1,2\n3,4").unwrap();
        assert!(yaml.contains("a:"));
        assert!(yaml.contains("b:"));
        assert!(yaml.contains("- "));
    }

    #[test]
    fn csv_yaml_roundtrip() {
        let csv = "a,b\n1,2\n3,4";
        let yaml = csv_to_yaml(csv).unwrap();
        let csv2 = yaml_to_csv(&yaml).unwrap();
        assert_eq!(csv2.trim_end(), csv);
    }

    // ---- csv_to_xml / xml_to_csv ----

    #[test]
    fn csv_to_xml_basic() {
        let xml = csv_to_xml("a,b\n1,2").unwrap();
        assert!(xml.contains("<root>"));
        assert!(xml.contains("<a>1</a>"));
        assert!(xml.contains("<b>2</b>"));
    }

    #[test]
    fn xml_to_csv_basic() {
        let xml = "<?xml version=\"1.0\"?><root><item><a>1</a><b>2</b></item><item><a>3</a><b>4</b></item></root>";
        let csv = xml_to_csv(xml).unwrap();
        assert!(csv.contains("a,b"));
        assert!(csv.contains("1,2"));
        assert!(csv.contains("3,4"));
    }

    // ---- tsv_to_json / json_to_tsv ----

    #[test]
    fn tsv_to_json_basic() {
        let json = tsv_to_json("a\tb\n1\t2").unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v.as_array().unwrap().len(), 1);
        assert_eq!(v[0]["a"], "1");
        assert_eq!(v[0]["b"], "2");
    }

    #[test]
    fn json_to_tsv_basic() {
        let tsv = json_to_tsv(r#"[{"a":1,"b":2},{"a":3,"b":4}]"#).unwrap();
        assert!(tsv.contains("a\tb"));
        assert!(tsv.contains("1\t2"));
    }

    // ---- json_to_xml / xml_to_json ----

    #[test]
    fn json_to_xml_basic() {
        let xml = json_to_xml(r#"{"a":1,"b":"hello"}"#).unwrap();
        assert!(xml.contains("<root>"));
        assert!(xml.contains("<a>1</a>"));
        assert!(xml.contains("<b>hello</b>"));
    }

    #[test]
    fn json_to_xml_array() {
        let xml = json_to_xml(r#"[1,2,3]"#).unwrap();
        assert!(xml.contains("<item>1</item>"));
        assert!(xml.contains("<item>2</item>"));
        assert!(xml.contains("<item>3</item>"));
    }

    #[test]
    fn json_to_xml_escape() {
        let xml = json_to_xml(r#"{"a":"<x> & \"y\""}"#).unwrap();
        assert!(xml.contains("&lt;x&gt;"));
        assert!(xml.contains("&amp;"));
        assert!(xml.contains("&quot;y&quot;"));
    }

    #[test]
    fn xml_to_json_basic() {
        let json = xml_to_json("<root><a>1</a><b>hello</b></root>").unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"], "hello");
    }

    #[test]
    fn xml_to_json_array() {
        let json = xml_to_json("<root><item>1</item><item>2</item></root>").unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 2);
    }

    #[test]
    fn json_xml_roundtrip_object() {
        let json = r#"{"a":1,"b":"hello","c":true}"#;
        let xml = json_to_xml(json).unwrap();
        let json2 = xml_to_json(&xml).unwrap();
        let v1: serde_json::Value = serde_json::from_str(json).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&json2).unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn xml_to_json_invalid() {
        assert!(xml_to_json("<root><a>1</a>").is_err());
        assert!(xml_to_json("not xml").is_err());
    }

    // ---- yaml_to_toml / toml_to_yaml ----

    #[test]
    fn yaml_to_toml_basic() {
        let toml = yaml_to_toml("a: 1\nb: hello").unwrap();
        assert!(toml.contains("a = 1"));
        assert!(toml.contains("b = \"hello\""));
    }

    #[test]
    fn toml_to_yaml_basic() {
        let yaml = toml_to_yaml("a = 1\nb = \"hello\"").unwrap();
        assert!(yaml.contains("a: 1"));
        assert!(yaml.contains("b: hello"));
    }

    #[test]
    fn yaml_toml_roundtrip() {
        let yaml = "a: 1\nb: hello\nc: true";
        let toml = yaml_to_toml(yaml).unwrap();
        let yaml2 = toml_to_yaml(&toml).unwrap();
        let v1: serde_json::Value = serde_json::from_str(&yaml_to_json(yaml).unwrap()).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&yaml_to_json(&yaml2).unwrap()).unwrap();
        assert_eq!(v1, v2);
    }

    // ---- yaml_to_xml / xml_to_yaml ----

    #[test]
    fn yaml_to_xml_basic() {
        let xml = yaml_to_xml("a: 1\nb: hello").unwrap();
        assert!(xml.contains("<a>1</a>"));
        assert!(xml.contains("<b>hello</b>"));
    }

    #[test]
    fn xml_to_yaml_basic() {
        let yaml = xml_to_yaml("<root><a>1</a><b>hello</b></root>").unwrap();
        assert!(yaml.contains("a: 1"));
        assert!(yaml.contains("b: hello"));
    }

    // ---- toml_to_xml / xml_to_toml ----

    #[test]
    fn toml_to_xml_basic() {
        let xml = toml_to_xml("a = 1\nb = \"hello\"").unwrap();
        assert!(xml.contains("<a>1</a>"));
        assert!(xml.contains("<b>hello</b>"));
    }

    #[test]
    fn xml_to_toml_basic() {
        let toml = xml_to_toml("<root><a>1</a><b>hello</b></root>").unwrap();
        assert!(toml.contains("a = 1"));
        assert!(toml.contains("b = \"hello\""));
    }

    // ---- md_to_txt ----

    #[test]
    fn md_to_txt_heading() {
        assert_eq!(md_to_txt("# Hello").unwrap(), "Hello");
    }

    #[test]
    fn md_to_txt_bold_code() {
        let txt = md_to_txt("This is **bold** and `code`").unwrap();
        assert_eq!(txt, "This is bold and code");
    }

    #[test]
    fn md_to_txt_paragraph() {
        assert_eq!(md_to_txt("hello world").unwrap(), "hello world");
    }

    #[test]
    fn md_to_txt_strips_markup() {
        let txt = md_to_txt("# Title\n\nSome **bold** text").unwrap();
        assert!(!txt.contains('#'));
        assert!(!txt.contains("**"));
        assert!(txt.contains("Title"));
        assert!(txt.contains("bold"));
    }
}
