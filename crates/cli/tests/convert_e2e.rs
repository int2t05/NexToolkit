//! 通用文件转换端到端测试:真实调起 nextool file-conv convert + 真实引擎。
//! 引擎未装时 early-return(复用 engine_e2e 的 require_engine 宏思路)。

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

use lopdf::{dictionary, Document, Object};

fn nextool() -> Command {
    Command::cargo_bin("nextool").unwrap()
}

fn engine_available(binary: &str) -> bool {
    let out = nextool()
        .args(["file-conv", "engine", "list"])
        .assert()
        .success();
    let stdout = std::str::from_utf8(&out.get_output().stdout).unwrap();
    stdout
        .lines()
        .any(|line| line.starts_with("✓") && line.contains(binary))
}

macro_rules! require_engine {
    ($binary:expr) => {
        if !engine_available($binary) {
            eprintln!("跳过:引擎 {} 未安装", $binary);
            return;
        }
    };
}

/// 造测试 PDF(lopdf,1 页)
fn write_pdf(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let mut doc = Document::with_version("1.4");
    let pages_id = doc.new_object_id();
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
    });
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![Object::Reference(page_id)],
            "Count" => 1,
        }),
    );
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);
    let id: Vec<u8> = (0..16).collect();
    doc.trailer.set(
        "ID",
        vec![
            Object::String(id.clone(), lopdf::StringFormat::Hexadecimal),
            Object::String(id, lopdf::StringFormat::Hexadecimal),
        ],
    );
    let path = dir.join(name);
    doc.save(&path).unwrap();
    path
}

// ---- 未知格式拒绝 ----

#[test]
fn convert_unknown_format_fails() {
    let dir = tempdir().unwrap();
    let bad = dir.path().join("x.xyz");
    fs::write(&bad, b"unknown").unwrap();

    nextool()
        .args(["file-conv", "convert"])
        .arg(&bad)
        .arg("pdf")
        .assert()
        .failure();
}

// ---- 图像/归档/字体/SVG 拒绝(应走专项命令)----

#[test]
fn convert_image_rejected_to_specialized() {
    let dir = tempdir().unwrap();
    let png = dir.path().join("a.png");
    let img =
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(2, 2, image::Rgb([255, 0, 0])));
    img.save(&png).unwrap();

    // 图像应走 image convert 专项命令,通用 convert 拒绝
    nextool()
        .args(["file-conv", "convert"])
        .arg(&png)
        .arg("jpg")
        .assert()
        .failure();
}

// ---- PDF → TXT(纯 Rust,无引擎依赖)----
// 空内容流 PDF 正确报"未提取到文本"(非 bug,是 extract.rs 对无文本 PDF 的合理行为)
// 带文本 PDF 需 pandoc 造,归 convert_md_to_pdf 链路覆盖

#[test]
fn convert_pdf_to_text_empty_pdf_reports_error() {
    let dir = tempdir().unwrap();
    let pdf = write_pdf(dir.path(), "empty.pdf");

    // 空 PDF 无文本内容流,应失败(返回 InvalidInput 提示)
    nextool()
        .args(["file-conv", "convert"])
        .arg(&pdf)
        .arg("txt")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("未提取到文本"));
}

#[test]
fn convert_pdf_to_text_with_content() {
    // 需 pandoc 造带文本的 PDF
    require_engine!("pandoc");
    let dir = tempdir().unwrap();
    let md = dir.path().join("src.md");
    fs::write(&md, "# 有文本的 PDF\n\n正文内容 here\n").unwrap();
    nextool()
        .args(["file-conv", "convert"])
        .arg(&md)
        .arg("pdf")
        .current_dir(dir.path())
        .assert()
        .success();
    let pdf = dir.path().join("src.pdf");
    assert!(pdf.exists());

    // pdf → txt
    nextool()
        .args(["file-conv", "convert"])
        .arg(&pdf)
        .arg("txt")
        .current_dir(dir.path())
        .assert()
        .success();

    let txt = dir.path().join("src.txt");
    assert!(txt.exists());
}

