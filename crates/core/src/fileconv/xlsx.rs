//! 电子表格模块:XLSX 读写与互转(字节域,纯内存)
//!
//! 基于 calamine(读 xlsx)+ rust_xlsxwriter(写 xlsx)。XLSX↔JSON 互转:
//! 读 XLSX → serde_json::Value(二维数组,首行作表头);JSON → 写 XLSX。
//! 输入输出为 `&[u8]`/`Vec<u8>`,不碰文件系统。

use crate::{ToolError, ToolResult};
use calamine::{open_workbook_auto_from_rs, Data, Reader};
use rust_xlsxwriter::Workbook;
use std::io::Cursor;

/// XLSX → JSON:读首个 sheet,输出二维数组 JSON(首行保留为数据,不特殊处理表头)
///
/// 每单元格按类型转 JSON:number/int/float/string/bool/null。空单元格为 null。
pub fn xlsx_to_json(data: &[u8]) -> ToolResult<String> {
    let cursor = Cursor::new(data.to_vec());
    let mut workbook = open_workbook_auto_from_rs(cursor)
        .map_err(|e| ToolError::Other(format!("打开 xlsx 失败: {e}")))?;
    let sheet = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| ToolError::InvalidInput("xlsx 无 sheet".into()))?;
    let range = workbook
        .worksheet_range(&sheet)
        .map_err(|e| ToolError::Other(format!("读取 sheet 失败: {e}")))?;

    let mut rows = Vec::with_capacity(range.height());
    for row in range.rows() {
        let mut cells = Vec::with_capacity(row.len());
        for cell in row {
            cells.push(cell_to_json(cell));
        }
        rows.push(serde_json::Value::Array(cells));
    }
    let value = serde_json::Value::Array(rows);
    serde_json::to_string_pretty(&value)
        .map_err(|e| ToolError::Other(format!("JSON 序列化失败: {e}")))
}

/// JSON → XLSX:输入二维数组 JSON(数组之数组,标量元素),写首个 sheet
///
/// 支持的标量:number/bool/string/null。非二维数组报错。
pub fn json_to_xlsx(data: &[u8]) -> ToolResult<Vec<u8>> {
    let value: serde_json::Value = serde_json::from_slice(data)
        .map_err(|e| ToolError::InvalidInput(format!("JSON 解析失败: {e}")))?;
    let rows = value
        .as_array()
        .ok_or_else(|| ToolError::InvalidInput("JSON 须为数组(二维)".into()))?;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    for (r, row) in rows.iter().enumerate() {
        let cells = row
            .as_array()
            .ok_or_else(|| ToolError::InvalidInput(format!("第 {r} 行非数组")))?;
        for (c, cell) in cells.iter().enumerate() {
            let row = r as u32;
            let col = c as u16;
            match cell {
                serde_json::Value::Null => {}
                serde_json::Value::Bool(b) => {
                    worksheet
                        .write_boolean(row, col, *b)
                        .map_err(|e| ToolError::Other(format!("xlsx 写入失败: {e}")))?;
                }
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        worksheet
                            .write_number(row, col, i as f64)
                            .map_err(|e| ToolError::Other(format!("xlsx 写入失败: {e}")))?;
                    } else if let Some(f) = n.as_f64() {
                        worksheet
                            .write_number(row, col, f)
                            .map_err(|e| ToolError::Other(format!("xlsx 写入失败: {e}")))?;
                    }
                }
                serde_json::Value::String(s) => {
                    worksheet
                        .write_string(row, col, s)
                        .map_err(|e| ToolError::Other(format!("xlsx 写入失败: {e}")))?;
                }
                // 嵌套对象/数组转字符串
                other => {
                    worksheet
                        .write_string(row, col, other.to_string())
                        .map_err(|e| ToolError::Other(format!("xlsx 写入失败: {e}")))?;
                }
            }
        }
    }
    workbook
        .save_to_buffer()
        .map_err(|e| ToolError::Other(format!("xlsx 写入失败: {e}")))
}

/// 单元格值 → JSON(calamine Data 枚举映射)
fn cell_to_json(cell: &Data) -> serde_json::Value {
    match cell {
        Data::Empty => serde_json::Value::Null,
        Data::String(s) => serde_json::Value::String(s.clone()),
        Data::DateTime(dt) => serde_json::Value::String(dt.to_string()),
        Data::Int(i) => serde_json::json!(i),
        Data::Float(f) => {
            // 整数值的浮点转 int JSON(避免 1.0 显示为 1.0)
            if f.fract() == 0.0 && f.is_finite() {
                serde_json::json!(*f as i64)
            } else {
                serde_json::json!(f)
            }
        }
        Data::Bool(b) => serde_json::Value::Bool(*b),
        Data::Error(e) => serde_json::Value::String(format!("错误: {e}")),
        Data::DurationIso(s) | Data::DateTimeIso(s) => serde_json::Value::String(s.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 造一个最小 xlsx 字节(rust_xlsxwriter,2x3 含表头)
    fn sample_xlsx() -> Vec<u8> {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.write_string(0, 0, "name").unwrap();
        sheet.write_string(0, 1, "age").unwrap();
        sheet.write_string(1, 0, "Alice").unwrap();
        sheet.write_number(1, 1, 30.0).unwrap();
        sheet.write_string(2, 0, "Bob").unwrap();
        sheet.write_number(2, 1, 25.0).unwrap();
        workbook.save_to_buffer().unwrap()
    }

    #[test]
    fn xlsx_to_json_reads_cells() {
        let json = xlsx_to_json(&sample_xlsx()).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let rows = value.as_array().unwrap();
        assert_eq!(rows.len(), 3, "应 3 行(含表头)");
        // 首行表头
        assert_eq!(rows[0][0], "name");
        assert_eq!(rows[0][1], "age");
        // 数据行:age 应为数字
        assert_eq!(rows[1][0], "Alice");
        assert_eq!(rows[1][1], 30);
    }

    #[test]
    fn json_to_xlsx_roundtrip() {
        let json = r#"[["name","age"],["Alice",30],["Bob",25]]"#;
        let xlsx = json_to_xlsx(json.as_bytes()).unwrap();
        // 读回验证
        let back = xlsx_to_json(&xlsx).unwrap();
        let value: serde_json::Value = serde_json::from_str(&back).unwrap();
        let rows = value.as_array().unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[1][0], "Alice");
        assert_eq!(rows[1][1], 30);
    }

    #[test]
    fn json_to_xlsx_invalid_rejected() {
        // 非数组应报错
        assert!(json_to_xlsx(b"{}").is_err());
        // 非二维(元素非数组)应报错
        assert!(json_to_xlsx(b"[1,2,3]").is_err());
    }

    #[test]
    fn xlsx_to_json_invalid_rejected() {
        assert!(xlsx_to_json(b"not xlsx").is_err());
    }
}
