//! 归档模块:ZIP/TAR/TAR.GZ/GZ 解压与压缩(字节域,纯内存)
//!
//! 输入输出为 `&[u8]`/`Vec<u8>`,不碰文件系统。格式经魔术字节检测自动路由。
//! 路径安全:拒绝 `..` 遍历与绝对路径,避免 zip-slip 类攻击。

use nextool_core::{ToolError, ToolResult};
use std::io::{Read, Write};

/// 归档格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    Gz,
}

/// 归档条目:归档内相对路径 + 文件内容(纯 gz 单文件时 path 为空串)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    pub path: String,
    pub data: Vec<u8>,
}

/// 检测归档格式(魔术字节路由)
///
/// ZIP: `50 4B 03 04`;GZ: `1F 8B`(tar.gz 与纯 gz 统一标 Gz,解压时再分);
/// TAR: `ustar` @ offset 257。无法识别返回 `InvalidInput`。
pub fn detect_archive_format(data: &[u8]) -> ToolResult<ArchiveFormat> {
    if data.len() >= 4 && data[0..4] == [0x50, 0x4B, 0x03, 0x04] {
        return Ok(ArchiveFormat::Zip);
    }
    if data.len() >= 2 && data[0] == 0x1F && data[1] == 0x8B {
        return Ok(ArchiveFormat::Gz);
    }
    if data.len() >= 262 && &data[257..262] == b"ustar" {
        return Ok(ArchiveFormat::Tar);
    }
    Err(ToolError::InvalidInput(
        "无法识别归档格式:魔术字节不匹配".into(),
    ))
}

/// 列出归档内文件,每行 `路径\t大小(字节)`,纯 gz 单文件路径显示为 `(解压内容)`
pub fn archive_list(data: &[u8]) -> ToolResult<String> {
    let (_fmt, entries) = extract_auto(data)?;
    let lines: Vec<String> = entries
        .iter()
        .map(|e| {
            let name = if e.path.is_empty() {
                "(解压内容)"
            } else {
                &e.path
            };
            format!("{name}\t{}", e.data.len())
        })
        .collect();
    Ok(lines.join("\n"))
}

/// 解压归档(自动检测格式),返回条目列表;路径安全校验
pub fn archive_extract(data: &[u8]) -> ToolResult<Vec<ArchiveEntry>> {
    Ok(extract_auto(data)?.1)
}

/// 解压归档(指定格式,跳过自动检测)
pub fn archive_extract_as(data: &[u8], format: ArchiveFormat) -> ToolResult<Vec<ArchiveEntry>> {
    match format {
        ArchiveFormat::Zip => zip_extract(data),
        ArchiveFormat::Tar => tar_extract(data),
        ArchiveFormat::TarGz => {
            let decompressed = gz_decompress(data)?;
            tar_extract(&decompressed)
        }
        ArchiveFormat::Gz => {
            let decompressed = gz_decompress(data)?;
            Ok(vec![ArchiveEntry {
                path: String::new(),
                data: decompressed,
            }])
        }
    }
}

/// 创建归档:将条目列表打包为指定格式
///
/// GZ 为单文件格式,`entries` 长度必须为 1;条目路径先经安全校验。
pub fn archive_create(entries: &[ArchiveEntry], format: ArchiveFormat) -> ToolResult<Vec<u8>> {
    for e in entries {
        if !e.path.is_empty() {
            validate_entry_path(&e.path)?;
        }
    }
    match format {
        ArchiveFormat::Zip => zip_create(entries),
        ArchiveFormat::Tar => tar_create(entries),
        ArchiveFormat::TarGz => {
            let tar = tar_create(entries)?;
            gz_compress(&tar)
        }
        ArchiveFormat::Gz => {
            if entries.len() != 1 {
                return Err(ToolError::InvalidInput(
                    "GZ 为单文件压缩格式,仅支持一个条目".into(),
                ));
            }
            gz_compress(&entries[0].data)
        }
    }
}

