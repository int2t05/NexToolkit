//! 归档模块:ZIP/TAR/TAR.GZ/GZ/BZ2/XZ/ZST 解压与压缩(字节域,纯内存)
//!
//! 输入输出为 `&[u8]`/`Vec<u8>`,不碰文件系统。格式经魔术字节检测自动路由。
//! 路径安全:拒绝 `..` 遍历与绝对路径,避免 zip-slip 类攻击。
//! BZ2/XZ/ZST 同 GZ 为单文件压缩格式,`entries` 长度必须为 1。

use crate::{ToolError, ToolResult};
use std::io::{Read, Write};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr, strum::EnumString, strum::EnumIter,
)]
pub enum ArchiveFormat {
    #[strum(serialize = "zip")]
    Zip,
    #[strum(serialize = "tar")]
    Tar,
    #[strum(serialize = "targz")]
    TarGz,
    #[strum(serialize = "gz")]
    Gz,
    #[strum(serialize = "7z")]
    SevenZ,
    #[strum(serialize = "bz2")]
    Bz2,
    #[strum(serialize = "xz")]
    Xz,
    #[strum(serialize = "zst")]
    Zst,
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
/// 7Z: `37 7A BC AF 27 1C`;TAR: `ustar` @ offset 257;
/// BZ2: `42 5A 68`("BZh");XZ: `FD 37 7A 58 5A 00`;ZST: `28 B5 2F FD`。
/// 无法识别返回 `InvalidInput`。
pub fn detect_archive_format(data: &[u8]) -> ToolResult<ArchiveFormat> {
    if data.len() >= 4 && data[0..4] == [0x50, 0x4B, 0x03, 0x04] {
        return Ok(ArchiveFormat::Zip);
    }
    if data.len() >= 6 && data[0..6] == [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C] {
        return Ok(ArchiveFormat::SevenZ);
    }
    if data.len() >= 6 && data[0..6] == [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00] {
        return Ok(ArchiveFormat::Xz);
    }
    if data.len() >= 4 && data[0..4] == [0x28, 0xB5, 0x2F, 0xFD] {
        return Ok(ArchiveFormat::Zst);
    }
    if data.len() >= 3 && data[0..3] == [0x42, 0x5A, 0x68] {
        return Ok(ArchiveFormat::Bz2);
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

/// 归档条目元信息(列出归档内容时的结构化返回)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveListEntry {
    pub path: String,
    pub size: usize,
}

/// 列出归档内文件(结构化:路径 + 字节数);纯 gz 单文件路径为 `(解压内容)`
pub fn archive_list_entries(data: &[u8]) -> ToolResult<Vec<ArchiveListEntry>> {
    let entries = extract_auto(data)?.1;
    Ok(entries
        .into_iter()
        .map(|e| {
            let path = if e.path.is_empty() {
                "(解压内容)".into()
            } else {
                e.path
            };
            ArchiveListEntry {
                path,
                size: e.data.len(),
            }
        })
        .collect())
}

/// 列出归档内文件,每行 `路径\t大小(字节)`(格式化展示,供 CLI 直接打印)
pub fn archive_list(data: &[u8]) -> ToolResult<String> {
    let entries = archive_list_entries(data)?;
    Ok(entries
        .iter()
        .map(|e| format!("{}\t{}", e.path, e.size))
        .collect::<Vec<_>>()
        .join("\n"))
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
        ArchiveFormat::Gz => single_entry(gz_decompress(data)?),
        ArchiveFormat::SevenZ => sevenz_extract(data),
        ArchiveFormat::Bz2 => single_entry(bz2_decompress(data)?),
        ArchiveFormat::Xz => single_entry(xz_decompress(data)?),
        ArchiveFormat::Zst => single_entry(zst_decompress(data)?),
    }
}

/// 创建归档:将条目列表打包为指定格式
///
/// GZ/BZ2/XZ/ZST 为单文件压缩格式,`entries` 长度必须为 1;条目路径先经安全校验。
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
        ArchiveFormat::Gz => single_file(entries, gz_compress),
        ArchiveFormat::SevenZ => Err(ToolError::InvalidInput("7z 创建暂不支持(仅解压)".into())),
        ArchiveFormat::Bz2 => single_file(entries, bz2_compress),
        ArchiveFormat::Xz => single_file(entries, xz_compress),
        ArchiveFormat::Zst => single_file(entries, zst_compress),
    }
}

