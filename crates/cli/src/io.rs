//! CLI 输入输出辅助:stdin 读取、退出码处理

use std::io::Read;

/// 读取输入:有参数用参数,否则读 stdin 全部;stdin 为空时报错
pub fn read_input(input: Option<String>) -> Result<String, String> {
    match input {
        Some(s) => Ok(s),
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| e.to_string())?;
            if buf.is_empty() {
                return Err("无输入:请提供参数或通过 stdin 传入".into());
            }
            Ok(buf)
        }
    }
}

/// 读取可选输入:有参数用参数,无参数且 stdin 无数据时返回 None(用于无输入的生成类工具)
pub fn read_input_optional(input: Option<String>) -> Result<Option<String>, String> {
    match input {
        Some(s) => Ok(Some(s)),
        None => {
            let mut buf = String::new();
            if std::io::stdin().read_to_string(&mut buf).is_ok() && !buf.is_empty() {
                Ok(Some(buf))
            } else {
                Ok(None)
            }
        }
    }
}
