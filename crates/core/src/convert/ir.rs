//! 转换 IR:树形文档格式经 serde_json::Value 中转互转
//!
//! JSON/YAML/TOML 共享同一中间表示,每种格式只需实现 parse/render 一次,
//! 任意两格式互转由 [`convert_format`] 统一分发(O(N) 实现而非 O(N²) 手写 pair)。

use crate::{ToolError, ToolResult};

/// 树形文档格式:经 serde_json::Value 中转互转
pub trait DocFormat {
    fn parse(&self, s: &str) -> ToolResult<serde_json::Value>;
    fn render(&self, v: &serde_json::Value) -> ToolResult<String>;
}

pub struct Json;
pub struct Yaml;
pub struct Toml;

impl DocFormat for Json {
    fn parse(&self, s: &str) -> ToolResult<serde_json::Value> {
        Ok(serde_json::from_str(s)?)
    }
    fn render(&self, v: &serde_json::Value) -> ToolResult<String> {
        Ok(serde_json::to_string_pretty(v)?)
    }
}

impl DocFormat for Yaml {
    fn parse(&self, s: &str) -> ToolResult<serde_json::Value> {
        Ok(serde_yaml::from_str(s)?)
    }
    fn render(&self, v: &serde_json::Value) -> ToolResult<String> {
        Ok(serde_yaml::to_string(v)?)
    }
}

impl DocFormat for Toml {
    fn parse(&self, s: &str) -> ToolResult<serde_json::Value> {
        let v: toml::Value = toml::from_str(s)?;
        Ok(serde_json::to_value(&v)?)
    }
    fn render(&self, v: &serde_json::Value) -> ToolResult<String> {
        check_no_null(v)?;
        let toml_value = toml::Value::try_from(v).map_err(|e| ToolError::Other(e.to_string()))?;
        toml::to_string(&toml_value).map_err(|e| ToolError::Other(e.to_string()))
    }
}

/// 通用转换:parse 源格式为 Value → render 为目标格式
pub fn convert_format(from: &dyn DocFormat, to: &dyn DocFormat, input: &str) -> ToolResult<String> {
    let value = from.parse(input)?;
    to.render(&value)
}

/// 递归检查 JSON 值中是否含 null;TOML 不支持 null,遇到返回 InvalidInput
fn check_no_null(v: &serde_json::Value) -> ToolResult<()> {
    match v {
        serde_json::Value::Null => Err(ToolError::InvalidInput("TOML 不支持 null 值".into())),
        serde_json::Value::Array(arr) => {
            for item in arr {
                check_no_null(item)?;
            }
            Ok(())
        }
        serde_json::Value::Object(obj) => {
            for (_, val) in obj {
                check_no_null(val)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

// ---- XML 格式(经 JSON Value 中转,简单约定)----
//
// 约定:顶层包 `<root>`;对象 key → 子元素名;数组 → 重复 `<item>` 子元素;
// 标量 → 元素文本(parse 时尝试还原 number/bool/null,否则字符串)。
// 不支持 XML 属性与命名空间;roundtrip 对纯标量/对象/数组保持值类型。

pub struct Xml;

impl DocFormat for Xml {
    fn parse(&self, s: &str) -> ToolResult<serde_json::Value> {
        xml_to_value(s)
    }
    fn render(&self, v: &serde_json::Value) -> ToolResult<String> {
        Ok(json_to_xml(v))
    }
}

/// XML → Value:跳过前置声明,栈式解析根元素内容
fn xml_to_value(s: &str) -> ToolResult<serde_json::Value> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    enum Frame {
        Root {
            value: Option<serde_json::Value>,
        },
        Element {
            name: String,
            children: Vec<(String, serde_json::Value)>,
            text: String,
        },
    }

    let mut reader = Reader::from_str(s);
    reader.config_mut().check_end_names = true;
    let mut buf = Vec::new();
    let mut stack: Vec<Frame> = vec![Frame::Root { value: None }];

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                stack.push(Frame::Element {
                    name,
                    children: Vec::new(),
                    text: String::new(),
                });
            }
            Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                match stack.last_mut() {
                    Some(Frame::Element { children, .. }) => {
                        children.push((name, serde_json::Value::Null));
                    }
                    Some(Frame::Root { value }) => *value = Some(serde_json::Value::Null),
                    None => return Err(ToolError::Other("XML 栈异常".into())),
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().map_err(|x| ToolError::Other(x.to_string()))?;
                if let Some(Frame::Element { text: t, .. }) = stack.last_mut() {
                    t.push_str(&text);
                }
            }
            Ok(Event::CData(e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).into_owned();
                if let Some(Frame::Element { text: t, .. }) = stack.last_mut() {
                    t.push_str(&text);
                }
            }
            Ok(Event::End(_)) => match stack.pop() {
                Some(Frame::Element {
                    name,
                    children,
                    text,
                }) => {
                    let value = compute_xml_value(children, text);
                    match stack.last_mut() {
                        Some(Frame::Element { children: pc, .. }) => pc.push((name, value)),
                        Some(Frame::Root { value: rv }) => *rv = Some(value),
                        None => return Err(ToolError::Other("XML 栈异常".into())),
                    }
                }
                Some(Frame::Root { .. }) => {}
                None => return Err(ToolError::Other("XML 栈异常".into())),
            },
            Ok(Event::Eof) => break,
            Ok(_) => continue,
            Err(e) => return Err(ToolError::Other(e.to_string())),
        }
    }

    match stack.pop() {
        Some(Frame::Root { value: Some(v) }) => Ok(v),
        _ => Err(ToolError::InvalidInput("XML 无根元素".into())),
    }
}