/// 单文件压缩格式(GZ/BZ2/XZ/ZST)统一处理:校验仅一个条目,取其数据压缩
fn single_file(
    entries: &[ArchiveEntry],
    compress: impl Fn(&[u8]) -> ToolResult<Vec<u8>>,
) -> ToolResult<Vec<u8>> {
    if entries.len() != 1 {
        return Err(ToolError::InvalidInput(
            "该格式为单文件压缩格式,仅支持一个条目".into(),
        ));
    }
    compress(&entries[0].data)
}

/// 单文件解压结果包装为单条目(path 为空串,与 GZ 模式一致)
fn single_entry(data: Vec<u8>) -> ToolResult<Vec<ArchiveEntry>> {
    Ok(vec![ArchiveEntry {
        path: String::new(),
        data,
    }])
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
        ArchiveFormat::SevenZ => Ok((ArchiveFormat::SevenZ, sevenz_extract(data)?)),
        ArchiveFormat::Bz2 => Ok((ArchiveFormat::Bz2, single_entry(bz2_decompress(data)?)?)),
        ArchiveFormat::Xz => Ok((ArchiveFormat::Xz, single_entry(xz_decompress(data)?)?)),
        ArchiveFormat::Zst => Ok((ArchiveFormat::Zst, single_entry(zst_decompress(data)?)?)),
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

fn bz2_compress(data: &[u8]) -> ToolResult<Vec<u8>> {
    use bzip2::{write::BzEncoder, Compression};
    let mut encoder = BzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

fn bz2_decompress(data: &[u8]) -> ToolResult<Vec<u8>> {
    use bzip2::read::BzDecoder;
    let mut decoder = BzDecoder::new(data);
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)?;
    Ok(buf)
}

fn xz_compress(data: &[u8]) -> ToolResult<Vec<u8>> {
    use xz2::write::XzEncoder;
    let mut encoder = XzEncoder::new(Vec::new(), 6);
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

fn xz_decompress(data: &[u8]) -> ToolResult<Vec<u8>> {
    use xz2::read::XzDecoder;
    let mut decoder = XzDecoder::new(data);
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)?;
    Ok(buf)
}

fn zst_compress(data: &[u8]) -> ToolResult<Vec<u8>> {
    Ok(zstd::encode_all(data, 3)?)
}

fn zst_decompress(data: &[u8]) -> ToolResult<Vec<u8>> {
    Ok(zstd::decode_all(data)?)
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

fn sevenz_extract(data: &[u8]) -> ToolResult<Vec<ArchiveEntry>> {
    use sevenz_rust2::{ArchiveReader, Password};
    use std::io::Cursor;
    let mut reader = ArchiveReader::new(Cursor::new(data), Password::empty())
        .map_err(|e| ToolError::Other(format!("7z 打开失败: {e}")))?;
    let mut entries = Vec::new();
    reader
        .for_each_entries(|entry, stream| {
            if entry.is_directory || !entry.has_stream {
                return Ok(false);
            }
            let mut buf = Vec::new();
            stream.read_to_end(&mut buf)?;
            entries.push(ArchiveEntry {
                path: entry.name.clone(),
                data: buf,
            });
            Ok(false)
        })
        .map_err(|e| ToolError::Other(format!("7z 解压失败: {e}")))?;
    // 路径安全校验(闭包返回类型限制,在外统一做)
    for e in &entries {
        validate_entry_path(&e.path)?;
    }
    Ok(entries)
}

fn zip_err(e: zip::result::ZipError) -> ToolError {
    ToolError::Other(e.to_string())
}

fn zip_extract(data: &[u8]) -> ToolResult<Vec<ArchiveEntry>> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data)).map_err(zip_err)?;
    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(zip_err)?;
        if file.is_dir() {
            continue;
        }
        // enclosed_name 拒绝 .. 与 Unix 绝对路径;validate_entry_path 补充反斜杠分隔的 .. 与 Windows 盘符
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
        zip.start_file(name, opts).map_err(zip_err)?;
        zip.write_all(&entry.data)?;
    }
    let cursor = zip.finish().map_err(zip_err)?;
    Ok(cursor.into_inner())
}

