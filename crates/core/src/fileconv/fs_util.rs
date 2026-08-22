//! 文件 IO 边界:转换产物落盘与碰撞处理(依赖 std::fs,非纯内存)
//!
//! 组合各域纯内存转换 + 文件读写 + [`super::path`] 路径计算,供 CLI/GUI 共享,避免边界逻辑重复。
//! 产物默认落源文件所在目录:解压到 `{stem}_extracted/`,转换到 `{stem}.{ext}`,
//! 碰撞追加 `_converted`→`(1)`→`(2)`,`create_new` 原子检查不静默覆盖。

use super::{Engine, EngineRunner};
use crate::{ToolError, ToolResult};
use std::io::Write;
use std::path::Path;

// ---- 归档 IO(archive feature)----

#[cfg(feature = "archive")]
use super::archive::{ArchiveEntry, ArchiveFormat, ArchiveListEntry};

/// 解压归档文件到目录(默认源文件旁 `_extracted`,碰撞追加 `(n)`)
///
/// 返回 `(输出目录, 写出的文件路径列表)`。
#[cfg(feature = "archive")]
pub fn extract_to_dir(
    archive_path: &str,
    output_dir: Option<&str>,
) -> ToolResult<(String, Vec<String>)> {
    let data = std::fs::read(archive_path)?;
    let entries = super::archive_extract(&data)?;
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

/// 压缩文件/目录为归档并落盘(默认输出到第一个路径旁),返回产物路径
///
/// 文件直接读;目录递归遍历,归档内保留相对路径结构(以输入路径的末段名为根),
/// 解压可还原目录层级。entry 路径用 `/` 分隔,经 [`super::archive_create`] 安全校验。
#[cfg(feature = "archive")]
pub fn compress_files(
    paths: &[String],
    format: ArchiveFormat,
    output: Option<&str>,
) -> ToolResult<String> {
    let mut entries = Vec::new();
    for p in paths {
        collect_entries(p, &mut entries)?;
    }
    let archive = super::archive_create(&entries, format)?;
    write_output(&archive, &paths[0], output, archive_ext(format))
}

/// 收集单个路径的归档条目:文件直接读;目录递归遍历
#[cfg(feature = "archive")]
fn collect_entries(input: &str, entries: &mut Vec<ArchiveEntry>) -> ToolResult<()> {
    let root = Path::new(input);
    let root_name = root
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| input.trim_end_matches(['/', '\\']).to_string());
    let meta = std::fs::metadata(input)?;
    if meta.is_file() {
        let data = std::fs::read(input)?;
        entries.push(ArchiveEntry {
            path: root_name,
            data,
        });
    } else if meta.is_dir() {
        collect_dir(root, &root_name, entries)?;
    } else {
        return Err(ToolError::InvalidInput(format!(
            "不支持的路径类型(非文件/目录): {input}"
        )));
    }
    Ok(())
}

/// 递归遍历目录:每个文件生成一条目,路径为 `prefix/相对子路径`(归档内统一 `/` 分隔)
#[cfg(feature = "archive")]
fn collect_dir(dir: &Path, prefix: &str, entries: &mut Vec<ArchiveEntry>) -> ToolResult<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let rel = format!("{prefix}/{name}");
        let meta = entry.metadata()?;
        if meta.is_file() {
            let data = std::fs::read(entry.path())?;
            entries.push(ArchiveEntry { path: rel, data });
        } else if meta.is_dir() {
            collect_dir(&entry.path(), &rel, entries)?;
        }
    }
    Ok(())
}

/// 归档互转并落盘(默认输出到源文件旁),返回产物路径
#[cfg(feature = "archive")]
pub fn convert_file(
    archive_path: &str,
    target: ArchiveFormat,
    output: Option<&str>,
) -> ToolResult<String> {
    let data = std::fs::read(archive_path)?;
    let out_data = super::archive_convert(&data, target)?;
    write_output(&out_data, archive_path, output, archive_ext(target))
}

#[cfg(feature = "archive")]
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

#[cfg(feature = "archive")]
fn archive_ext(fmt: ArchiveFormat) -> &'static str {
    match fmt {
        ArchiveFormat::Zip => "zip",
        ArchiveFormat::Tar => "tar",
        ArchiveFormat::TarGz => "tar.gz",
        ArchiveFormat::Gz => "gz",
        ArchiveFormat::SevenZ => "7z",
        ArchiveFormat::Bz2 => "bz2",
        ArchiveFormat::Xz => "xz",
        ArchiveFormat::Zst => "zst",
    }
}

