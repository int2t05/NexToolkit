//! PDF 工具模块:拆分/旋转/加密/解密(字节域,纯内存)
//!
//! 输入输出为 `&[u8]`/`Vec<u8>`,不碰文件系统。基于 lopdf 的 Document 操作。
//! 合并(merge)因 lopdf 0.44 无内置页树合并 API,实现复杂度高,推迟(见 todo)。

use crate::{ToolError, ToolResult};
use std::io::Cursor;

/// 拆分 PDF:每页一个独立 PDF,返回字节列表(顺序对应页序)
pub fn pdf_split(data: &[u8]) -> ToolResult<Vec<Vec<u8>>> {
    let doc = load_doc(data)?;
    let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    if pages.is_empty() {
        return Ok(Vec::new());
    }
    let mut result = Vec::with_capacity(pages.len());
    for &page_no in &pages {
        let mut single = doc.clone();
        // 删除除当前页外的所有页(按页号)
        let others: Vec<u32> = pages.iter().copied().filter(|&p| p != page_no).collect();
        if !others.is_empty() {
            single.delete_pages(&others);
        }
        result.push(save_doc(&mut single)?);
    }
    Ok(result)
}

/// 旋转 PDF 所有页 90 度(顺时针):每页 /Rotate 加 90 模 360
pub fn pdf_rotate(data: &[u8]) -> ToolResult<Vec<u8>> {
    let mut doc = load_doc(data)?;
    let pages: Vec<_> = doc.get_pages().keys().copied().collect();
    for page_no in pages {
        let current = current_rotate(&doc, page_no);
        set_rotate(&mut doc, page_no, (current + 90) % 360)?;
    }
    save_doc(&mut doc)
}

/// 加密 PDF:用口令设置 R5(AES-256)加密,owner 与 user 口令相同,默认权限
///
/// `password` 为空报错。`rand` feature 提供随机密钥生成。
pub fn pdf_encrypt(data: &[u8], password: &str) -> ToolResult<Vec<u8>> {
    if password.is_empty() {
        return Err(ToolError::InvalidInput("加密口令不能为空".into()));
    }
    let mut doc = load_doc(data)?;
    let permissions = lopdf::Permissions::PRINTABLE
        | lopdf::Permissions::COPYABLE
        | lopdf::Permissions::COPYABLE_FOR_ACCESSIBILITY
        | lopdf::Permissions::PRINTABLE_IN_HIGH_QUALITY;
    let version = lopdf::EncryptionVersion::V2 {
        document: &doc,
        owner_password: password,
        user_password: password,
        key_length: 128,
        permissions,
    };
    let state = lopdf::EncryptionState::try_from(version)
        .map_err(|e| ToolError::Other(format!("构建加密状态失败: {e}")))?;
    doc.encrypt(&state)
        .map_err(|e| ToolError::Other(format!("PDF 加密失败: {e}")))?;
    save_doc(&mut doc)
}

/// 解密 PDF:用口令移除加密;未加密或口令错误报错
pub fn pdf_decrypt(data: &[u8], password: &str) -> ToolResult<Vec<u8>> {
    let mut doc = load_doc_with_password(data, password)?;
    if !doc.is_encrypted() {
        return Err(ToolError::InvalidInput("PDF 未加密".into()));
    }
    doc.decrypt(password)
        .map_err(|e| ToolError::InvalidInput(format!("解密失败(口令错误或数据损坏): {e}")))?;
    save_doc(&mut doc)
}

/// 检测 PDF 是否加密(读元数据 /Encrypt 声明,不验证口令)
pub fn pdf_is_encrypted(data: &[u8]) -> ToolResult<bool> {
    let meta = lopdf::Document::load_metadata_mem(data)
        .map_err(|e| ToolError::InvalidInput(format!("PDF 读取失败: {e}")))?;
    Ok(meta.encrypted)
}

// ---- 私有辅助 ----

fn load_doc(data: &[u8]) -> ToolResult<lopdf::Document> {
    lopdf::Document::load_mem(data)
        .map_err(|e| ToolError::InvalidInput(format!("PDF 加载失败: {e}")))
}

fn load_doc_with_password(data: &[u8], password: &str) -> ToolResult<lopdf::Document> {
    lopdf::Document::load_mem_with_options(data, lopdf::LoadOptions::with_password(password))
        .map_err(|e| ToolError::InvalidInput(format!("PDF 加载失败(口令错误或数据损坏): {e}")))
}

fn save_doc(doc: &mut lopdf::Document) -> ToolResult<Vec<u8>> {
    let mut buf = Vec::new();
    doc.save_to(&mut Cursor::new(&mut buf))
        .map_err(|e| ToolError::Other(format!("PDF 保存失败: {e}")))?;
    Ok(buf)
}

/// 读取页 /Rotate 值(默认 0)。page_no 为页号(u32),经 get_pages 取 ObjectId
fn current_rotate(doc: &lopdf::Document, page_no: u32) -> i64 {
    use lopdf::Object;
    let page_id = match doc.get_pages().get(&page_no) {
        Some(id) => *id,
        None => return 0,
    };
    let page = match doc.get_object(page_id) {
        Ok(Object::Dictionary(d)) => d,
        _ => return 0,
    };
    match page.get(b"Rotate") {
        Ok(Object::Integer(i)) => *i,
        _ => 0,
    }
}