/// 归档格式互转:解压(自动检测)→ 重新打包为目标格式
///
/// 多文件归档转 GZ 时由 [`archive_create`] 拒绝(`InvalidInput`)。
pub fn archive_convert(data: &[u8], target: ArchiveFormat) -> ToolResult<Vec<u8>> {
    let entries = archive_extract(data)?;
    archive_create(&entries, target)
}

// ---- 私有:格式特定解压/压缩 ----

/// 自动检测并解压,处理 GZ 歧义(先解压 gz 再尝试 tar:成功为 tar.gz,否则纯 gz)
fn extract_auto(data: &[u8]) -> ToolResult<(ArchiveFormat, Vec<ArchiveEntry>)> {
    match detect_archive_format(data)? {
        ArchiveFormat::Zip => Ok((ArchiveFormat::Zip, zip_extract(data)?)),
        ArchiveFormat::Tar => Ok((ArchiveFormat::Tar, tar_extract(data)?)),
        ArchiveFormat::Gz => {
            let decompressed = gz_decompress(data)?;
            match tar_extract(&decompressed) {
                Ok(entries) => Ok((ArchiveFormat::TarGz, entries)),
                Err(_) => Ok((
                    ArchiveFormat::Gz,
                    vec![ArchiveEntry {
                        path: String::new(),
                        data: decompressed,
                    }],
                )),
            }
        }
        ArchiveFormat::TarGz => unreachable!("detect_archive_format 不会返回 TarGz"),
    }
}

fn gz_compress(data: &[u8]) -> ToolResult<Vec<u8>> {
    use flate2::{write::GzEncoder, Compression};
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

fn gz_decompress(data: &[u8]) -> ToolResult<Vec<u8>> {
    use flate2::read::GzDecoder;
    let mut decoder = GzDecoder::new(data);
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)?;
    Ok(buf)
}

fn tar_extract(data: &[u8]) -> ToolResult<Vec<ArchiveEntry>> {
    let mut archive = tar::Archive::new(data);
    let mut entries = Vec::new();
    for entry in archive.entries()? {
        let mut entry = entry?;
        if entry.header().entry_type().is_dir() {
            continue;
        }
        let path = entry.path()?.to_string_lossy().to_string();
        validate_entry_path(&path)?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        entries.push(ArchiveEntry { path, data: buf });
    }
    Ok(entries)
}

fn tar_create(entries: &[ArchiveEntry]) -> ToolResult<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut buf);
        for entry in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(entry.data.len() as u64);
            header.set_mode(0o644);
            header.set_mtime(0);
            header.set_cksum();
            builder.append_data(&mut header, entry.path.as_str(), entry.data.as_slice())?;
        }
        builder.finish()?;
    }
    Ok(buf)
}

fn zip_extract(data: &[u8]) -> ToolResult<Vec<ArchiveEntry>> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data))
        .map_err(|e| ToolError::Other(e.to_string()))?;
    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| ToolError::Other(e.to_string()))?;
        if file.is_dir() {
            continue;
        }
        // enclosed_name 已对 .. 做净化,返回 None 表示路径不安全
        let path = file
            .enclosed_name()
            .ok_or_else(|| ToolError::InvalidInput(format!("zip 条目路径不安全: {}", file.name())))?
            .to_string_lossy()
            .to_string();
        validate_entry_path(&path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        entries.push(ArchiveEntry { path, data: buf });
    }
    Ok(entries)
}

fn zip_create(entries: &[ArchiveEntry]) -> ToolResult<Vec<u8>> {
    use zip::write::SimpleFileOptions;
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default();
    for entry in entries {
        let name = if entry.path.is_empty() {
            "content"
        } else {
            entry.path.as_str()
        };
        zip.start_file(name, opts)
            .map_err(|e| ToolError::Other(e.to_string()))?;
        zip.write_all(&entry.data)?;
    }
    let cursor = zip.finish().map_err(|e| ToolError::Other(e.to_string()))?;
    Ok(cursor.into_inner())
}