/// 列出归档内文件(读盘 → 调纯 archive_list_entries),返回结构化条目列表
#[cfg(feature = "archive")]
pub fn list_archive_file(path: &str) -> ToolResult<Vec<ArchiveListEntry>> {
    let data = std::fs::read(path)?;
    super::archive_list_entries(&data)
}

// ---- 图像 IO(image feature)----

#[cfg(feature = "image")]
use super::image::ImageFormat;

/// 图像格式互转并落盘(默认输出到源文件旁),返回产物路径
#[cfg(feature = "image")]
pub fn convert_image_file(
    input: &str,
    target: ImageFormat,
    output: Option<&str>,
) -> ToolResult<String> {
    let data = std::fs::read(input)?;
    let out_data = super::image_convert(&data, target)?;
    write_output(&out_data, input, output, target.ext())
}

/// 图像缩放并落盘(默认输出到源文件旁,同格式),返回产物路径
///
/// `width`/`height` 一维为 0 时按另一维等比缩放;目标格式与源相同。
#[cfg(feature = "image")]
pub fn resize_image_file(
    input: &str,
    width: u32,
    height: u32,
    output: Option<&str>,
) -> ToolResult<String> {
    let data = std::fs::read(input)?;
    let fmt = super::detect_image_format(&data)?;
    let out_data = super::image_resize(&data, width, height, fmt)?;
    write_output(&out_data, input, output, fmt.ext())
}

/// 图像裁剪并落盘(默认输出到源文件旁,同格式),返回产物路径
#[cfg(feature = "image")]
pub fn crop_image_file(
    input: &str,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    output: Option<&str>,
) -> ToolResult<String> {
    let data = std::fs::read(input)?;
    let fmt = super::detect_image_format(&data)?;
    let out_data = super::image_crop(&data, x, y, width, height, fmt)?;
    write_output(&out_data, input, output, fmt.ext())
}

/// 图像翻转并落盘(默认输出到源文件旁,同格式),返回产物路径
#[cfg(feature = "image")]
pub fn flip_image_file(
    input: &str,
    direction: super::image::FlipDirection,
    output: Option<&str>,
) -> ToolResult<String> {
    let data = std::fs::read(input)?;
    let fmt = super::detect_image_format(&data)?;
    let out_data = super::image_flip(&data, direction, fmt)?;
    write_output(&out_data, input, output, fmt.ext())
}

/// 图像滤镜并落盘(默认输出到源文件旁,同格式),返回产物路径
#[cfg(feature = "image")]
pub fn filter_image_file(
    input: &str,
    filter: super::image::FilterKind,
    output: Option<&str>,
) -> ToolResult<String> {
    let data = std::fs::read(input)?;
    let fmt = super::detect_image_format(&data)?;
    let out_data = super::image_filter(&data, filter, fmt)?;
    write_output(&out_data, input, output, fmt.ext())
}

/// 亮度/对比度调整并落盘(默认输出到源文件旁,同格式),返回产物路径
#[cfg(feature = "image")]
pub fn adjust_image_file(
    input: &str,
    brightness: i32,
    contrast: f32,
    output: Option<&str>,
) -> ToolResult<String> {
    let data = std::fs::read(input)?;
    let fmt = super::detect_image_format(&data)?;
    let out_data = super::image_adjust(&data, brightness, contrast, fmt)?;
    write_output(&out_data, input, output, fmt.ext())
}

/// JPEG 压缩并落盘(输出 JPEG 格式,默认源文件旁 .jpg),返回产物路径
#[cfg(feature = "image")]
pub fn compress_jpeg_image_file(
    input: &str,
    quality: u8,
    output: Option<&str>,
) -> ToolResult<String> {
    let data = std::fs::read(input)?;
    let out_data = super::image_compress_jpeg(&data, quality)?;
    write_output(&out_data, input, output, "jpg")
}

// ---- PDF IO(pdf feature)----

/// 拆分 PDF:每页一个独立 PDF,输出到源文件旁 `{stem}_split_{n}.pdf`,返回产物路径列表
#[cfg(feature = "pdf")]
pub fn split_pdf(input: &str, output_dir: Option<&str>) -> ToolResult<Vec<String>> {
    let data = std::fs::read(input)?;
    let parts = super::pdf_split(&data)?;
    if parts.is_empty() {
        return Ok(Vec::new());
    }
    let dir = match output_dir {
        Some(d) => {
            std::fs::create_dir_all(d)?;
            d.to_string()
        }
        None => resolve_extract_dir(input, None)?,
    };
    let stem = Path::new(input)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "split".into());
    let mut written = Vec::with_capacity(parts.len());
    for (i, part) in parts.iter().enumerate() {
        // 碰撞:_split_0 → _split_0(1) ...
        let base = format!("{dir}/{}_split_{}", stem, i + 1);
        let path = write_bytes_safe(part, &format!("{base}.pdf"), "pdf")?;
        written.push(path);
    }
    Ok(written)
}