/// 元素内容 → Value:有子元素则对象/数组,否则文本标量
fn compute_xml_value(
    children: Vec<(String, serde_json::Value)>,
    text: String,
) -> serde_json::Value {
    if !children.is_empty() {
        if children.iter().all(|(k, _)| k == "item") {
            serde_json::Value::Array(children.into_iter().map(|(_, v)| v).collect())
        } else {
            let mut map = serde_json::Map::new();
            for (k, v) in children {
                match map.remove(&k) {
                    Some(serde_json::Value::Array(mut arr)) => {
                        arr.push(v);
                        map.insert(k, serde_json::Value::Array(arr));
                    }
                    Some(existing) => {
                        map.insert(k, serde_json::Value::Array(vec![existing, v]));
                    }
                    None => {
                        map.insert(k, v);
                    }
                }
            }
            serde_json::Value::Object(map)
        }
    } else {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            serde_json::Value::Null
        } else {
            parse_xml_scalar(trimmed)
        }
    }
}

/// 文本标量尝试还原为 number/bool/null,失败保持字符串
fn parse_xml_scalar(s: &str) -> serde_json::Value {
    if s == "true" {
        serde_json::Value::Bool(true)
    } else if s == "false" {
        serde_json::Value::Bool(false)
    } else if s == "null" {
        serde_json::Value::Null
    } else if let Ok(n) = s.parse::<i64>() {
        serde_json::Value::Number(serde_json::Number::from(n))
    } else if let Ok(f) = s.parse::<f64>() {
        serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or_else(|| serde_json::Value::String(s.into()))
    } else {
        serde_json::Value::String(s.into())
    }
}

/// Value → XML 字符串(顶层包 `<root>`,声明在前)
fn json_to_xml(v: &serde_json::Value) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<root>");
    render_xml_value(v, &mut out);
    out.push_str("</root>");
    out
}

fn render_xml_value(v: &serde_json::Value, out: &mut String) {
    match v {
        serde_json::Value::Null => {}
        serde_json::Value::String(s) => escape_xml_text(s, out),
        serde_json::Value::Number(n) => out.push_str(&n.to_string()),
        serde_json::Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        serde_json::Value::Array(arr) => {
            for item in arr {
                out.push_str("<item>");
                render_xml_value(item, out);
                out.push_str("</item>");
            }
        }
        serde_json::Value::Object(obj) => {
            for (k, val) in obj {
                let name = escape_xml_name(k);
                out.push('<');
                out.push_str(&name);
                out.push('>');
                render_xml_value(val, out);
                out.push_str("</");
                out.push_str(&name);
                out.push('>');
            }
        }
    }
}

fn escape_xml_text(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
}

/// JSON key → 合法 XML 元素名(非法字符替 `_`,空名或数字开头补 `_`)
fn escape_xml_name(key: &str) -> String {
    let mut out = String::new();
    for (i, c) in key.chars().enumerate() {
        let valid = if i == 0 {
            c.is_ascii_alphabetic() || c == '_'
        } else {
            c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'
        };
        out.push(if valid { c } else { '_' });
    }
    if out.is_empty() {
        out.push('_');
    }
    out
}