/// 校验条目路径:拒绝绝对路径(Unix `/`、Windows 盘符)与 `..` 遍历
fn validate_entry_path(path: &str) -> ToolResult<()> {
    if path.starts_with('/') || path.starts_with('\\') {
        return Err(ToolError::InvalidInput(format!(
            "归档条目路径为绝对路径: {path}"
        )));
    }
    // Windows 盘符如 C:\
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return Err(ToolError::InvalidInput(format!(
            "归档条目路径为绝对路径: {path}"
        )));
    }
    for component in path.split(['/', '\\']) {
        if component == ".." {
            return Err(ToolError::InvalidInput(format!(
                "归档条目路径含 .. 遍历: {path}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entries() -> Vec<ArchiveEntry> {
        vec![
            ArchiveEntry {
                path: "a.txt".into(),
                data: b"hello".to_vec(),
            },
            ArchiveEntry {
                path: "dir/b.txt".into(),
                data: b"world".to_vec(),
            },
        ]
    }

    // ---- detect_archive_format ----

    #[test]
    fn detect_zip_magic() {
        let zip = archive_create(&sample_entries(), ArchiveFormat::Zip).unwrap();
        assert_eq!(detect_archive_format(&zip).unwrap(), ArchiveFormat::Zip);
    }

    #[test]
    fn detect_gz_magic() {
        let gz = archive_create(
            &[ArchiveEntry {
                path: "x".into(),
                data: b"data".to_vec(),
            }],
            ArchiveFormat::Gz,
        )
        .unwrap();
        assert_eq!(detect_archive_format(&gz).unwrap(), ArchiveFormat::Gz);
    }

    #[test]
    fn detect_tar_magic() {
        let tar = archive_create(&sample_entries(), ArchiveFormat::Tar).unwrap();
        assert_eq!(detect_archive_format(&tar).unwrap(), ArchiveFormat::Tar);
    }

    #[test]
    fn detect_unknown_rejected() {
        assert!(detect_archive_format(&[0x00, 0x01, 0x02]).is_err());
        assert!(detect_archive_format(b"plain text").is_err());
        assert!(detect_archive_format(&[]).is_err());
    }

    #[test]
    fn detect_short_input_rejected() {
        // 不足魔术字节长度
        assert!(detect_archive_format(&[0x50, 0x4B]).is_err());
    }

    // ---- validate_entry_path ----

    #[test]
    fn reject_path_traversal() {
        assert!(validate_entry_path("../etc/passwd").is_err());
        assert!(validate_entry_path("dir/../secret").is_err());
        assert!(validate_entry_path("a/../../b").is_err());
    }

    #[test]
    fn reject_absolute_path() {
        assert!(validate_entry_path("/etc/passwd").is_err());
        assert!(validate_entry_path("\\windows\\sys").is_err());
        assert!(validate_entry_path("C:\\Users").is_err());
    }

    #[test]
    fn accept_safe_paths() {
        assert!(validate_entry_path("a.txt").is_ok());
        assert!(validate_entry_path("dir/b.txt").is_ok());
        assert!(validate_entry_path("a/b/c.txt").is_ok());
    }

    // ---- gz ----

    #[test]
    fn gz_roundtrip() {
        for data in [b"".as_ref(), b"a", b"hello world", b"\x00\x01\x02\xff"] {
            let compressed = gz_compress(data).unwrap();
            assert_eq!(gz_decompress(&compressed).unwrap(), data);
        }
    }

    #[test]
    fn gz_decompress_invalid() {
        assert!(gz_decompress(b"not gzip").is_err());
        assert!(gz_decompress(&[]).is_err());
    }

    #[test]
    fn gz_create_single_entry_only() {
        let entries = sample_entries(); // 2 条目
        assert!(archive_create(&entries, ArchiveFormat::Gz).is_err());
    }

    #[test]
    fn gz_single_roundtrip() {
        let entry = ArchiveEntry {
            path: "data.bin".into(),
            data: b"contents".to_vec(),
        };
        let gz = archive_create(std::slice::from_ref(&entry), ArchiveFormat::Gz).unwrap();
        let extracted = archive_extract_as(&gz, ArchiveFormat::Gz).unwrap();
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].data, b"contents");
        assert!(extracted[0].path.is_empty()); // 纯 gz 单文件 path 为空
    }

    // ---- tar ----

    #[test]
    fn tar_roundtrip() {
        let entries = sample_entries();
        let tar = archive_create(&entries, ArchiveFormat::Tar).unwrap();
        let extracted = archive_extract_as(&tar, ArchiveFormat::Tar).unwrap();
        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted[0].path, "a.txt");
        assert_eq!(extracted[0].data, b"hello");
        assert_eq!(extracted[1].path, "dir/b.txt");
        assert_eq!(extracted[1].data, b"world");
    }

    #[test]
    fn tar_empty_entries() {
        let tar = archive_create(&[], ArchiveFormat::Tar).unwrap();
        let extracted = archive_extract_as(&tar, ArchiveFormat::Tar).unwrap();
        assert!(extracted.is_empty());
    }

    #[test]
    fn tar_binary_data_roundtrip() {
        let data: Vec<u8> = (0..=255).collect();
        let entries = vec![ArchiveEntry {
            path: "bytes.bin".into(),
            data,
        }];
        let tar = archive_create(&entries, ArchiveFormat::Tar).unwrap();
        let extracted = archive_extract_as(&tar, ArchiveFormat::Tar).unwrap();
        assert_eq!(extracted[0].data, (0..=255).collect::<Vec<u8>>());
    }

    #[test]
    fn tar_skips_directory_entries() {
        // 含目录条目的 tar:解压应跳过目录,只返回文件
        let mut buf = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut buf);
            let mut dir_header = tar::Header::new_gnu();
            dir_header.set_entry_type(tar::EntryType::Directory);
            dir_header.set_size(0);
            dir_header.set_mode(0o755);
            dir_header.set_mtime(0);
            dir_header.set_cksum();
            builder
                .append_data(&mut dir_header, "subdir", std::io::empty())
                .unwrap();
            let mut file_header = tar::Header::new_gnu();
            file_header.set_size(3);
            file_header.set_mode(0o644);
            file_header.set_mtime(0);
            file_header.set_cksum();
            builder
                .append_data(&mut file_header, "subdir/f.txt", &b"abc"[..])
                .unwrap();
            builder.finish().unwrap();
        }
        let extracted = tar_extract(&buf).unwrap();
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].path, "subdir/f.txt");
        assert_eq!(extracted[0].data, b"abc");
    }

    // ---- targz ----

    #[test]
    fn targz_roundtrip() {
        let entries = sample_entries();
        let targz = archive_create(&entries, ArchiveFormat::TarGz).unwrap();
        // 自动检测:targz 的魔术字节是 Gz
        assert_eq!(detect_archive_format(&targz).unwrap(), ArchiveFormat::Gz);
        let extracted = archive_extract(&targz).unwrap(); // 自动解 tar.gz
        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted[0].data, b"hello");
    }

    #[test]
    fn targz_extract_as_explicit() {
        let entries = sample_entries();
        let targz = archive_create(&entries, ArchiveFormat::TarGz).unwrap();
        let extracted = archive_extract_as(&targz, ArchiveFormat::TarGz).unwrap();
        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted[1].path, "dir/b.txt");
    }

    #[test]
    fn targz_list_shows_inner_files() {
        let entries = sample_entries();
        let targz = archive_create(&entries, ArchiveFormat::TarGz).unwrap();
        let list = archive_list(&targz).unwrap();
        assert!(list.contains("a.txt\t5"));
        assert!(list.contains("dir/b.txt\t5"));
    }

    // ---- zip ----

    #[test]
    fn zip_roundtrip() {
        let entries = sample_entries();
        let zip = archive_create(&entries, ArchiveFormat::Zip).unwrap();
        let extracted = archive_extract_as(&zip, ArchiveFormat::Zip).unwrap();
        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted[0].path, "a.txt");
        assert_eq!(extracted[0].data, b"hello");
        assert_eq!(extracted[1].path, "dir/b.txt");
        assert_eq!(extracted[1].data, b"world");
    }

    #[test]
    fn zip_binary_data_roundtrip() {
        let data: Vec<u8> = (0..=255).collect();
        let entries = vec![ArchiveEntry {
            path: "bytes.bin".into(),
            data,
        }];
        let zip = archive_create(&entries, ArchiveFormat::Zip).unwrap();
        let extracted = archive_extract_as(&zip, ArchiveFormat::Zip).unwrap();
        assert_eq!(extracted[0].data, (0..=255).collect::<Vec<u8>>());
    }

    #[test]
    fn zip_empty_entries() {
        let zip = archive_create(&[], ArchiveFormat::Zip).unwrap();
        let extracted = archive_extract_as(&zip, ArchiveFormat::Zip).unwrap();
        assert!(extracted.is_empty());
    }

    #[test]
    fn zip_extract_invalid() {
        assert!(zip_extract(b"not a zip").is_err());
    }

    // ---- compute_output_path / compute_extract_dir 测试见 path.rs ----

    // ---- 统一 API ----

    #[test]
    fn extract_auto_detects_zip() {
        let zip = archive_create(&sample_entries(), ArchiveFormat::Zip).unwrap();
        let extracted = archive_extract(&zip).unwrap();
        assert_eq!(extracted.len(), 2);
    }

    #[test]
    fn extract_auto_detects_tar() {
        let tar = archive_create(&sample_entries(), ArchiveFormat::Tar).unwrap();
        let extracted = archive_extract(&tar).unwrap();
        assert_eq!(extracted[0].path, "a.txt");
    }

    #[test]
    fn extract_auto_detects_targz() {
        let targz = archive_create(&sample_entries(), ArchiveFormat::TarGz).unwrap();
        let extracted = archive_extract(&targz).unwrap();
        assert_eq!(extracted.len(), 2);
    }

    #[test]
    fn list_zip_format() {
        let zip = archive_create(&sample_entries(), ArchiveFormat::Zip).unwrap();
        let list = archive_list(&zip).unwrap();
        assert!(list.contains("a.txt\t5"));
        assert!(list.contains("dir/b.txt\t5"));
    }

    #[test]
    fn extract_unknown_format_rejected() {
        assert!(archive_extract(b"not an archive").is_err());
    }

    // ---- archive_convert ----

    #[test]
    fn convert_zip_to_tar() {
        let entries = sample_entries();
        let zip = archive_create(&entries, ArchiveFormat::Zip).unwrap();
        let tar = archive_convert(&zip, ArchiveFormat::Tar).unwrap();
        let extracted = archive_extract_as(&tar, ArchiveFormat::Tar).unwrap();
        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted[0].data, b"hello");
    }

    #[test]
    fn convert_tar_to_zip() {
        let entries = sample_entries();
        let tar = archive_create(&entries, ArchiveFormat::Tar).unwrap();
        let zip = archive_convert(&tar, ArchiveFormat::Zip).unwrap();
        let extracted = archive_extract_as(&zip, ArchiveFormat::Zip).unwrap();
        assert_eq!(extracted[1].path, "dir/b.txt");
    }

    #[test]
    fn convert_multi_to_gz_rejected() {
        let zip = archive_create(&sample_entries(), ArchiveFormat::Zip).unwrap();
        assert!(archive_convert(&zip, ArchiveFormat::Gz).is_err());
    }

    #[test]
    fn convert_targz_to_zip() {
        let entries = sample_entries();
        let targz = archive_create(&entries, ArchiveFormat::TarGz).unwrap();
        let zip = archive_convert(&targz, ArchiveFormat::Zip).unwrap();
        let extracted = archive_extract_as(&zip, ArchiveFormat::Zip).unwrap();
        assert_eq!(extracted[0].data, b"hello");
    }
}