/// 旋转 PDF 指定角度并落盘(默认输出到源文件旁),返回产物路径
///
/// `degrees` 须为 90/180/270。
#[cfg(feature = "pdf")]
pub fn rotate_pdf(input: &str, degrees: u32, output: Option<&str>) -> ToolResult<String> {
    let data = std::fs::read(input)?;
    let out_data = super::pdf_rotate(&data, degrees)?;
    write_output(&out_data, input, output, "pdf")
}

/// 加密 PDF 并落盘(默认输出到源文件旁),返回产物路径
#[cfg(feature = "pdf")]
pub fn encrypt_pdf(input: &str, password: &str, output: Option<&str>) -> ToolResult<String> {
    let data = std::fs::read(input)?;
    let out_data = super::pdf_encrypt(&data, password)?;
    write_output(&out_data, input, output, "pdf")
}

/// 解密 PDF 并落盘(默认输出到源文件旁),返回产物路径
#[cfg(feature = "pdf")]
pub fn decrypt_pdf(input: &str, password: &str, output: Option<&str>) -> ToolResult<String> {
    let data = std::fs::read(input)?;
    let out_data = super::pdf_decrypt(&data, password)?;
    write_output(&out_data, input, output, "pdf")
}

/// 检查 PDF 文件是否已加密(读盘 → 调纯 pdf_is_encrypted)
#[cfg(feature = "pdf")]
pub fn is_pdf_encrypted_file(path: &str) -> ToolResult<bool> {
    let data = std::fs::read(path)?;
    super::pdf_is_encrypted(&data)
}

// ---- 字体 IO(font feature)----

#[cfg(feature = "font")]
use super::font::FontFormat;

/// 字体格式互转并落盘(默认输出到源文件旁),返回产物路径
#[cfg(feature = "font")]
pub fn convert_font_file(
    input: &str,
    target: FontFormat,
    output: Option<&str>,
) -> ToolResult<String> {
    let data = std::fs::read(input)?;
    let out_data = super::font_convert(&data, target)?;
    write_output(&out_data, input, output, target.ext())
}

/// 读取字体文件元数据(读盘 → 调纯 font_metadata → 格式化文本)
#[cfg(feature = "font")]
pub fn read_font_meta_file(input: &str) -> ToolResult<String> {
    let data = std::fs::read(input)?;
    let meta = super::font_metadata(&data)?;
    Ok(super::font_meta_to_text(&meta))
}

// ---- SVG IO(svg feature)----

#[cfg(feature = "svg")]
use super::svg::SvgFormat;

/// SVG 栅格化并落盘(默认输出到源文件旁),返回产物路径
#[cfg(feature = "svg")]
pub fn convert_svg_file(
    input: &str,
    target: SvgFormat,
    output: Option<&str>,
) -> ToolResult<String> {
    let data = std::fs::read(input)?;
    let out_data = super::svg_render(&data, target)?;
    write_output(&out_data, input, output, target.ext())
}

// ---- 引擎 IO ----

/// 引擎转换并落盘(默认输出到源文件旁),返回产物路径
///
/// 引擎直接操作文件路径(非字节域):计算输出路径 → 检查可用性 → 委托执行。
/// 显式 `output` 用之;否则源文件旁 `{stem}.{output_ext}`(引擎默认覆盖,与字节域碰撞策略不同)。
pub fn engine_convert_file(
    input: &str,
    engine: Engine,
    output_ext: &str,
    output: Option<&str>,
    runner: &dyn EngineRunner,
) -> ToolResult<String> {
    let out_path = match output {
        Some(o) => o.to_string(),
        None => {
            // 默认输出源文件旁;若同扩展名会覆盖源文件(如 pdf→pdf 压缩),用 _converted 后缀避开
            let attempt = if same_ext(input, output_ext) { 1 } else { 0 };
            super::path::compute_output_path(input, output_ext, attempt)
        }
    };
    super::engine_convert(runner, engine, input, &out_path)?;
    Ok(out_path)
}

/// 输入路径末段扩展名是否与目标扩展名相同(忽略大小写)
fn same_ext(input: &str, ext: &str) -> bool {
    Path::new(input)
        .extension()
        .map(|e| e.eq_ignore_ascii_case(ext))
        .unwrap_or(false)
}

// ---- 通用 IO helpers(无 feature gate)----

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

