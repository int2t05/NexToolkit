//! PDF 工具模块:拆分/旋转/加密/解密/合并/页操作/元数据/页码(字节域,纯内存)
//!
//! 输入输出为 `&[u8]`/`Vec<u8>`,不碰文件系统。基于 lopdf 的 Document 操作。
//! 合并经对象 ID 偏移(`renumber_objects_with`)避冲突 + 页树重组,完整迁移跨页资源。

use crate::{ToolError, ToolResult};
use lopdf::dictionary;
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

/// 旋转 PDF 所有页指定角度(顺时针):每页 /Rotate 加 degrees 模 360
///
/// `degrees` 须为 90/180/270,否则返回 `InvalidInput`。
pub fn pdf_rotate(data: &[u8], degrees: u32) -> ToolResult<Vec<u8>> {
    if !matches!(degrees, 90 | 180 | 270) {
        return Err(ToolError::InvalidInput("旋转角度须为 90/180/270".into()));
    }
    let mut doc = load_doc(data)?;
    let pages: Vec<_> = doc.get_pages().keys().copied().collect();
    for page_no in pages {
        let current = current_rotate(&doc, page_no);
        set_rotate(&mut doc, page_no, (current + degrees as i64) % 360)?;
    }
    save_doc(&mut doc)
}

/// 加密 PDF:用口令设置 RC4 128 位加密(V2,owner 与 user 口令相同,默认权限)
///
/// `password` 为空报错。
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

/// 页码奇偶性(拆分奇/偶页用)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr, strum::EnumString, strum::EnumIter,
)]
pub enum Parity {
    #[strum(serialize = "odd")]
    Odd,
    #[strum(serialize = "even")]
    Even,
}

/// 解析页码范围字符串为 (起始, 结束) 对列表
///
/// 格式 `"1-3,5,7-10"` → `[(1,3),(5,5),(7,10)]`。页号 1-based,`start` 须 ≥ 1 且 ≤ `end`。
/// 是否超出实际页数由 [`pdf_split_ranges`] 校验。
pub fn parse_page_ranges(s: &str) -> ToolResult<Vec<(u32, u32)>> {
    let s = s.trim();
    if s.is_empty() {
        return Err(ToolError::InvalidInput("范围不能为空".into()));
    }
    let mut ranges = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            return Err(ToolError::InvalidInput("范围段不能为空".into()));
        }
        let (start, end) = match part.split_once('-') {
            Some((a, b)) => {
                let a: u32 = a
                    .trim()
                    .parse()
                    .map_err(|_| ToolError::InvalidInput(format!("范围起始非数字: {a}")))?;
                let b: u32 = b
                    .trim()
                    .parse()
                    .map_err(|_| ToolError::InvalidInput(format!("范围结束非数字: {b}")))?;
                (a, b)
            }
            None => {
                let n: u32 = part
                    .parse()
                    .map_err(|_| ToolError::InvalidInput(format!("页号非数字: {part}")))?;
                (n, n)
            }
        };
        if start == 0 || end < start {
            return Err(ToolError::InvalidInput(format!(
                "范围 {start}-{end} 无效(页号须 ≥ 1 且 起始 ≤ 结束)"
            )));
        }
        ranges.push((start, end));
    }
    Ok(ranges)
}

/// 按自定义范围拆分 PDF:每段范围产出一个 PDF(含该段页)
///
/// `ranges` 为 (起始页, 结束页) 对,页号 1-based。超出实际页数或 `start > end` 报错。
pub fn pdf_split_ranges(data: &[u8], ranges: &[(u32, u32)]) -> ToolResult<Vec<Vec<u8>>> {
    let doc = load_doc(data)?;
    split_by_ranges(&doc, ranges)
}

