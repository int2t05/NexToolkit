//! 文件转换端到端测试(补充 cli_smoke.rs):
//! 图像处理(裁剪/翻转/滤镜/亮度对比度/JPEG压缩)+ 图像新格式 + PDF 旋转角度 + 归档压缩文件夹。
//! 真实文件 IO,临时目录,程序化造图。

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

use image::GenericImageView;

fn nextool() -> Command {
    Command::cargo_bin("nextool").unwrap()
}

/// 造 4x3 红色 PNG 写入临时目录
fn write_png(dir: &std::path::Path, name: &str, w: u32, h: u32) -> std::path::PathBuf {
    let img =
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(w, h, image::Rgb([255, 0, 0])));
    let path = dir.join(name);
    img.save(&path).unwrap();
    path
}

// ---- 图像处理(I-18/19/21/23/28)----

#[test]
fn image_crop_produces_smaller() {
    let dir = tempdir().unwrap();
    let png = write_png(dir.path(), "src.png", 4, 3);

    nextool()
        .args(["file-conv", "image", "crop"])
        .arg(&png)
        .args(["--x", "0", "--y", "0", "--width", "2", "--height", "2"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("已"));

    let out = dir.path().join("src_converted.png");
    assert!(out.exists());
    assert_eq!(image::open(&out).unwrap().dimensions(), (2, 2));
}

#[test]
fn image_flip_horizontal() {
    let dir = tempdir().unwrap();
    let png = write_png(dir.path(), "flip.png", 4, 3);

    nextool()
        .args(["file-conv", "image", "flip"])
        .arg(&png)
        .args(["--direction", "h"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("flip_converted.png").exists());
}

#[test]
fn image_flip_vertical() {
    let dir = tempdir().unwrap();
    let png = write_png(dir.path(), "fv.png", 4, 3);

    nextool()
        .args(["file-conv", "image", "flip"])
        .arg(&png)
        .args(["--direction", "v"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("fv_converted.png").exists());
}

#[test]
fn image_filter_grayscale() {
    let dir = tempdir().unwrap();
    let png = write_png(dir.path(), "color.png", 4, 3);

    nextool()
        .args(["file-conv", "image", "filter"])
        .arg(&png)
        .args(["--filter", "grayscale"])
        .current_dir(dir.path())
        .assert()
        .success();

    // 灰度产物:RGB 三通道应相等(红色变灰后 R=G=B)
    let out = dir.path().join("color_converted.png");
    let img = image::open(&out).unwrap();
    let pixel = img.get_pixel(0, 0);
    assert_eq!(pixel.0[0], pixel.0[1]);
    assert_eq!(pixel.0[1], pixel.0[2]);
}

#[test]
fn image_filter_invert() {
    let dir = tempdir().unwrap();
    let png = write_png(dir.path(), "inv.png", 4, 3);

    nextool()
        .args(["file-conv", "image", "filter"])
        .arg(&png)
        .args(["--filter", "invert"])
        .current_dir(dir.path())
        .assert()
        .success();

    // 反相:红色 [255,0,0] → [0,255,255]
    let out = dir.path().join("inv_converted.png");
    let img = image::open(&out).unwrap();
    let pixel = img.get_pixel(0, 0);
    assert_eq!(pixel.0[0], 0);
    assert_eq!(pixel.0[1], 255);
}

#[test]
fn image_filter_blur() {
    let dir = tempdir().unwrap();
    let png = write_png(dir.path(), "blur.png", 4, 3);

    nextool()
        .args(["file-conv", "image", "filter"])
        .arg(&png)
        .args(["--filter", "blur"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("blur_converted.png").exists());
}

#[test]
fn image_adjust_brightness() {
    let dir = tempdir().unwrap();
    let png = write_png(dir.path(), "bright.png", 4, 3);

    nextool()
        .args(["file-conv", "image", "adjust"])
        .arg(&png)
        .args(["--brightness", "50", "--contrast", "1.0"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("bright_converted.png").exists());
}

#[test]
fn image_compress_jpeg_quality() {
    let dir = tempdir().unwrap();
    let png = write_png(dir.path(), "src.png", 8, 8);

    nextool()
        .args(["file-conv", "image", "compress-jpeg"])
        .arg(&png)
        .args(["--quality", "20"])
        .current_dir(dir.path())
        .assert()
        .success();

    let out = dir.path().join("src.jpg");
    assert!(out.exists());
    // 产物应为 JPEG(魔术字节)
    let bytes = fs::read(&out).unwrap();
    assert_eq!(&bytes[..3], &[0xFF, 0xD8, 0xFF]);
}

#[test]
fn image_compress_jpeg_quality_invalid() {
    let dir = tempdir().unwrap();
    let png = write_png(dir.path(), "bad.png", 4, 3);

    // quality > 100 应失败
    nextool()
        .args(["file-conv", "image", "compress-jpeg"])
        .arg(&png)
        .args(["--quality", "200"])
        .assert()
        .failure();
}

// ---- 图像新格式(I-02~08,QOI/PNM 可往返)----

#[test]
fn image_convert_png_to_qoi() {
    let dir = tempdir().unwrap();
    let png = write_png(dir.path(), "q.png", 4, 3);

    nextool()
        .args(["file-conv", "image", "convert"])
        .arg(&png)
        .arg("qoi")
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("q.qoi").exists());
}

#[test]
fn image_convert_png_to_pnm() {
    let dir = tempdir().unwrap();
    let png = write_png(dir.path(), "p.png", 4, 3);

    nextool()
        .args(["file-conv", "image", "convert"])
        .arg(&png)
        .arg("pnm")
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("p.pnm").exists());
}

#[test]
fn image_convert_to_dds_rejected() {
    // DDS 仅解码,作转换目标应失败
    let dir = tempdir().unwrap();
    let png = write_png(dir.path(), "d.png", 4, 3);

    nextool()
        .args(["file-conv", "image", "convert"])
        .arg(&png)
        .arg("dds")
        .assert()
        .failure();
}

// ---- PDF 旋转角度(P-06)----

use lopdf::{dictionary, Document, Object};

/// 造 N 页 PDF(lopdf,同 cli_smoke 模式)
fn write_pdf(dir: &std::path::Path, name: &str, pages: u32) -> std::path::PathBuf {
    let mut doc = Document::with_version("1.4");
    let pages_id = doc.new_object_id();
    let mut page_ids = Vec::new();
    for _ in 0..pages {
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        });
        page_ids.push(page_id);
    }
    let kids: Vec<Object> = page_ids.into_iter().map(Object::Reference).collect();
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => pages,
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

#[test]
fn pdf_rotate_180_degrees() {
    let dir = tempdir().unwrap();
    let pdf = write_pdf(dir.path(), "r.pdf", 1);

    nextool()
        .args(["file-conv", "pdf", "rotate"])
        .arg(&pdf)
        .args(["--degrees", "180"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("已旋转"));

    let out = dir.path().join("r_converted.pdf");
    assert!(out.exists());
    // 验证 /Rotate 为 180
    let doc = Document::load(&out).unwrap();
    let page_id = *doc.get_pages().values().next().unwrap();
    if let Object::Dictionary(d) = doc.get_object(page_id).unwrap() {
        if let Ok(Object::Integer(i)) = d.get(b"Rotate") {
            assert_eq!(*i, 180);
        }
    }
}

#[test]
fn pdf_rotate_270_degrees() {
    let dir = tempdir().unwrap();
    let pdf = write_pdf(dir.path(), "r.pdf", 1);

    nextool()
        .args(["file-conv", "pdf", "rotate"])
        .arg(&pdf)
        .args(["--degrees", "270"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("r_converted.pdf").exists());
}

#[test]
fn pdf_rotate_invalid_degrees_fails() {
    let dir = tempdir().unwrap();
    let pdf = write_pdf(dir.path(), "bad.pdf", 1);

    // 45 度应失败(只支持 90/180/270)
    nextool()
        .args(["file-conv", "pdf", "rotate"])
        .arg(&pdf)
        .args(["--degrees", "45"])
        .assert()
        .failure();
}

// ---- 归档压缩文件夹(修 bug 后验证)----

#[test]
fn archive_compress_directory_preserves_structure() {
    let dir = tempdir().unwrap();
    // 造文件夹结构:root/a.txt + root/sub/b.txt
    let root = dir.path().join("root");
    let sub = root.join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(root.join("a.txt"), "aaa").unwrap();
    fs::write(sub.join("b.txt"), "bbb").unwrap();

    nextool()
        .args(["file-conv", "archive", "compress", "zip"])
        .arg(&root)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("已创建归档"));

    let zip = dir.path().join("root.zip");
    assert!(zip.exists(), "压缩文件夹应产出归档");

    // 列出应含目录结构
    nextool()
        .args(["file-conv", "archive", "list"])
        .arg(&zip)
        .assert()
        .success()
        .stdout(predicate::str::contains("root/a.txt"))
        .stdout(predicate::str::contains("root/sub/b.txt"));
}

#[test]
fn archive_compress_directory_extract_restores() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("data");
    fs::create_dir_all(root.join("nested")).unwrap();
    fs::write(root.join("top.txt"), "top").unwrap();
    fs::write(root.join("nested/inner.txt"), "inner").unwrap();

    // 压缩文件夹
    nextool()
        .args(["file-conv", "archive", "compress", "tar"])
        .arg(&root)
        .current_dir(dir.path())
        .assert()
        .success();

    let tar = dir.path().join("data.tar");
    // 解压到独立目录,验证结构还原
    let out = dir.path().join("restored");
    nextool()
        .args(["file-conv", "archive", "extract"])
        .arg(&tar)
        .arg("--output-dir")
        .arg(&out)
        .assert()
        .success();

    assert!(out.join("data/top.txt").exists(), "应还原 top.txt");
    assert!(out.join("data/nested/inner.txt").exists(), "应还原嵌套文件");
}