/// 迭代碰撞后缀直到创建成功:路径由 `path_fn` 生成,创建由 `op` 执行;超过 100 次报错
fn retry_unique<F, O>(path_fn: F, op: O, collision_msg: &str) -> ToolResult<String>
where
    F: Fn(u32) -> String,
    O: Fn(&str) -> Result<(), std::io::Error>,
{
    for attempt in 0..100u32 {
        let path = path_fn(attempt);
        match op(&path) {
            Ok(()) => return Ok(path),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(ToolError::Io(e)),
        }
    }
    Err(ToolError::Other(collision_msg.into()))
}

/// 统一产物落盘:显式 `output` 路径则原子写入;否则碰撞重试写源文件旁
fn write_output(
    data: &[u8],
    input_path: &str,
    output: Option<&str>,
    target_ext: &str,
) -> ToolResult<String> {
    match output {
        Some(o) => {
            write_bytes_create_new(data, o)?;
            Ok(o.to_string())
        }
        None => write_bytes_safe(data, input_path, target_ext),
    }
}

/// 迭代碰撞后缀写入直到成功,返回最终路径;超过 100 次报错
fn write_bytes_safe(data: &[u8], input_path: &str, target_ext: &str) -> ToolResult<String> {
    retry_unique(
        |a| super::path::compute_output_path(input_path, target_ext, a),
        |p| write_bytes_create_new(data, p),
        "碰撞次数过多:同名产物已存在 100 个",
    )
}

/// 确定解压/拆分输出目录:显式指定用之;否则源文件旁 `{stem}_extracted`,碰撞追加 `(n)`
#[cfg(any(feature = "archive", feature = "pdf"))]
fn resolve_extract_dir(input: &str, output_dir: Option<&str>) -> ToolResult<String> {
    match output_dir {
        Some(d) => {
            std::fs::create_dir_all(d)?;
            Ok(d.to_string())
        }
        None => retry_unique(
            |a| super::path::compute_extract_dir(input, a),
            |p| std::fs::create_dir(p),
            "碰撞次数过多:同名解压目录已存在 100 个",
        ),
    }
}

#[cfg(all(test, feature = "archive"))]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// 临时目录隔离:每个测试唯一前缀,结束清理
    struct TempDir(PathBuf);
    impl TempDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("nextool_{name}"));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            TempDir(path)
        }
        fn join(&self, rel: &str) -> String {
            self.0.join(rel).to_string_lossy().into_owned()
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn compress_directory_preserves_structure() {
        let tmp = TempDir::new("compress_dir");
        fs::create_dir_all(tmp.0.join("root/sub")).unwrap();
        fs::write(tmp.0.join("root/a.txt"), b"aaa").unwrap();
        fs::write(tmp.0.join("root/sub/b.txt"), b"bbb").unwrap();

        let input = tmp.join("root");
        let out = compress_files(&[input], ArchiveFormat::Zip, None).unwrap();

        let data = fs::read(&out).unwrap();
        let entries = crate::archive_extract(&data).unwrap();
        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        assert!(paths.contains(&"root/a.txt"), "缺 root/a.txt: {paths:?}");
        assert!(
            paths.contains(&"root/sub/b.txt"),
            "缺 root/sub/b.txt: {paths:?}"
        );
    }

    #[test]
    fn compress_single_file_keeps_basename() {
        let tmp = TempDir::new("compress_single");
        fs::write(tmp.0.join("a.txt"), b"aaa").unwrap();

        let input = tmp.join("a.txt");
        let out = compress_files(&[input], ArchiveFormat::Zip, None).unwrap();

        let data = fs::read(&out).unwrap();
        let entries = crate::archive_extract(&data).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "a.txt");
        assert_eq!(entries[0].data, b"aaa");
    }

    #[test]
    fn compress_mixed_files_and_dirs() {
        let tmp = TempDir::new("compress_mixed");
        fs::write(tmp.0.join("top.txt"), b"t").unwrap();
        fs::create_dir_all(tmp.0.join("d")).unwrap();
        fs::write(tmp.0.join("d/inner.txt"), b"i").unwrap();

        let out = compress_files(
            &[tmp.join("top.txt"), tmp.join("d")],
            ArchiveFormat::Tar,
            None,
        )
        .unwrap();

        let data = fs::read(&out).unwrap();
        let entries = crate::archive_extract(&data).unwrap();
        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        assert!(paths.contains(&"top.txt"), "缺 top.txt: {paths:?}");
        assert!(paths.contains(&"d/inner.txt"), "缺 d/inner.txt: {paths:?}");
    }
}
