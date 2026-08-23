//! 通用文件转换路由:按源/目标格式分发到引擎或纯 Rust 转换。
//!
//! 用户心智是"源格式→目标格式"(对标 freeconvert),内部按 [`FormatFamily`] 路由:
//! 图像/归档/xlsx 走纯 Rust;Office/标记/电子书走引擎;PDF 按目标分流(压缩/提取/反向)。
//! 引擎产物路径由引擎决定(LibreOffice 自动生成 inputstem.<ext>),本模块负责落盘路径计算与重命名。

use crate::{ToolError, ToolResult};
use std::path::Path;

use super::{Engine, EngineRunner};

/// 文件格式族(按转换路径分流)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatFamily {
    Image,
    Archive,
    Office,
    Markup,
    Ebook,
    Pdf,
    Xlsx,
    Font,
    Svg,
}

/// 按扩展名(无点,小写)判定格式族
pub fn detect_family(ext: &str) -> Option<FormatFamily> {
    let e = ext.trim_start_matches('.').to_ascii_lowercase();
    Some(match e.as_str() {
        // 图像
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "tiff" | "tif" | "ico" | "dds" | "ff"
        | "farbfeld" | "hdr" | "exr" | "pnm" | "pgm" | "ppm" | "pbm" | "pam" | "qoi" | "tga" => {
            FormatFamily::Image
        }
        // 归档
        "zip" | "tar" | "gz" | "tgz" | "targz" | "bz2" | "xz" | "zst" | "7z" => {
            FormatFamily::Archive
        }
        // Office(LibreOffice 支持)
        "doc" | "docx" | "docm" | "dot" | "dotx" | "dotm" | "rtf" | "odt" | "wps" | "wpd"
        | "xls" | "xlsx" | "xlsm" | "xlt" | "xltx" | "ods" | "fods" | "csv" | "tsv" | "ppt"
        | "pptx" | "pps" | "ppsx" | "pot" | "potx" | "odp" | "key" => FormatFamily::Office,
        // 标记语言(pandoc)
        "md" | "markdown" | "html" | "htm" | "rst" | "adoc" | "asciidoc" | "org" | "tex"
        | "latex" | "wiki" | "mediawiki" | "textile" | "opml" | "ipynb" | "json" | "yaml"
        | "yml" | "toml" | "xml" => FormatFamily::Markup,
        // 电子书
        "epub" | "mobi" | "azw" | "azw3" | "fb2" | "lrf" | "lit" | "pdb" | "cbz" | "cbr" => {
            FormatFamily::Ebook
        }
        // PDF
        "pdf" => FormatFamily::Pdf,
        // XLSX(纯 Rust 读写)
        // (xlsx 已归 Office,纯 Rust 路径在 convert_file 内按 target 分流)
        // 字体
        "ttf" | "otf" | "woff" | "woff2" | "eot" => FormatFamily::Font,
        // 矢量
        "svg" | "svgz" => FormatFamily::Svg,
        _ => return None,
    })
}

/// 通用文件转换:按源格式族 + 目标格式路由到引擎或纯 Rust 转换,返回产物路径。
///
/// 引擎产物路径规则:LibreOffice 在 outdir 生成 `inputstem.<target>`,本函数计算该路径;
/// pandoc/calibre 直接输出到指定路径。同扩展名用 `_converted` 后缀避免覆盖源。
pub fn convert_any(input: &str, target: &str, runner: &dyn EngineRunner) -> ToolResult<String> {
    let input_path = Path::new(input);
    let input_ext = input_path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let family = detect_family(&input_ext)
        .ok_or_else(|| ToolError::InvalidInput(format!("无法识别源格式: .{input_ext}")))?;
    let target = target.trim_start_matches('.').to_ascii_lowercase();

    match family {
        FormatFamily::Office => convert_office(input, &target, runner),
        FormatFamily::Markup => convert_markup(input, &target, runner),
        FormatFamily::Ebook => convert_ebook(input, &target, runner),
        FormatFamily::Pdf => convert_pdf(input, &target, runner),
        FormatFamily::Image
        | FormatFamily::Archive
        | FormatFamily::Xlsx
        | FormatFamily::Font
        | FormatFamily::Svg => Err(ToolError::InvalidInput(
            "该格式族请用专项命令(图像/归档/字体/SVG 已有独立工具)".into(),
        )),
    }
}

/// Office 文档转换(LibreOffice):输出到 outdir,产物名 `inputstem.<target>`
fn convert_office(input: &str, target: &str, runner: &dyn EngineRunner) -> ToolResult<String> {
    let input_path = Path::new(input);
    let stem = input_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".into());
    let out_dir = input_path
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".into());
    // LibreOffice 产物路径:outdir/stem.target
    let out_path = format!("{out_dir}/{stem}.{target}");
    super::engine_convert(runner, Engine::LibreOffice, input, &out_path)?;
    Ok(out_path)
}