/// 校验条目路径:拒绝绝对路径(`/`、`\`)与 Windows 盘符(`C:`)与 `..` 遍历
fn validate_entry_path(path: &str) -> ToolResult<()> {
    let bytes = path.as_bytes();
    // 绝对路径(`/` 或 `\`)或 Windows 盘符(`C:`)
    let is_absolute = path.starts_with('/')
        || path.starts_with('\\')
        || (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':');
    if is_absolute {
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

    // ---- 7z ----

    /// 用 sevenz-rust2 writer 造一个含两文件的 7z 字节(真实数据,非 mock)
    fn sample_sevenz() -> Vec<u8> {
        use sevenz_rust2::{ArchiveEntry, ArchiveWriter};
        let mut buf = Vec::new();
        let mut writer = ArchiveWriter::new(std::io::Cursor::new(&mut buf)).unwrap();
        let mut e1 = ArchiveEntry::new_file("a.txt");
        e1.has_stream = true;
        writer
            .push_archive_entry(e1, Some(b"hello".as_slice()))
            .unwrap();
        let mut e2 = ArchiveEntry::new_file("dir/b.txt");
        e2.has_stream = true;
        writer
            .push_archive_entry(e2, Some(b"world".as_slice()))
            .unwrap();
        writer.finish().unwrap();
        buf
    }

    #[test]
    fn detect_sevenz_magic() {
        let data = sample_sevenz();
        assert_eq!(detect_archive_format(&data).unwrap(), ArchiveFormat::SevenZ);
    }

    #[test]
    fn sevenz_extract_roundtrip() {
        let data = sample_sevenz();
        let entries = archive_extract_as(&data, ArchiveFormat::SevenZ).unwrap();
        assert_eq!(entries.len(), 2, "应解出 2 个文件");
        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        assert!(paths.contains(&"a.txt"));
        assert!(paths.contains(&"dir/b.txt"));
        for e in &entries {
            match e.path.as_str() {
                "a.txt" => assert_eq!(e.data, b"hello"),
                "dir/b.txt" => assert_eq!(e.data, b"world"),
                _ => {}
            }
        }
    }

    #[test]
    fn sevenz_create_rejected() {
        let entries = vec![ArchiveEntry {
            path: "x".into(),
            data: b"data".to_vec(),
        }];
        assert!(archive_create(&entries, ArchiveFormat::SevenZ).is_err());
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

    // ---- extract 路径安全(归档级集成)----

    fn malicious_zip(path: &str) -> Vec<u8> {
        use zip::write::SimpleFileOptions;
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        zip.start_file(path, SimpleFileOptions::default()).unwrap();
        zip.write_all(b"x").unwrap();
        zip.finish().unwrap().into_inner()
    }

    fn malicious_tar(path: &str) -> Vec<u8> {
        // tar crate 写入端拒绝含 `..` 的路径;先构造合法 tar,再篡改 name 字段并重算 chksum
        let mut buf = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut buf);
            let mut header = tar::Header::new_gnu();
            header.set_size(1);
            header.set_mode(0o644);
            header.set_mtime(0);
            header.set_cksum();
            builder
                .append_data(&mut header, "placeholder", &b"x"[..])
                .unwrap();
            builder.finish().unwrap();
        }
        let name = path.as_bytes();
        buf[..name.len()].copy_from_slice(name);
        buf[name.len()..100].fill(0);
        buf[148..156].fill(b' ');
        let chksum: u32 = buf[..512].iter().map(|&b| b as u32).sum();
        let chksum_oct = format!("{:06o}\0 ", chksum);
        buf[148..156].copy_from_slice(chksum_oct.as_bytes());
        buf
    }

    fn malicious_sevenz(path: &str) -> Vec<u8> {
        use sevenz_rust2::{ArchiveEntry, ArchiveWriter};
        let mut buf = Vec::new();
        let mut writer = ArchiveWriter::new(std::io::Cursor::new(&mut buf)).unwrap();
        let mut e = ArchiveEntry::new_file(path);
        e.has_stream = true;
        writer.push_archive_entry(e, Some(b"x".as_slice())).unwrap();
        writer.finish().unwrap();
        buf
    }

    #[test]
    fn zip_rejects_malicious_paths() {
        for &evil in &["../evil", "/etc/passwd", "C:\\x"] {
            assert!(
                zip_extract(&malicious_zip(evil)).is_err(),
                "zip 应拒绝: {evil}"
            );
        }
    }

    #[test]
    fn tar_rejects_malicious_paths() {
        for &evil in &["../evil", "/etc/passwd", "C:\\x"] {
            assert!(
                tar_extract(&malicious_tar(evil)).is_err(),
                "tar 应拒绝: {evil}"
            );
        }
    }

    #[test]
    fn sevenz_rejects_malicious_paths() {
        for &evil in &["../evil", "/etc/passwd", "C:\\x"] {
            assert!(
                sevenz_extract(&malicious_sevenz(evil)).is_err(),
                "7z 应拒绝: {evil}"
            );
        }
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

    #[test]
    fn gz_list_shows_decompressed_content() {
        let gz = archive_create(
            std::slice::from_ref(&ArchiveEntry {
                path: "data.bin".into(),
                data: b"contents".to_vec(),
            }),
            ArchiveFormat::Gz,
        )
        .unwrap();
        let list = archive_list(&gz).unwrap();
        assert!(list.contains("(解压内容)\t8"));
    }

    // ---- bz2 ----

    #[test]
    fn bz2_roundtrip() {
        for data in [b"".as_ref(), b"a", b"hello world", b"\x00\x01\x02\xff"] {
            let compressed = bz2_compress(data).unwrap();
            assert_eq!(bz2_decompress(&compressed).unwrap(), data);
        }
    }

    #[test]
    fn detect_bz2_magic() {
        let bz2 = archive_create(
            std::slice::from_ref(&ArchiveEntry {
                path: "x".into(),
                data: b"data".to_vec(),
            }),
            ArchiveFormat::Bz2,
        )
        .unwrap();
        assert_eq!(detect_archive_format(&bz2).unwrap(), ArchiveFormat::Bz2);
    }

    #[test]
    fn bz2_single_roundtrip() {
        let entry = ArchiveEntry {
            path: "data.bin".into(),
            data: b"contents".to_vec(),
        };
        let bz2 = archive_create(std::slice::from_ref(&entry), ArchiveFormat::Bz2).unwrap();
        let extracted = archive_extract_as(&bz2, ArchiveFormat::Bz2).unwrap();
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].data, b"contents");
        assert!(extracted[0].path.is_empty());
    }

    #[test]
    fn bz2_multi_entry_rejected() {
        assert!(archive_create(&sample_entries(), ArchiveFormat::Bz2).is_err());
    }

    #[test]
    fn bz2_extract_auto() {
        let bz2 = archive_create(
            std::slice::from_ref(&ArchiveEntry {
                path: "x".into(),
                data: b"auto detect me".to_vec(),
            }),
            ArchiveFormat::Bz2,
        )
        .unwrap();
        let extracted = archive_extract(&bz2).unwrap();
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].data, b"auto detect me");
    }

    // ---- xz ----

    #[test]
    fn xz_roundtrip() {
        for data in [b"".as_ref(), b"a", b"hello world", b"\x00\x01\x02\xff"] {
            let compressed = xz_compress(data).unwrap();
            assert_eq!(xz_decompress(&compressed).unwrap(), data);
        }
    }

    #[test]
    fn detect_xz_magic() {
        let xz = archive_create(
            std::slice::from_ref(&ArchiveEntry {
                path: "x".into(),
                data: b"data".to_vec(),
            }),
            ArchiveFormat::Xz,
        )
        .unwrap();
        assert_eq!(detect_archive_format(&xz).unwrap(), ArchiveFormat::Xz);
    }

    #[test]
    fn xz_single_roundtrip() {
        let entry = ArchiveEntry {
            path: "data.bin".into(),
            data: b"xz contents".to_vec(),
        };
        let xz = archive_create(std::slice::from_ref(&entry), ArchiveFormat::Xz).unwrap();
        let extracted = archive_extract_as(&xz, ArchiveFormat::Xz).unwrap();
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].data, b"xz contents");
    }

    // ---- zst ----

    #[test]
    fn zst_roundtrip() {
        for data in [b"".as_ref(), b"a", b"hello world", b"\x00\x01\x02\xff"] {
            let compressed = zst_compress(data).unwrap();
            assert_eq!(zst_decompress(&compressed).unwrap(), data);
        }
    }

    #[test]
    fn detect_zst_magic() {
        let zst = archive_create(
            std::slice::from_ref(&ArchiveEntry {
                path: "x".into(),
                data: b"data".to_vec(),
            }),
            ArchiveFormat::Zst,
        )
        .unwrap();
        assert_eq!(detect_archive_format(&zst).unwrap(), ArchiveFormat::Zst);
    }

    #[test]
    fn zst_single_roundtrip() {
        let entry = ArchiveEntry {
            path: "data.bin".into(),
            data: b"zst contents".to_vec(),
        };
        let zst = archive_create(std::slice::from_ref(&entry), ArchiveFormat::Zst).unwrap();
        let extracted = archive_extract_as(&zst, ArchiveFormat::Zst).unwrap();
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].data, b"zst contents");
    }

    // ---- 单文件格式互转 ----

    #[test]
    fn convert_gz_to_xz() {
        let gz = archive_create(
            std::slice::from_ref(&ArchiveEntry {
                path: "x".into(),
                data: b"convert me".to_vec(),
            }),
            ArchiveFormat::Gz,
        )
        .unwrap();
        let xz = archive_convert(&gz, ArchiveFormat::Xz).unwrap();
        let extracted = archive_extract_as(&xz, ArchiveFormat::Xz).unwrap();
        assert_eq!(extracted[0].data, b"convert me");
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
        let extracted = archive_extract(&targz).unwrap();
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