// ---- PDF → PDF(压缩,需 Ghostscript)----

#[test]
fn convert_pdf_to_pdf_compress() {
    require_engine!("gswin64c");
    let dir = tempdir().unwrap();
    let pdf = write_pdf(dir.path(), "c.pdf");

    nextool()
        .args(["file-conv", "convert"])
        .arg(&pdf)
        .arg("pdf")
        .current_dir(dir.path())
        .assert()
        .success();

    // 同扩展名用 _converted
    let out = dir.path().join("c_converted.pdf");
    assert!(out.exists(), "应产出压缩 pdf");
    let bytes = fs::read(&out).unwrap();
    assert_eq!(&bytes[..4], b"%PDF", "产物应为 PDF");
}

// ---- MD → HTML(pandoc)----

#[test]
fn convert_md_to_html() {
    require_engine!("pandoc");
    let dir = tempdir().unwrap();
    let md = dir.path().join("note.md");
    fs::write(&md, "# Hello NexToolkit\n\n测试内容\n").unwrap();

    nextool()
        .args(["file-conv", "convert"])
        .arg(&md)
        .arg("html")
        .current_dir(dir.path())
        .assert()
        .success();

    let html = dir.path().join("note.html");
    assert!(html.exists());
    let content = fs::read_to_string(&html).unwrap();
    assert!(content.contains("<h1"), "html 应含 h1");
}

// ---- MD → DOCX(pandoc)----

#[test]
fn convert_md_to_docx() {
    require_engine!("pandoc");
    let dir = tempdir().unwrap();
    let md = dir.path().join("doc.md");
    fs::write(&md, "# Title\n\n正文\n").unwrap();

    nextool()
        .args(["file-conv", "convert"])
        .arg(&md)
        .arg("docx")
        .current_dir(dir.path())
        .assert()
        .success();

    let docx = dir.path().join("doc.docx");
    assert!(docx.exists());
    let bytes = fs::read(&docx).unwrap();
    assert_eq!(&bytes[..4], b"PK\x03\x04", "docx 应为 zip");
}

// ---- DOCX → PDF(LibreOffice,需先 pandoc 造 docx)----

#[test]
fn convert_docx_to_pdf() {
    require_engine!("soffice");
    require_engine!("pandoc");
    let dir = tempdir().unwrap();
    // pandoc 造 docx
    let md = dir.path().join("src.md");
    fs::write(&md, "# Office 转换测试\n\n内容\n").unwrap();
    nextool()
        .args(["file-conv", "convert"])
        .arg(&md)
        .arg("docx")
        .current_dir(dir.path())
        .assert()
        .success();
    let docx = dir.path().join("src.docx");
    assert!(docx.exists(), "pandoc 应造 docx");

    // docx → pdf
    nextool()
        .args(["file-conv", "convert"])
        .arg(&docx)
        .arg("pdf")
        .current_dir(dir.path())
        .assert()
        .success();

    let pdf = dir.path().join("src.pdf");
    assert!(pdf.exists(), "应产出 pdf");
    let bytes = fs::read(&pdf).unwrap();
    assert_eq!(&bytes[..4], b"%PDF", "产物应为 PDF");
}

// ---- HTML → EPUB(calibre)----

#[test]
fn convert_html_to_epub() {
    require_engine!("ebook-convert");
    let dir = tempdir().unwrap();
    let html = dir.path().join("book.html");
    fs::write(&html, "<html><body><h1>书</h1><p>内容</p></body></html>").unwrap();

    nextool()
        .args(["file-conv", "convert"])
        .arg(&html)
        .arg("epub")
        .current_dir(dir.path())
        .assert()
        .success();

    let epub = dir.path().join("book.epub");
    assert!(epub.exists());
    let bytes = fs::read(&epub).unwrap();
    assert_eq!(&bytes[..4], b"PK\x03\x04", "epub 应为 zip");
}