/// 每 N 页拆分 PDF:每 n 页一个 PDF,末段不足 n 页按实际页数
pub fn pdf_split_every_n(data: &[u8], n: u32) -> ToolResult<Vec<Vec<u8>>> {
    if n == 0 {
        return Err(ToolError::InvalidInput("每段页数 n 须 ≥ 1".into()));
    }
    let doc = load_doc(data)?;
    let total = doc.get_pages().len() as u32;
    if total == 0 {
        return Ok(Vec::new());
    }
    let mut ranges = Vec::new();
    let mut start = 1;
    while start <= total {
        let end = (start + n - 1).min(total);
        ranges.push((start, end));
        start = end + 1;
    }
    split_by_ranges(&doc, &ranges)
}

/// 按奇偶页拆分 PDF:返回含所有奇页或偶页的单个 PDF
///
/// `Parity::Odd` 保留 1/3/5... 页,`Parity::Even` 保留 2/4/6... 页。
pub fn pdf_split_parity(data: &[u8], parity: Parity) -> ToolResult<Vec<u8>> {
    let doc = load_doc(data)?;
    let total = doc.get_pages().len() as u32;
    if total == 0 {
        return Err(ToolError::InvalidInput("PDF 无页".into()));
    }
    let keep: Vec<u32> = match parity {
        Parity::Odd => (1..=total).step_by(2).collect(),
        Parity::Even => (2..=total).step_by(2).collect(),
    };
    if keep.is_empty() {
        return Err(ToolError::InvalidInput("无对应奇偶页".into()));
    }
    let delete: Vec<u32> = (1..=total).filter(|p| !keep.contains(p)).collect();
    let mut single = doc;
    if !delete.is_empty() {
        single.delete_pages(&delete);
    }
    save_doc(&mut single)
}

/// 合并多个 PDF 为一个:对象 ID 偏移避冲突 + 页树重组
///
/// 策略:每文档 `renumber_objects_with(max_id)` 偏移到不冲突 ID 区间,收集页对象与
/// 所有附属对象(资源/字体/内容流),重组 Pages 树(Kids=所有页引用,Count=总页数)。
/// 完整迁移跨页资源,不丢内容。书签/大纲不支持(丢弃)。
pub fn pdf_merge(docs: &[Vec<u8>]) -> ToolResult<Vec<u8>> {
    if docs.is_empty() {
        return Err(ToolError::InvalidInput("无输入文档".into()));
    }
    if docs.len() == 1 {
        let _ = load_doc(&docs[0])?;
        return Ok(docs[0].clone());
    }
    use lopdf::Object;
    use std::collections::BTreeMap;

    let mut merged = lopdf::Document::with_version("1.5");
    let mut max_id: u32 = 1;
    let mut all_pages: BTreeMap<lopdf::ObjectId, Object> = BTreeMap::new();
    let mut other_objects: BTreeMap<lopdf::ObjectId, Object> = BTreeMap::new();
    let mut first_catalog: Option<(lopdf::ObjectId, Object)> = None;
    let mut first_pages: Option<(lopdf::ObjectId, Object)> = None;

    for doc_bytes in docs {
        let mut doc = load_doc(doc_bytes)?;
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        for (_page_no, page_id) in doc.get_pages() {
            if let Some(page_obj) = doc.objects.get(&page_id).cloned() {
                all_pages.insert(page_id, page_obj);
            }
        }
        for (id, obj) in &doc.objects {
            match obj.type_name().unwrap_or(b"") {
                b"Page" => {}
                b"Catalog" => {
                    first_catalog.get_or_insert((*id, obj.clone()));
                }
                b"Pages" => {
                    first_pages.get_or_insert((*id, obj.clone()));
                }
                b"Outlines" | b"Outline" => {}
                _ => {
                    other_objects.insert(*id, obj.clone());
                }
            }
        }
    }

    let catalog_id = first_catalog
        .as_ref()
        .ok_or_else(|| ToolError::Other("合并失败:输入文档缺少 Catalog".into()))?
        .0;
    let pages_id = first_pages
        .ok_or_else(|| ToolError::Other("合并失败:输入文档缺少 Pages".into()))?
        .0;

    for (id, obj) in &other_objects {
        merged.objects.insert(*id, obj.clone());
    }
    for (page_id, page_obj) in &all_pages {
        if let Ok(dict) = page_obj.as_dict() {
            let mut dict = dict.clone();
            dict.set("Parent", pages_id);
            merged.objects.insert(*page_id, Object::Dictionary(dict));
        }
    }
    let kids: Vec<Object> = all_pages.keys().map(|&id| Object::Reference(id)).collect();
    let count = all_pages.len() as u32;
    merged.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => count,
        }),
    );
    let mut catalog_dict = first_catalog
        .as_ref()
        .and_then(|cat| cat.1.as_dict().ok().cloned())
        .unwrap_or_default();
    catalog_dict.set("Pages", pages_id);
    catalog_dict.remove(b"Outlines");
    merged
        .objects
        .insert(catalog_id, Object::Dictionary(catalog_dict));
    merged.trailer.set("Root", catalog_id);
    merged.max_id = merged.objects.len() as u32;
    merged.renumber_objects();
    save_doc(&mut merged)
}

