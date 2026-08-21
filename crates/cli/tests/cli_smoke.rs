//! CLI 集成测试:真实调起 nextool 二进制,验证子命令路由、stdin 输入、退出码与错误路径
//! 禁止 mock,真实调用真实数据

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_lists_all_domains() {
    // 顶层 help 应列出七个域
    Command::cargo_bin("nextool")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("encode"))
        .stdout(predicate::str::contains("convert"))
        .stdout(predicate::str::contains("format"))
        .stdout(predicate::str::contains("generate"))
        .stdout(predicate::str::contains("text"))
        .stdout(predicate::str::contains("crypto"))
        .stdout(predicate::str::contains("net-time"));
}

#[test]
fn base64_encode_arg() {
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["encode", "base64", "encode", "Hello"])
        .assert()
        .success()
        .stdout("SGVsbG8=\n");
}

#[test]
fn base64_decode_stdin() {
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["encode", "base64", "decode"])
        .write_stdin("SGVsbG8=")
        .assert()
        .success()
        .stdout("Hello\n");
}

#[test]
fn base64_invalid_nonzero_exit() {
    // 非法输入应非零退出码 + stderr 错误
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["encode", "base64", "decode", "!!!!"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("错误"));
}

#[test]
fn json_to_yaml_stdin() {
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["convert", "json-yaml", "to"])
        .write_stdin("{\"a\":1,\"b\":[2,3]}")
        .assert()
        .success()
        .stdout(predicate::str::contains("a: 1"))
        .stdout(predicate::str::contains("- 2"));
}

#[test]
fn numbase_convert() {
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["convert", "numbase", "16", "10", "ff"])
        .assert()
        .success()
        .stdout("255\n");
}

#[test]
fn hash_sha256() {
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["generate", "hash", "sha256", "abc"])
        .assert()
        .success()
        .stdout("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\n");
}

#[test]
fn case_snake() {
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["text", "case", "snake", "Hello World"])
        .assert()
        .success()
        .stdout("hello_world\n");
}

#[test]
fn regex_match() {
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["text", "regex-match", r"\d+", "a12b3"])
        .assert()
        .success()
        .stdout("12\n3\n");
}

#[test]
fn aes_gcm_roundtrip() {
    // 加密后解密应还原明文
    let enc = Command::cargo_bin("nextool")
        .unwrap()
        .args(["crypto", "aes-encrypt", "--password", "pw", "Secret"])
        .assert()
        .success();
    let ciphertext = std::str::from_utf8(&enc.get_output().stdout)
        .unwrap()
        .trim();

    Command::cargo_bin("nextool")
        .unwrap()
        .args(["crypto", "aes-decrypt", "--password", "pw", ciphertext])
        .assert()
        .success()
        .stdout("Secret\n");
}

#[test]
fn aes_gcm_wrong_password_fails() {
    // 错误口令应解密失败(非零退出)
    let enc = Command::cargo_bin("nextool")
        .unwrap()
        .args(["crypto", "aes-encrypt", "--password", "pw", "Secret"])
        .assert()
        .success();
    let ciphertext = std::str::from_utf8(&enc.get_output().stdout)
        .unwrap()
        .trim();

    Command::cargo_bin("nextool")
        .unwrap()
        .args(["crypto", "aes-decrypt", "--password", "wrong", ciphertext])
        .assert()
        .failure();
}

#[test]
fn timestamp_roundtrip() {
    // ts -> human -> ts 应一致(Asia/Shanghai +08)
    Command::cargo_bin("nextool")
        .unwrap()
        .args([
            "net-time",
            "ts-to-human",
            "1700000000",
            "--tz",
            "Asia/Shanghai",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("2023-11-15T06:13:20+08:00"));

    Command::cargo_bin("nextool")
        .unwrap()
        .args([
            "net-time",
            "ts-from-human",
            "2023-11-15 06:13:20",
            "--tz",
            "Asia/Shanghai",
        ])
        .assert()
        .success()
        .stdout("1700000000\n");
}

// ---- 文件转换:归档(真实文件 IO,临时目录,禁止 mock)----

use std::fs;
use tempfile::tempdir;

#[test]
fn help_lists_fileconv_domain() {
    Command::cargo_bin("nextool")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("file-conv"));
}