/// 标记语言转换(pandoc):直接输出到 stem.target
fn convert_markup(input: &str, target: &str, runner: &dyn EngineRunner) -> ToolResult<String> {
    let out_path = compute_output_path(input, target);
    super::engine_convert(runner, Engine::Pandoc, input, &out_path)?;
    Ok(out_path)
}

/// 电子书转换(calibre):直接输出到 stem.target
fn convert_ebook(input: &str, target: &str, runner: &dyn EngineRunner) -> ToolResult<String> {
    let out_path = compute_output_path(input, target);
    super::engine_convert(runner, Engine::Calibre, input, &out_path)?;
    Ok(out_path)
}

/// PDF 转换:按目标分流——pdf→pdf 压缩(Ghostscript);pdf→txt 纯 Rust 提取;其他→LibreOffice
fn convert_pdf(input: &str, target: &str, runner: &dyn EngineRunner) -> ToolResult<String> {
    match target {
        "pdf" => {
            // 同格式压缩:用 _converted 后缀
            let out_path = compute_output_path(input, target);
            super::engine_convert(runner, Engine::Ghostscript, input, &out_path)?;
            Ok(out_path)
        }
        "txt" => {
            // 纯 Rust 提取
            let out_path = compute_output_path(input, target);
            let data = std::fs::read(input)?;
            let text = super::pdf_to_text(&data)?;
            std::fs::write(&out_path, text.as_bytes())?;
            Ok(out_path)
        }
        _ => {
            // pdf→docx/xlsx 等走 LibreOffice
            let out_path = compute_output_path(input, target);
            super::engine_convert(runner, Engine::LibreOffice, input, &out_path)?;
            Ok(out_path)
        }
    }
}

/// 计算输出路径:源文件旁 `{stem}.{target}`,同扩展名用 `_converted` 后缀
fn compute_output_path(input: &str, target: &str) -> String {
    let p = Path::new(input);
    let dir = p
        .parent()
        .map(|x| x.to_string_lossy().into_owned())
        .unwrap_or_default();
    let stem = p
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".into());
    let input_ext = p
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    // 同扩展名加 _converted,避免覆盖源
    let name = if input_ext == target {
        // 已有 _converted 则不再叠加
        let base = stem.strip_suffix("_converted").unwrap_or(&stem);
        format!("{base}_converted.{target}")
    } else {
        format!("{stem}.{target}")
    };
    if dir.is_empty() {
        name
    } else {
        format!("{dir}/{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_image_family() {
        assert_eq!(detect_family("png"), Some(FormatFamily::Image));
        assert_eq!(detect_family("JPG"), Some(FormatFamily::Image));
        assert_eq!(detect_family(".webp"), Some(FormatFamily::Image));
    }

    #[test]
    fn detect_office_family() {
        assert_eq!(detect_family("docx"), Some(FormatFamily::Office));
        assert_eq!(detect_family("xlsx"), Some(FormatFamily::Office));
        assert_eq!(detect_family("pptx"), Some(FormatFamily::Office));
        assert_eq!(detect_family("odt"), Some(FormatFamily::Office));
        assert_eq!(detect_family("rtf"), Some(FormatFamily::Office));
    }

    #[test]
    fn detect_markup_family() {
        assert_eq!(detect_family("md"), Some(FormatFamily::Markup));
        assert_eq!(detect_family("html"), Some(FormatFamily::Markup));
        assert_eq!(detect_family("rst"), Some(FormatFamily::Markup));
        assert_eq!(detect_family("adoc"), Some(FormatFamily::Markup));
    }

    #[test]
    fn detect_ebook_family() {
        assert_eq!(detect_family("epub"), Some(FormatFamily::Ebook));
        assert_eq!(detect_family("mobi"), Some(FormatFamily::Ebook));
        assert_eq!(detect_family("azw3"), Some(FormatFamily::Ebook));
    }

    #[test]
    fn detect_pdf_family() {
        assert_eq!(detect_family("pdf"), Some(FormatFamily::Pdf));
    }

    #[test]
    fn detect_unknown_returns_none() {
        assert_eq!(detect_family("xyz"), None);
        assert_eq!(detect_family(""), None);
    }

    #[test]
    fn output_path_different_ext() {
        assert_eq!(compute_output_path("/a/b.md", "html"), "/a/b.html");
        assert_eq!(compute_output_path("doc.docx", "pdf"), "doc.pdf");
    }

    #[test]
    fn output_path_same_ext_uses_converted() {
        assert_eq!(compute_output_path("/a/b.pdf", "pdf"), "/a/b_converted.pdf");
        // 已有 _converted 不叠加
        assert_eq!(
            compute_output_path("/a/b_converted.pdf", "pdf"),
            "/a/b_converted.pdf"
        );
    }
}