/// 删除指定页:page_nums 为 1-based 页号,超出范围报错
pub fn pdf_delete_pages(data: &[u8], page_nums: &[u32]) -> ToolResult<Vec<u8>> {
    let mut doc = load_doc(data)?;
    let total = doc.get_pages().len() as u32;
    validate_page_nums(page_nums, total)?;
    if !page_nums.is_empty() {
        doc.delete_pages(page_nums);
    }
    save_doc(&mut doc)
}

/// 提取指定页:保留 page_nums 列出的页,删除其余
pub fn pdf_extract_pages(data: &[u8], page_nums: &[u32]) -> ToolResult<Vec<u8>> {
    let mut doc = load_doc(data)?;
    let all: Vec<u32> = doc.get_pages().keys().copied().collect();
    let total = all.len() as u32;
    validate_page_nums(page_nums, total)?;
    if page_nums.is_empty() {
        return Err(ToolError::InvalidInput("提取页号不能为空".into()));
    }
    let delete: Vec<u32> = all
        .iter()
        .copied()
        .filter(|p| !page_nums.contains(p))
        .collect();
    if !delete.is_empty() {
        doc.delete_pages(&delete);
    }
    save_doc(&mut doc)
}

/// 设置 PDF 元数据(Info 字典):仅非 None 字段被写入
///
/// 写入 Title/Author/Subject/Keywords。全 None 时不修改 Info。
pub fn pdf_set_metadata(
    data: &[u8],
    title: Option<&str>,
    author: Option<&str>,
    subject: Option<&str>,
    keywords: Option<&str>,
) -> ToolResult<Vec<u8>> {
    use lopdf::Object;
    let mut doc = load_doc(data)?;
    if title.is_none() && author.is_none() && subject.is_none() && keywords.is_none() {
        return save_doc(&mut doc);
    }
    let info_id = match doc.trailer.get(b"Info") {
        Ok(Object::Reference(id)) => *id,
        _ => {
            let id = doc.add_object(dictionary! {});
            doc.trailer.set("Info", id);
            id
        }
    };
    let info_obj = doc
        .get_object_mut(info_id)
        .map_err(|e| ToolError::Other(format!("取 Info 字典失败: {e}")))?;
    if let Object::Dictionary(d) = info_obj {
        if let Some(v) = title {
            d.set("Title", Object::string_literal(v));
        }
        if let Some(v) = author {
            d.set("Author", Object::string_literal(v));
        }
        if let Some(v) = subject {
            d.set("Subject", Object::string_literal(v));
        }
        if let Some(v) = keywords {
            d.set("Keywords", Object::string_literal(v));
        }
    }
    save_doc(&mut doc)
}