#[test]
fn archive_compress_then_list_zip() {
    let dir = tempdir().unwrap();
    let a = dir.path().join("a.txt");
    fs::write(&a, "hello").unwrap();
    let b = dir.path().join("b.txt");
    fs::write(&b, "world").unwrap();

    // 压缩为 zip(输出到第一个文件旁:a.zip)
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "archive", "compress", "zip", "a.txt", "b.txt"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("已创建归档"));

    let zip = dir.path().join("a.zip");
    assert!(zip.exists(), "产物应落源文件所在目录");

    // 列出 zip 内容
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "archive", "list"])
        .arg(&zip)
        .assert()
        .success()
        .stdout(predicate::str::contains("a.txt"))
        .stdout(predicate::str::contains("b.txt"));
}

#[test]
fn archive_extract_writes_files_to_source_dir() {
    let dir = tempdir().unwrap();
    // 先造一个 zip
    let a = dir.path().join("a.txt");
    fs::write(&a, "hello").unwrap();
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "archive", "compress", "zip", "a.txt"])
        .current_dir(dir.path())
        .assert()
        .success();
    let zip = dir.path().join("a.zip");

    // 在子目录解压,避免与源 a.txt 同级干扰
    let sub = dir.path().join("sub");
    fs::create_dir(&sub).unwrap();
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "archive", "extract"])
        .arg(&zip)
        .arg("--output-dir")
        .arg(sub.join("out"))
        .assert()
        .success()
        .stdout(predicate::str::contains("已解压"));

    let extracted = sub.join("out").join("a.txt");
    assert!(extracted.exists(), "解压应写出文件");
    assert_eq!(fs::read_to_string(&extracted).unwrap(), "hello");
}

#[test]
fn archive_convert_zip_to_tar() {
    let dir = tempdir().unwrap();
    let a = dir.path().join("a.txt");
    fs::write(&a, "data").unwrap();
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "archive", "compress", "zip", "a.txt"])
        .current_dir(dir.path())
        .assert()
        .success();
    let zip = dir.path().join("a.zip");

    // 转 tar(输出到源文件旁:a.tar)
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "archive", "convert"])
        .arg(&zip)
        .arg("tar")
        .current_dir(dir.path())
        .assert()
        .success();

    let tar = dir.path().join("a.tar");
    assert!(tar.exists(), "转换产物应落源文件所在目录");

    // 列出 tar 验证内容
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "archive", "list"])
        .arg(&tar)
        .assert()
        .success()
        .stdout(predicate::str::contains("a.txt"));
}

#[test]
fn archive_extract_collision_appends_suffix() {
    let dir = tempdir().unwrap();
    // 造两个同名 a.txt 的不同内容 zip,解压第二次应产生 _extracted(1)
    let mk = |name: &str| {
        let p = dir.path().join(name);
        fs::write(&p, name).unwrap();
        p
    };
    let a1 = mk("a.txt");
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "archive", "compress", "zip", "a.txt"])
        .current_dir(dir.path())
        .assert()
        .success();
    let zip = dir.path().join("a.zip");

    // 第一次解压:默认 a_extracted
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "archive", "extract"])
        .arg(&zip)
        .current_dir(dir.path())
        .assert()
        .success();
    let first = dir.path().join("a_extracted").join("a.txt");
    assert!(first.exists());

    // 第二次解压:碰撞 → a_extracted(1)
    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "archive", "extract"])
        .arg(&zip)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("a_extracted(1)"));

    let second = dir.path().join("a_extracted(1)").join("a.txt");
    assert!(second.exists(), "碰撞应追加 (1) 后缀");
    let _ = a1;
}

// ---- 文件转换:图像(真实文件 IO,临时目录,程序化造 PNG)----

use image::GenericImageView;

/// 造一个 4x3 红色 PNG 写入临时目录,返回路径
fn write_sample_png(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let img =
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(4, 3, image::Rgb([255, 0, 0])));
    let path = dir.join(name);
    img.save(&path).unwrap();
    path
}

