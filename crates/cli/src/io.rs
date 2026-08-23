//! CLI 输入输出辅助:stdin 读取、print 样板

use std::io::Read;

use nextool_core::ToolResult;

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

/// 读取二进制输入:按文件路径读取全部字节(供归档列表等纯内存查看场景)
pub fn read_bytes_input(path: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("读取文件失败 {path}: {e}"))
}

/// 读输入 → 执行 → 打印,折叠 read_input + map_err + println 三行样板
///
/// 适用于"单输入 → core 文本函数 → println"的子命令臂。
/// 多输入/PEM 加载/可选输入等结构不同的臂应保持显式。
pub fn print_text<F>(input: Option<String>, f: F) -> Result<(), String>
where
    F: FnOnce(&str) -> ToolResult<String>,
{
    let input = read_input(input)?;
    println!("{}", f(&input).map_err(|e| e.to_string())?);
    Ok(())
}