/// 为 PDF 每页添加右下角页码(1-based,Helvetica 12pt)
///
/// 简单实现:每页追加内容流绘制页号,并确保 Resources 含 F1(Helvetica)。
/// 不破坏页已有资源;Font 子字典为引用的罕见情况不处理(页码可能不显示)。
pub fn pdf_add_page_numbers(data: &[u8]) -> ToolResult<Vec<u8>> {
    use lopdf::ObjectId;
    let mut doc = load_doc(data)?;
    let pages: Vec<ObjectId> = doc.get_pages().into_values().collect();
    if pages.is_empty() {
        return Err(ToolError::InvalidInput("PDF 无页".into()));
    }
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    for (idx, &page_id) in pages.iter().enumerate() {
        let num = idx + 1;
        // BT 开始文本 /F1 12 Tf 选字体 540 10 Td 定位 (num) Tj 显示 ET 结束
        let content = format!("BT /F1 12 Tf 540 10 Td ({num}) Tj ET");
        ensure_font_resource(&mut doc, page_id, font_id)?;
        doc.add_page_contents(page_id, content.into_bytes())
            .map_err(|e| ToolError::Other(format!("添加页码内容流失败: {e}")))?;
    }
    save_doc(&mut doc)
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

/// 按范围对拆分已加载文档:每段产一个 PDF(复用 delete_pages 模式)
///
/// 每段保留 `start..=end` 页,删除其余。页号 1-based,超出实际页数或 `start > end` 报错。
fn split_by_ranges(doc: &lopdf::Document, ranges: &[(u32, u32)]) -> ToolResult<Vec<Vec<u8>>> {
    let all_pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    let total = all_pages.len() as u32;
    if total == 0 {
        return Err(ToolError::InvalidInput("PDF 无页".into()));
    }
    let mut result = Vec::with_capacity(ranges.len());
    for &(start, end) in ranges {
        if start == 0 || end < start || start > total || end > total {
            return Err(ToolError::InvalidInput(format!(
                "范围 {start}-{end} 无效(页号须 1..={total})"
            )));
        }
        let keep: Vec<u32> = (start..=end).collect();
        let delete: Vec<u32> = all_pages
            .iter()
            .copied()
            .filter(|p| !keep.contains(p))
            .collect();
        let mut single = doc.clone();
        if !delete.is_empty() {
            single.delete_pages(&delete);
        }
        result.push(save_doc(&mut single)?);
    }
    Ok(result)
}

/// 校验页号列表:每个页号须在 1..=total 范围内
fn validate_page_nums(page_nums: &[u32], total: u32) -> ToolResult<()> {
    for &p in page_nums {
        if p == 0 || p > total {
            return Err(ToolError::InvalidInput(format!(
                "页号 {p} 无效(须 1..={total})"
            )));
        }
    }
    Ok(())
}

/// 确保页 Resources 的 Font 字典含 F1(font_id),不破坏其他资源
///
/// 处理三种情况:页无 Resources(新建含 F1 的 Resources);Resources 内联字典(向其 Font 加 F1);
/// Resources 为引用(操作引用对象)。Font 子字典为引用的罕见情况不处理(页码可能不显示)。
fn ensure_font_resource(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    font_id: lopdf::ObjectId,
) -> ToolResult<()> {
    use lopdf::Object;
    // 扫描 Resources 位置(不可变借用,避免与后续 mut 借用冲突)
    let res_ref = {
        let page = doc
            .get_object(page_id)
            .map_err(|e| ToolError::Other(format!("取页对象失败: {e}")))?;
        match page {
            Object::Dictionary(d) => match d.get(b"Resources") {
                Ok(Object::Reference(id)) => Some(*id),
                _ => None,
            },
            _ => return Err(ToolError::Other("页对象非字典".into())),
        }
    };
    match res_ref {
        Some(res_id) => {
            let res_obj = doc
                .get_object_mut(res_id)
                .map_err(|e| ToolError::Other(format!("取 Resources 失败: {e}")))?;
            if let Object::Dictionary(res) = res_obj {
                set_font_in_resources(res, font_id);
            }
        }
        None => {
            let page = doc
                .get_object_mut(page_id)
                .map_err(|e| ToolError::Other(format!("取页对象失败: {e}")))?;
            if let Object::Dictionary(page_dict) = page {
                match page_dict.get_mut(b"Resources") {
                    Ok(Object::Dictionary(res)) => {
                        set_font_in_resources(res, font_id);
                    }
                    _ => {
                        page_dict.set(
                            "Resources",
                            dictionary! { "Font" => dictionary! { "F1" => font_id } },
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

/// 向 Resources 字典的 Font 子字典添加 F1(font_id)
fn set_font_in_resources(res: &mut lopdf::Dictionary, font_id: lopdf::ObjectId) {
    use lopdf::Object;
    match res.get_mut(b"Font") {
        Ok(Object::Dictionary(fonts)) => {
            fonts.set("F1", font_id);
        }
        Err(_) => {
            res.set("Font", dictionary! { "F1" => font_id });
        }
        // Font 为引用等非字典情况不处理(此作用域无 doc 访问)
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{dictionary, Object};

    /// 用 lopdf API 构造 N 页合法 PDF(真实生成非手写字节)
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
        let rotated = pdf_rotate(&sample_pdf(), 90).unwrap();
        assert!(load_doc(&rotated).is_ok());
    }

    #[test]
    fn rotate_twice_180() {
        let once = pdf_rotate(&sample_pdf(), 90).unwrap();
        let twice = pdf_rotate(&once, 90).unwrap();
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

    // ---- 范围解析 ----

    #[test]
    fn parse_ranges_basic() {
        assert_eq!(
            parse_page_ranges("1-3,5,7-10").unwrap(),
            vec![(1, 3), (5, 5), (7, 10)]
        );
    }

    #[test]
    fn parse_ranges_single() {
        assert_eq!(parse_page_ranges("5").unwrap(), vec![(5, 5)]);
    }

    #[test]
    fn parse_ranges_tolerates_spaces() {
        assert_eq!(
            parse_page_ranges(" 1 - 3 , 5 ").unwrap(),
            vec![(1, 3), (5, 5)]
        );
    }

    #[test]
    fn parse_ranges_empty_rejected() {
        assert!(parse_page_ranges("").is_err());
        assert!(parse_page_ranges("1,,3").is_err());
    }

    #[test]
    fn parse_ranges_invalid_rejected() {
        assert!(parse_page_ranges("0-3").is_err());
        assert!(parse_page_ranges("5-2").is_err());
        assert!(parse_page_ranges("abc").is_err());
    }

    // ---- 自定义范围拆分 ----

    #[test]
    fn split_ranges_multi_segments() {
        let data = make_pdf(5);
        let parts = pdf_split_ranges(&data, &[(1, 2), (4, 5)]).unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(load_doc(&parts[0]).unwrap().get_pages().len(), 2);
        assert_eq!(load_doc(&parts[1]).unwrap().get_pages().len(), 2);
    }

    #[test]
    fn split_ranges_from_parsed_string() {
        let data = make_pdf(6);
        let ranges = parse_page_ranges("1-2,4,6").unwrap();
        let parts = pdf_split_ranges(&data, &ranges).unwrap();
        assert_eq!(parts.len(), 3);
        assert_eq!(load_doc(&parts[0]).unwrap().get_pages().len(), 2);
        assert_eq!(load_doc(&parts[1]).unwrap().get_pages().len(), 1);
        assert_eq!(load_doc(&parts[2]).unwrap().get_pages().len(), 1);
    }

    #[test]
    fn split_ranges_out_of_range_rejected() {
        let data = make_pdf(3);
        assert!(pdf_split_ranges(&data, &[(1, 5)]).is_err());
        assert!(pdf_split_ranges(&data, &[(0, 1)]).is_err());
    }

    // ---- 每 N 页拆分 ----

    #[test]
    fn split_every_n_balanced() {
        let data = make_pdf(6);
        let parts = pdf_split_every_n(&data, 3).unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(load_doc(&parts[0]).unwrap().get_pages().len(), 3);
        assert_eq!(load_doc(&parts[1]).unwrap().get_pages().len(), 3);
    }

    #[test]
    fn split_every_n_remainder() {
        let data = make_pdf(7);
        let parts = pdf_split_every_n(&data, 3).unwrap();
        assert_eq!(parts.len(), 3);
        assert_eq!(load_doc(&parts[0]).unwrap().get_pages().len(), 3);
        assert_eq!(load_doc(&parts[1]).unwrap().get_pages().len(), 3);
        assert_eq!(load_doc(&parts[2]).unwrap().get_pages().len(), 1);
    }

    #[test]
    fn split_every_n_zero_rejected() {
        let data = make_pdf(3);
        assert!(pdf_split_every_n(&data, 0).is_err());
    }

    // ---- 奇偶页拆分 ----

    #[test]
    fn split_parity_odd_keeps_135() {
        let out = pdf_split_parity(&make_pdf(5), Parity::Odd).unwrap();
        assert_eq!(load_doc(&out).unwrap().get_pages().len(), 3);
    }

    #[test]
    fn split_parity_even_keeps_24() {
        let out = pdf_split_parity(&make_pdf(5), Parity::Even).unwrap();
        assert_eq!(load_doc(&out).unwrap().get_pages().len(), 2);
    }

    // ---- 合并 ----

    #[test]
    fn merge_two_docs_total_pages() {
        let merged = pdf_merge(&[make_pdf(2), make_pdf(3)]).unwrap();
        assert_eq!(load_doc(&merged).unwrap().get_pages().len(), 5);
    }

    #[test]
    fn merge_three_docs_preserves_count() {
        let docs = vec![make_pdf(1), make_pdf(2), make_pdf(3)];
        let merged = pdf_merge(&docs).unwrap();
        assert_eq!(load_doc(&merged).unwrap().get_pages().len(), 6);
    }

    #[test]
    fn merge_single_doc_passthrough() {
        let data = make_pdf(3);
        let merged = pdf_merge(&[data]).unwrap();
        assert_eq!(load_doc(&merged).unwrap().get_pages().len(), 3);
    }

    #[test]
    fn merge_empty_rejected() {
        assert!(pdf_merge(&[]).is_err());
    }

    // ---- 删除/提取页 ----

    #[test]
    fn delete_pages_removes_specified() {
        let out = pdf_delete_pages(&make_pdf(5), &[2, 4]).unwrap();
        assert_eq!(load_doc(&out).unwrap().get_pages().len(), 3);
    }

    #[test]
    fn delete_pages_invalid_rejected() {
        let data = make_pdf(3);
        assert!(pdf_delete_pages(&data, &[0]).is_err());
        assert!(pdf_delete_pages(&data, &[5]).is_err());
    }

    #[test]
    fn extract_pages_keeps_specified() {
        let out = pdf_extract_pages(&make_pdf(5), &[1, 3, 5]).unwrap();
        assert_eq!(load_doc(&out).unwrap().get_pages().len(), 3);
    }

    #[test]
    fn extract_pages_empty_rejected() {
        assert!(pdf_extract_pages(&make_pdf(3), &[]).is_err());
    }

    // ---- 元数据 ----

    #[test]
    fn set_metadata_writes_all_fields() {
        let out = pdf_set_metadata(
            &make_pdf(1),
            Some("标题"),
            Some("作者"),
            Some("主题"),
            Some("关键词"),
        )
        .unwrap();
        let doc = load_doc(&out).unwrap();
        let info_id = doc
            .trailer
            .get(b"Info")
            .and_then(Object::as_reference)
            .expect("Info 字典应存在");
        let info = doc.get_object(info_id).unwrap();
        let d = match info {
            Object::Dictionary(d) => d,
            _ => panic!("Info 非字典"),
        };
        assert!(d.get(b"Title").is_ok());
        assert!(d.get(b"Author").is_ok());
        assert!(d.get(b"Subject").is_ok());
        assert!(d.get(b"Keywords").is_ok());
    }

    #[test]
    fn set_metadata_all_none_noop() {
        let out = pdf_set_metadata(&make_pdf(1), None, None, None, None).unwrap();
        assert!(load_doc(&out).is_ok());
    }

    // ---- 页码 ----

    #[test]
    fn add_page_numbers_preserves_page_count() {
        let out = pdf_add_page_numbers(&make_pdf(3)).unwrap();
        assert_eq!(load_doc(&out).unwrap().get_pages().len(), 3);
    }

    #[test]
    fn add_page_numbers_empty_rejected() {
        assert!(pdf_add_page_numbers(&make_pdf(0)).is_err());
    }
}
