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
