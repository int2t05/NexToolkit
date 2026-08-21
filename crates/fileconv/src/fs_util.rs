//! 文件 IO 边界:归档产物落盘与碰撞处理(依赖 std::fs,非纯内存)
//!
//! 纯内存转换在 [`crate::archive`];本模块组合纯逻辑 + 文件读写,供 CLI/GUI 共享,
//! 避免边界逻辑重复。产物默认落源文件所在目录:解压到 `{stem}_extracted/`,
//! 压缩/转换到 `{stem}.{ext}`,碰撞追加 `_converted`→`(1)`→`(2)`,不静默覆盖。

use crate::{ArchiveEntry, ArchiveFormat};
use nextool_core::{ToolError, ToolResult};
use std::io::Write;
use std::path::Path;

/// 解压归档文件到目录(默认源文件旁 `_extracted`,碰撞追加 `(n)`)
///
/// 返回 `(输出目录, 写出的文件路径列表)`。
pub fn extract_to_dir(
    archive_path: &str,
    output_dir: Option<&str>,
) -> ToolResult<(String, Vec<String>)> {
    let data = std::fs::read(archive_path)?;
    let entries = crate::archive_extract(&data)?;
    let out_dir = resolve_extract_dir(archive_path, output_dir)?;
    let mut written = Vec::with_capacity(entries.len());
    for entry in entries {
        let name = entry_name(&entry, archive_path);
        let target = Path::new(&out_dir).join(&name);
        write_bytes_create_new(&entry.data, &target.to_string_lossy())?;
        written.push(target.to_string_lossy().into_owned());
    }
    Ok((out_dir, written))
}

/// 压缩文件为归档并落盘(默认输出到第一个文件旁),返回产物路径
pub fn compress_files(
    paths: &[String],
    format: ArchiveFormat,
    output: Option<&str>,
) -> ToolResult<String> {
    let entries: Vec<ArchiveEntry> = paths
        .iter()
        .map(|p| {
            let data = std::fs::read(p)?;
            let path = Path::new(p)
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| p.clone());
            Ok(ArchiveEntry { path, data })
        })
        .collect::<ToolResult<_>>()?;
    let archive = crate::archive_create(&entries, format)?;
    match output {
        Some(o) => {
            write_bytes_create_new(&archive, o)?;
            Ok(o.to_string())
        }
        None => write_bytes_safe(&archive, &paths[0], archive_ext(format)),
    }
}

/// 归档互转并落盘(默认输出到源文件旁),返回产物路径
pub fn convert_file(
    archive_path: &str,
    target: ArchiveFormat,
    output: Option<&str>,
) -> ToolResult<String> {
    let data = std::fs::read(archive_path)?;
    let out_data = crate::archive_convert(&data, target)?;
    match output {
        Some(o) => {
            write_bytes_create_new(&out_data, o)?;
            Ok(o.to_string())
        }
        None => write_bytes_safe(&out_data, archive_path, archive_ext(target)),
    }
}

/// 条目输出文件名:纯 gz 单文件(path 空)用源文件 stem,否则用条目路径
fn entry_name(entry: &ArchiveEntry, archive_path: &str) -> String {
    if entry.path.is_empty() {
        Path::new(archive_path)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "extracted".into())
    } else {
        entry.path.clone()
    }
}

/// 归档格式扩展名
fn archive_ext(fmt: ArchiveFormat) -> &'static str {
    match fmt {
        ArchiveFormat::Zip => "zip",
        ArchiveFormat::Tar => "tar",
        ArchiveFormat::TarGz => "tar.gz",
        ArchiveFormat::Gz => "gz",
    }
}

/// 原子写入(碰撞失败,不覆盖):`create_new` 保证检查与创建在同一系统调用,无 TOCTOU 竞态
///
/// 返回原始 [`std::io::Error`],调用方按 `ErrorKind::AlreadyExists` 判碰撞。自动创建父目录。
fn write_bytes_create_new(data: &[u8], path: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    std::io::BufWriter::new(file).write_all(data)
}

/// 迭代碰撞后缀写入直到成功,返回最终路径;超过 100 次报错
fn write_bytes_safe(data: &[u8], input_path: &str, target_ext: &str) -> ToolResult<String> {
    for attempt in 0..100u32 {
        let out = crate::compute_output_path(input_path, target_ext, attempt);
        match write_bytes_create_new(data, &out) {
            Ok(()) => return Ok(out),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(ToolError::Io(e)),
        }
    }
    Err(ToolError::Other(
        "碰撞次数过多:同名产物已存在 100 个".into(),
    ))
}

/// 确定解压输出目录:显式指定用之;否则源文件旁 `{stem}_extracted`,碰撞追加 `(n)`
fn resolve_extract_dir(input: &str, output_dir: Option<&str>) -> ToolResult<String> {
    match output_dir {
        Some(d) => {
            std::fs::create_dir_all(d)?;
            Ok(d.to_string())
        }
        None => {
            for attempt in 0..100u32 {
                let dir = crate::compute_extract_dir(input, attempt);
                match std::fs::create_dir(&dir) {
                    Ok(()) => return Ok(dir),
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(e) => return Err(ToolError::Io(e)),
                }
            }
            Err(ToolError::Other(
                "碰撞次数过多:同名解压目录已存在 100 个".into(),
            ))
        }
    }
}