#[test]
fn image_convert_png_to_jpeg() {
    let dir = tempdir().unwrap();
    let png = write_sample_png(dir.path(), "src.png");

    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "image", "convert"])
        .arg(&png)
        .arg("jpg")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("已转换"));

    let jpeg = dir.path().join("src.jpg");
    assert!(jpeg.exists(), "产物应落源文件所在目录");

    // 验证产物确为 JPEG(魔术字节 FF D8 FF)
    let bytes = fs::read(&jpeg).unwrap();
    assert_eq!(&bytes[..3], &[0xFF, 0xD8, 0xFF]);
}

#[test]
fn image_convert_png_to_webp() {
    let dir = tempdir().unwrap();
    let png = write_sample_png(dir.path(), "img.png");

    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "image", "convert"])
        .arg(&png)
        .arg("webp")
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("img.webp").exists());
}

#[test]
fn image_resize_smaller() {
    let dir = tempdir().unwrap();
    let png = write_sample_png(dir.path(), "big.png");

    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "image", "resize"])
        .arg(&png)
        .args(["--width", "2", "--height", "2"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("已缩放"));

    // 产物落源目录,同格式 PNG 但与源同名碰撞 → big_converted.png,尺寸 2x2
    let out = dir.path().join("big_converted.png");
    let resized = image::open(&out).unwrap();
    assert_eq!(resized.dimensions(), (2, 2));
}

#[test]
fn image_resize_keep_aspect() {
    let dir = tempdir().unwrap();
    // 4x3 → width=8,height=0 应等比到 8x6
    let png = write_sample_png(dir.path(), "orig.png");

    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "image", "resize"])
        .arg(&png)
        .args(["--width", "8", "--height", "0"])
        .current_dir(dir.path())
        .assert()
        .success();

    let out = dir.path().join("orig_converted.png");
    let resized = image::open(&out).unwrap();
    assert_eq!(resized.dimensions(), (8, 6));
}

#[test]
fn image_convert_invalid_input_fails() {
    let dir = tempdir().unwrap();
    let bad = dir.path().join("not.png");
    fs::write(&bad, b"not an image").unwrap();

    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "image", "convert"])
        .arg(&bad)
        .arg("jpg")
        .assert()
        .failure();
}

// ---- 文件转换:PDF(真实文件 IO,lopdf 造 fixture)----

use lopdf::{dictionary, Document, Object};

/// 用 lopdf 构造 N 页合法 PDF 写入临时目录
fn write_sample_pdf(dir: &std::path::Path, name: &str, pages: u32) -> std::path::PathBuf {
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
fn pdf_split_creates_per_page_files() {
    let dir = tempdir().unwrap();
    let pdf = write_sample_pdf(dir.path(), "doc.pdf", 3);

    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "pdf", "split"])
        .arg(&pdf)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("已拆分为 3 个"));

    // 产物落源文件旁 _extracted 目录,3 个单页 PDF
    let split_dir = dir.path().join("doc_extracted");
    let count = fs::read_dir(&split_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "pdf"))
        .count();
    assert_eq!(count, 3, "应产出 3 个 PDF");
}

#[test]
fn pdf_rotate_produces_output() {
    let dir = tempdir().unwrap();
    let pdf = write_sample_pdf(dir.path(), "r.pdf", 1);

    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "pdf", "rotate"])
        .arg(&pdf)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("已旋转"));

    // 产物落源文件旁(同格式碰撞 → _converted)
    assert!(dir.path().join("r_converted.pdf").exists());
}

#[test]
fn pdf_encrypt_marks_encrypted() {
    let dir = tempdir().unwrap();
    let pdf = write_sample_pdf(dir.path(), "e.pdf", 1);

    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "pdf", "encrypt"])
        .arg(&pdf)
        .args(["--password", "secret"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("已加密"));

    let enc = dir.path().join("e_converted.pdf");
    assert!(enc.exists());
    // 产物应声明加密(/Encrypt 在 trailer)
    let meta = Document::load_metadata_mem(&fs::read(&enc).unwrap()).unwrap();
    assert!(meta.encrypted, "加密产物应声明 /Encrypt");
}

#[test]
fn pdf_encrypt_empty_password_fails() {
    let dir = tempdir().unwrap();
    let pdf = write_sample_pdf(dir.path(), "x.pdf", 1);

    Command::cargo_bin("nextool")
        .unwrap()
        .args(["file-conv", "pdf", "encrypt"])
        .arg(&pdf)
        .args(["--password", ""])
        .assert()
        .failure();
}