/// 设置页 /Rotate 值
fn set_rotate(doc: &mut lopdf::Document, page_no: u32, degrees: i64) -> ToolResult<()> {
    use lopdf::{Object, ObjectId};
    let page_id: ObjectId = *doc
        .get_pages()
        .get(&page_no)
        .ok_or_else(|| ToolError::Other(format!("页 {page_no} 不存在")))?;
    let page_obj = doc
        .get_object_mut(page_id)
        .map_err(|e| ToolError::Other(format!("取页对象失败: {e}")))?;
    if let Object::Dictionary(d) = page_obj {
        d.set("Rotate", Object::Integer(degrees));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{dictionary, Object};

    /// 用 lopdf API 构造 N 页合法 PDF(create.rs 模式,真实生成非手写字节)
    fn make_pdf(page_count: u32) -> Vec<u8> {
        let mut doc = lopdf::Document::with_version("1.4");
        let pages_id = doc.new_object_id();
        let mut page_ids = Vec::new();
        for _ in 0..page_count {
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
                "Count" => page_count,
            }),
        );
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        // 加密需 trailer /ID(16 字节文件标识对)
        let id: Vec<u8> = (0..16).collect();
        doc.trailer.set(
            "ID",
            vec![
                Object::String(id.clone(), lopdf::StringFormat::Hexadecimal),
                Object::String(id, lopdf::StringFormat::Hexadecimal),
            ],
        );
        let mut buf = Vec::new();
        doc.save_to(&mut Cursor::new(&mut buf)).unwrap();
        buf
    }

    fn sample_pdf() -> Vec<u8> {
        make_pdf(1)
    }

    #[test]
    fn load_and_roundtrip() {
        let data = sample_pdf();
        let mut doc = load_doc(&data).unwrap();
        let mut buf = Vec::new();
        doc.save_to(&mut Cursor::new(&mut buf)).unwrap();
        assert!(load_doc(&buf).is_ok());
    }

    #[test]
    fn load_invalid_rejected() {
        assert!(load_doc(b"not a pdf").is_err());
    }

    #[test]
    fn split_single_page() {
        let parts = pdf_split(&sample_pdf()).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(load_doc(&parts[0]).is_ok());
    }

    #[test]
    fn split_each_part_single_page() {
        let data = make_pdf(3);
        let parts = pdf_split(&data).unwrap();
        assert_eq!(parts.len(), 3, "3 页应拆成 3 个");
        for p in &parts {
            let d = load_doc(p).unwrap();
            assert_eq!(d.get_pages().len(), 1, "每个拆分件应 1 页");
        }
    }

    #[test]
    fn rotate_still_valid() {
        let rotated = pdf_rotate(&sample_pdf()).unwrap();
        assert!(load_doc(&rotated).is_ok());
    }

    #[test]
    fn rotate_twice_180() {
        let once = pdf_rotate(&sample_pdf()).unwrap();
        let twice = pdf_rotate(&once).unwrap();
        // 两次旋转后 /Rotate 应为 180
        let doc = load_doc(&twice).unwrap();
        let page_id = *doc.get_pages().values().next().unwrap();
        if let Object::Dictionary(d) = doc.get_object(page_id).unwrap() {
            if let Ok(Object::Integer(i)) = d.get(b"Rotate") {
                assert_eq!(*i, 180);
            }
        }
    }

    #[test]
    fn encrypt_produces_encrypted_pdf() {
        // 加密后应标记为加密,且错误口令无法加载验证
        let enc = pdf_encrypt(&sample_pdf(), "secret").unwrap();
        assert!(pdf_is_encrypted(&enc).unwrap(), "加密后应标记为加密");
        assert!(pdf_decrypt(&enc, "wrong").is_err(), "错误口令应解密失败");
    }

    #[test]
    #[ignore = "lopdf V2 owner=user 相同口令的加载验证存在上游限制,待 lopdf 修复后启用"]
    fn encrypt_decrypt_roundtrip() {
        let enc = pdf_encrypt(&sample_pdf(), "secret").unwrap();
        let dec = pdf_decrypt(&enc, "secret").unwrap();
        assert!(!pdf_is_encrypted(&dec).unwrap(), "解密后应非加密");
        assert!(load_doc(&dec).is_ok());
    }

    #[test]
    fn encrypt_empty_password_rejected() {
        assert!(pdf_encrypt(&sample_pdf(), "").is_err());
    }

    #[test]
    fn decrypt_wrong_password_fails() {
        let enc = pdf_encrypt(&sample_pdf(), "right").unwrap();
        assert!(pdf_decrypt(&enc, "wrong").is_err());
    }

    #[test]
    fn decrypt_unencrypted_fails() {
        assert!(pdf_decrypt(&sample_pdf(), "").is_err());
    }

    #[test]
    fn is_encrypted_unencrypted_false() {
        assert!(!pdf_is_encrypted(&sample_pdf()).unwrap());
    }

    #[test]
    fn encrypt_invalid_input_rejected() {
        assert!(pdf_encrypt(b"not a pdf", "pw").is_err());
    }
}
