//! 字体模块:TTF/OTF 与 WOFF 互转 + 元数据查看(字节域,纯内存)
//!
//! TTF 与 OTF 同为 sfnt 容器(差异在 `glyf` vs `CFF ` 轮廓表,容器层一致),
//! 故 TTF↔WOFF 与 OTF↔WOFF 经同一包装逻辑处理。WOFF 是 sfnt 的 zlib 压缩包装:
//! 44 字节头 + 每表 20 字节目录 + 压缩表数据。元数据经 `ttf-parser` 读 name 表。
//! WOFF2 需 Brotli,暂不支持。输入输出为 `&[u8]`/`Vec<u8>`,不碰文件系统。

use crate::{ToolError, ToolResult};
use std::io::{Read, Write};

/// 字体格式:sfnt(TTF/OTF 容器)或 WOFF
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr, strum::EnumString, strum::EnumIter,
)]
pub enum FontFormat {
    #[strum(serialize = "ttf")]
    Ttf,
    #[strum(serialize = "woff")]
    Woff,
}

impl FontFormat {
    /// 扩展名(无点)
    pub fn ext(&self) -> &'static str {
        match self {
            FontFormat::Ttf => "ttf",
            FontFormat::Woff => "woff",
        }
    }
}

/// 字体元数据:name 表关键字段 + 字重/斜体/UPM
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontMeta {
    pub family: Option<String>,
    pub subfamily: Option<String>,
    pub full_name: Option<String>,
    pub post_script_name: Option<String>,
    pub version: Option<String>,
    pub copyright: Option<String>,
    pub weight: Option<u16>,
    pub italic: bool,
    pub units_per_em: Option<u16>,
    pub num_glyphs: u16,
}

/// 检测字体格式(魔术字节路由)
///
/// sfnt: `00 01 00 00`(TrueType)/ `4F 54 54 4F`("OTTO" CFF)/ `74 72 75 65`("true");
/// WOFF: `77 4F 46 46`("wOFF");WOFF2: `77 4F 46 32`("wOF2",不支持)。
pub fn detect_font_format(data: &[u8]) -> ToolResult<FontFormat> {
    if data.len() < 4 {
        return Err(ToolError::InvalidInput("数据过短,无法识别字体格式".into()));
    }
    let magic = &data[0..4];
    if magic == [0x00, 0x01, 0x00, 0x00]
        || magic == *b"OTTO"
        || magic == *b"true"
        || magic == *b"typ1"
    {
        return Ok(FontFormat::Ttf);
    }
    if magic == *b"wOFF" {
        return Ok(FontFormat::Woff);
    }
    if magic == *b"wOF2" {
        return Err(ToolError::InvalidInput(
            "WOFF2 暂不支持(需 Brotli 解码,仅做 TTF↔WOFF)".into(),
        ));
    }
    Err(ToolError::InvalidInput(
        "无法识别字体格式:魔术字节不匹配".into(),
    ))
}

/// 读取字体元数据:name 表关键字段 + OS/2 字重 + head UPM + maxp 字形数
///
/// WOFF 输入自动先解包为 sfnt 再解析。
pub fn font_metadata(data: &[u8]) -> ToolResult<FontMeta> {
    let sfnt = match detect_font_format(data)? {
        FontFormat::Ttf => data.to_vec(),
        FontFormat::Woff => woff_to_ttf(data)?,
    };
    let face = ttf_parser::Face::parse(&sfnt, 0)
        .map_err(|e| ToolError::Parse(format!("字体解析失败: {e}")))?;

    let mut meta = FontMeta {
        family: None,
        subfamily: None,
        full_name: None,
        post_script_name: None,
        version: None,
        copyright: None,
        weight: Some(face.weight().to_number()),
        italic: face.is_italic(),
        units_per_em: Some(face.units_per_em()),
        num_glyphs: face.number_of_glyphs(),
    };
    // name 表:遍历记录,按 name_id 取值(优先 Windows 平台 en-US 记录)
    for name in face.names() {
        let is_windows_en =
            name.platform_id == ttf_parser::PlatformId::Windows && name.language_id == 0x0409;
        let Some(s) = decode_name(name.platform_id, name.name) else {
            continue;
        };
        let assign = |slot: &mut Option<String>| {
            if slot.is_none() || is_windows_en {
                *slot = Some(s.clone());
            }
        };
        match name.name_id {
            0 => assign(&mut meta.copyright),
            1 => assign(&mut meta.family),
            2 => assign(&mut meta.subfamily),
            4 => assign(&mut meta.full_name),
            5 => assign(&mut meta.version),
            6 => assign(&mut meta.post_script_name),
            _ => {}
        }
    }
    Ok(meta)
}

/// 解码 name 表字符串:Windows 平台 UTF-16BE,Mac/其他平台按 UTF-8 lossy
fn decode_name(platform: ttf_parser::PlatformId, data: &[u8]) -> Option<String> {
    if data.is_empty() {
        return None;
    }
    if platform == ttf_parser::PlatformId::Windows {
        let u16s: Vec<u16> = data
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16(&u16s).ok()
    } else {
        Some(String::from_utf8_lossy(data).into_owned())
    }
}

/// 格式化元数据为多行文本(每行 `字段: 值`),供 CLI 打印 / GUI 展示
pub fn font_meta_to_text(meta: &FontMeta) -> String {
    let opt = |s: &Option<String>| s.clone().unwrap_or_else(|| "无".into());
    let opt_u = |s: &Option<u16>| s.map_or("无".into(), |v| v.to_string());
    [
        format!("字族: {}", opt(&meta.family)),
        format!("子族: {}", opt(&meta.subfamily)),
        format!("全名: {}", opt(&meta.full_name)),
        format!("PostScript 名: {}", opt(&meta.post_script_name)),
        format!("版本: {}", opt(&meta.version)),
        format!("版权: {}", opt(&meta.copyright)),
        format!("字重: {}", opt_u(&meta.weight)),
        format!("斜体: {}", if meta.italic { "是" } else { "否" }),
        format!("UPM: {}", opt_u(&meta.units_per_em)),
        format!("字形数: {}", meta.num_glyphs),
    ]
    .join("\n")
}

/// 字体格式互转:sfnt↔WOFF
///
/// TTF→WOFF:解包 sfnt 表 → zlib 压缩每表 → 构造 WOFF 容器。
/// WOFF→TTF:解压表数据 → 重组 sfnt 容器。
pub fn font_convert(data: &[u8], target: FontFormat) -> ToolResult<Vec<u8>> {
    let source = detect_font_format(data)?;
    match (source, target) {
        (FontFormat::Ttf, FontFormat::Woff) => ttf_to_woff(data),
        (FontFormat::Woff, FontFormat::Ttf) => woff_to_ttf(data),
        (FontFormat::Ttf, FontFormat::Ttf) | (FontFormat::Woff, FontFormat::Woff) => Err(
            ToolError::InvalidInput("源格式与目标格式相同,无需转换".into()),
        ),
    }
}

// ---- 私有:sfnt 解析与 WOFF 包装 ----

/// sfnt 表目录条目
struct SfntTable {
    tag: [u8; 4],
    checksum: u32,
    offset: u32,
    length: u32,
}

/// WOFF 表目录条目
struct WoffTable {
    tag: [u8; 4],
    offset: u32,
    comp_length: u32,
    orig_length: u32,
    orig_checksum: u32,
}

/// 解析 sfnt 表目录(flavor + 表列表)
fn parse_sfnt(data: &[u8]) -> ToolResult<(u32, Vec<SfntTable>)> {
    if data.len() < 12 {
        return Err(ToolError::InvalidInput("sfnt 数据过短(< 12 字节)".into()));
    }
    let flavor = read_u32_be(data, 0);
    if !matches!(flavor, 0x00010000 | 0x4F54544F | 0x74727565 | 0x74797031) {
        return Err(ToolError::InvalidInput(format!(
            "非 sfnt 容器:flavor=0x{flavor:08X}"
        )));
    }
    let num_tables = read_u16_be(data, 4) as usize;
    if data.len() < 12 + num_tables * 16 {
        return Err(ToolError::InvalidInput("sfnt 表目录越界".into()));
    }
    let mut tables = Vec::with_capacity(num_tables);
    for i in 0..num_tables {
        let off = 12 + i * 16;
        tables.push(SfntTable {
            tag: data[off..off + 4].try_into().unwrap(),
            checksum: read_u32_be(data, off + 4),
            offset: read_u32_be(data, off + 8),
            length: read_u32_be(data, off + 12),
        });
    }
    Ok((flavor, tables))
}

/// 解析 WOFF 头 + 表目录(flavor + 表列表)
fn parse_woff(data: &[u8]) -> ToolResult<(u32, Vec<WoffTable>)> {
    if data.len() < 44 {
        return Err(ToolError::InvalidInput("WOFF 数据过短(< 44 字节头)".into()));
    }
    let signature = read_u32_be(data, 0);
    if signature != 0x774F4646 {
        return Err(ToolError::InvalidInput("非 WOFF 格式:签名不匹配".into()));
    }
    let flavor = read_u32_be(data, 4);
    let num_tables = read_u16_be(data, 12) as usize;
    if data.len() < 44 + num_tables * 20 {
        return Err(ToolError::InvalidInput("WOFF 表目录越界".into()));
    }
    let mut tables = Vec::with_capacity(num_tables);
    for i in 0..num_tables {
        let off = 44 + i * 20;
        tables.push(WoffTable {
            tag: data[off..off + 4].try_into().unwrap(),
            offset: read_u32_be(data, off + 4),
            comp_length: read_u32_be(data, off + 8),
            orig_length: read_u32_be(data, off + 12),
            orig_checksum: read_u32_be(data, off + 16),
        });
    }
    Ok((flavor, tables))
}

/// TTF/OTF(sfnt)→ WOFF:zlib 压缩每表 + 构造 WOFF 容器
///
/// 压缩后若不小于原长度则原样存储(`comp_length == orig_length`)。
fn ttf_to_woff(data: &[u8]) -> ToolResult<Vec<u8>> {
    let (flavor, tables) = parse_sfnt(data)?;
    let num_tables = tables.len();
    let header_size = 44;
    let dir_size = num_tables * 20;

    // 先构造表数据段 + 目录元信息(偏移在数据段确定后再填)
    let mut table_data = Vec::new();
    let mut dir_entries: Vec<([u8; 4], u32, u32, u32, u32)> = Vec::with_capacity(num_tables);
    for t in &tables {
        let raw_start = t.offset as usize;
        let raw_end = raw_start + t.length as usize;
        if raw_end > data.len() {
            return Err(ToolError::InvalidInput(format!(
                "sfnt 表 {:?} 数据越界",
                t.tag
            )));
        }
        let raw = &data[raw_start..raw_end];
        let data_offset = (header_size + dir_size + table_data.len()) as u32;
        let (stored, comp_length) = if raw.len() <= 4 {
            // 极短数据压缩无收益,原样存储
            table_data.extend_from_slice(raw);
            (raw.len(), raw.len() as u32)
        } else {
            let compressed = zlib_compress(raw)?;
            if compressed.len() < raw.len() {
                table_data.extend_from_slice(&compressed);
                (compressed.len(), compressed.len() as u32)
            } else {
                table_data.extend_from_slice(raw);
                (raw.len(), raw.len() as u32)
            }
        };
        // 未压缩时 comp_length 须等于 orig_length(规范约定)
        let comp_length = if stored == raw.len() {
            raw.len() as u32
        } else {
            comp_length
        };
        dir_entries.push((t.tag, data_offset, comp_length, t.length, t.checksum));
    }

    let total_length = (header_size + dir_size + table_data.len()) as u32;
    let total_sfnt_size = data.len() as u32;

    let mut woff = Vec::with_capacity(total_length as usize);
    // WOFF 头(44 字节)
    write_u32_be(&mut woff, 0x774F4646); // signature "wOFF"
    write_u32_be(&mut woff, flavor);
    write_u32_be(&mut woff, total_length);
    write_u16_be(&mut woff, num_tables as u16);
    write_u16_be(&mut woff, 0); // reserved
    write_u32_be(&mut woff, total_sfnt_size);
    write_u16_be(&mut woff, 1); // majorVersion
    write_u16_be(&mut woff, 0); // minorVersion
    write_u32_be(&mut woff, 0); // metaOffset
    write_u32_be(&mut woff, 0); // metaLength
    write_u32_be(&mut woff, 0); // metaOrigLength
    write_u32_be(&mut woff, 0); // privOffset
    write_u32_be(&mut woff, 0); // privLength
                                // 表目录(每条 20 字节)
    for (tag, offset, comp_length, orig_length, checksum) in &dir_entries {
        woff.extend_from_slice(tag);
        write_u32_be(&mut woff, *offset);
        write_u32_be(&mut woff, *comp_length);
        write_u32_be(&mut woff, *orig_length);
        write_u32_be(&mut woff, *checksum);
    }
    // 表数据
    woff.extend_from_slice(&table_data);
    Ok(woff)
}

/// WOFF → TTF/OTF(sfnt):解压表数据 + 重组 sfnt 容器(表 4 字节对齐)
fn woff_to_ttf(data: &[u8]) -> ToolResult<Vec<u8>> {
    let (flavor, tables) = parse_woff(data)?;
    let num_tables = tables.len();

    // 解压每表
    let mut decompressed: Vec<([u8; 4], Vec<u8>, u32)> = Vec::with_capacity(num_tables);
    for t in &tables {
        let start = t.offset as usize;
        let end = start + t.comp_length as usize;
        if end > data.len() {
            return Err(ToolError::InvalidInput(format!(
                "WOFF 表 {:?} 数据越界",
                t.tag
            )));
        }
        let raw = &data[start..end];
        let data_bytes = if t.comp_length == t.orig_length {
            raw.to_vec()
        } else {
            zlib_decompress(raw)?
        };
        if data_bytes.len() != t.orig_length as usize {
            return Err(ToolError::InvalidInput(format!(
                "WOFF 表 {:?} 解压长度不匹配(期望 {}, 实际 {})",
                t.tag,
                t.orig_length,
                data_bytes.len()
            )));
        }
        decompressed.push((t.tag, data_bytes, t.orig_checksum));
    }

    // 重组 sfnt:表按 tag 升序(规范要求)
    decompressed.sort_by_key(|a| a.0);

    let (search_range, entry_selector, range_shift) = sfnt_table_metrics(num_tables);
    let header_size = 12;
    let dir_size = num_tables * 16;

    // 计算每表数据偏移(4 字节对齐)
    let mut offsets = Vec::with_capacity(num_tables);
    let mut cursor = header_size + dir_size;
    for (_, d, _) in &decompressed {
        offsets.push(cursor as u32);
        cursor += d.len();
        cursor += (4 - d.len() % 4) % 4; // 对齐填充
    }

    let mut sfnt = Vec::with_capacity(cursor);
    // sfnt 偏移表(12 字节)
    write_u32_be(&mut sfnt, flavor);
    write_u16_be(&mut sfnt, num_tables as u16);
    write_u16_be(&mut sfnt, search_range);
    write_u16_be(&mut sfnt, entry_selector);
    write_u16_be(&mut sfnt, range_shift);
    // 表目录(每条 16 字节)
    for ((tag, d, checksum), offset) in decompressed.iter().zip(offsets.iter()) {
        sfnt.extend_from_slice(tag);
        write_u32_be(&mut sfnt, *checksum);
        write_u32_be(&mut sfnt, *offset);
        write_u32_be(&mut sfnt, d.len() as u32);
    }
    // 表数据(4 字节对齐填充 0)
    for (_, d, _) in &decompressed {
        sfnt.extend_from_slice(d);
        let pad = (4 - d.len() % 4) % 4;
        sfnt.extend(vec![0u8; pad]);
    }
    Ok(sfnt)
}

/// sfnt 偏移表搜索参数(numTables → searchRange/entrySelector/rangeShift)
fn sfnt_table_metrics(num_tables: usize) -> (u16, u16, u16) {
    if num_tables == 0 {
        return (0, 0, 0);
    }
    let entry_selector = (usize::BITS - num_tables.leading_zeros() - 1) as u16;
    let search_range = 16 * (1 << entry_selector);
    let range_shift = (num_tables * 16) as u16 - search_range;
    (search_range, entry_selector, range_shift)
}

/// zlib 压缩(2 字节头 + deflate + 4 字节 adler32,WOFF 规范要求)
fn zlib_compress(data: &[u8]) -> ToolResult<Vec<u8>> {
    use flate2::{write::ZlibEncoder, Compression};
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

/// zlib 解压
fn zlib_decompress(data: &[u8]) -> ToolResult<Vec<u8>> {
    use flate2::read::ZlibDecoder;
    let mut decoder = ZlibDecoder::new(data);
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)?;
    Ok(buf)
}

// ---- 大端序读写 ----

fn read_u32_be(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap())
}

fn read_u16_be(data: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(data[offset..offset + 2].try_into().unwrap())
}

fn write_u32_be(buf: &mut Vec<u8>, val: u32) {
    buf.extend_from_slice(&val.to_be_bytes());
}

fn write_u16_be(buf: &mut Vec<u8>, val: u16) {
    buf.extend_from_slice(&val.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造最小合法 sfnt(含 head + hhea + maxp + name 表),用于 roundtrip 与元数据测试
    ///
    /// head 含合法 magicNumber 与 unitsPerEm;hhea/maxp 满足 ttf-parser 必备表要求;
    /// name 含 Windows 平台英文记录。
    fn build_minimal_ttf() -> Vec<u8> {
        let head = build_head_table();
        let hhea = build_hhea_table();
        let maxp = build_maxp_table();
        let name = build_name_table("TestFont");
        let mut tables = vec![
            (*b"head", head),
            (*b"hhea", hhea),
            (*b"maxp", maxp),
            (*b"name", name),
        ];
        tables.sort_by_key(|a| a.0);

        let num_tables = tables.len();
        let (search_range, entry_selector, range_shift) = sfnt_table_metrics(num_tables);
        let header_size = 12;
        let dir_size = num_tables * 16;

        let mut offsets = Vec::with_capacity(num_tables);
        let mut cursor = header_size + dir_size;
        for (_, d) in &tables {
            offsets.push(cursor as u32);
            cursor += d.len();
            cursor += (4 - d.len() % 4) % 4;
        }

        let mut sfnt = Vec::with_capacity(cursor);
        write_u32_be(&mut sfnt, 0x00010000); // sfntVersion (TrueType)
        write_u16_be(&mut sfnt, num_tables as u16);
        write_u16_be(&mut sfnt, search_range);
        write_u16_be(&mut sfnt, entry_selector);
        write_u16_be(&mut sfnt, range_shift);
        for ((tag, d), offset) in tables.iter().zip(offsets.iter()) {
            sfnt.extend_from_slice(tag);
            write_u32_be(&mut sfnt, 0); // checksum(简化,测试不校验)
            write_u32_be(&mut sfnt, *offset);
            write_u32_be(&mut sfnt, d.len() as u32);
        }
        for (_, d) in &tables {
            sfnt.extend_from_slice(d);
            let pad = (4 - d.len() % 4) % 4;
            sfnt.extend(vec![0u8; pad]);
        }
        sfnt
    }

    /// 构造合法 head 表(54 字节):magicNumber=0x5F0F3CF5,unitsPerEm=1000
    fn build_head_table() -> Vec<u8> {
        let mut h = Vec::with_capacity(54);
        write_u16_be(&mut h, 1); // majorVersion
        write_u16_be(&mut h, 0); // minorVersion
        write_u32_be(&mut h, 0x00010000); // fontRevision (1.0)
        write_u32_be(&mut h, 0); // checksumAdjustment
        write_u32_be(&mut h, 0x5F0F3CF5); // magicNumber(必须)
        write_u16_be(&mut h, 0x000B); // flags
        write_u16_be(&mut h, 1000); // unitsPerEm
        h.extend_from_slice(&[0u8; 8]); // created (LONGDATETIME)
        h.extend_from_slice(&[0u8; 8]); // modified (LONGDATETIME)
        write_u16_be(&mut h, 0); // xMin (i16)
        write_u16_be(&mut h, 0); // yMin (i16)
        write_u16_be(&mut h, 0); // xMax (i16)
        write_u16_be(&mut h, 0); // yMax (i16)
        write_u16_be(&mut h, 0); // macStyle
        write_u16_be(&mut h, 8); // lowestRecPPEM
        write_u16_be(&mut h, 2); // fontDirectionHint (i16)
        write_u16_be(&mut h, 0); // indexToLocFormat (i16)
        write_u16_be(&mut h, 0); // glyphDataFormat (i16)
        h
    }

    /// 构造 maxp 表(6 字节 CFF 版本):version=0x00005000,numGlyphs=1
    fn build_maxp_table() -> Vec<u8> {
        let mut m = Vec::with_capacity(6);
        write_u32_be(&mut m, 0x00005000); // version (CFF maxp)
        write_u16_be(&mut m, 1); // numGlyphs
        m
    }

    /// 构造 hhea 表(36 字节):ascender/descender/numberOfHMetrics 等基本字段
    fn build_hhea_table() -> Vec<u8> {
        let mut h = Vec::with_capacity(36);
        write_u16_be(&mut h, 1); // majorVersion
        write_u16_be(&mut h, 0); // minorVersion
        write_u16_be(&mut h, 800); // ascender (i16)
        write_u16_be(&mut h, (-200i16) as u16); // descender (i16)
        write_u16_be(&mut h, 0); // lineGap (i16)
        write_u16_be(&mut h, 1000); // advanceWidthMax
        write_u16_be(&mut h, 0); // minLeftSideBearing (i16)
        write_u16_be(&mut h, 0); // minRightSideBearing (i16)
        write_u16_be(&mut h, 1000); // xMaxExtent (i16)
        write_u16_be(&mut h, 1); // caretSlopeRise (i16)
        write_u16_be(&mut h, 0); // caretSlopeRun (i16)
        write_u16_be(&mut h, 0); // caretOffset (i16)
        h.extend_from_slice(&[0u8; 8]); // reserved × 4 (i16)
        write_u16_be(&mut h, 0); // metricDataFormat (i16)
        write_u16_be(&mut h, 1); // numberOfHMetrics
        h
    }

    /// 构造 name 表:Windows 平台(platformID=3, encodingID=1, languageID=0x0409)英文记录
    fn build_name_table(family: &str) -> Vec<u8> {
        // 记录:(name_id, 字符串)
        let records: [(u16, String); 5] = [
            (1, family.into()),
            (2, "Regular".into()),
            (4, family.into()),
            (5, "Version 1.0".into()),
            (6, format!("{family}-Regular")),
        ];
        // 字符串数据(UTF-16BE)
        let mut string_data = Vec::new();
        let mut string_offsets: Vec<(u16, u16)> = Vec::new(); // (length, offset)
        for (_, s) in &records {
            let offset = string_data.len() as u16;
            let utf16: Vec<u8> = s.encode_utf16().flat_map(|c| c.to_be_bytes()).collect();
            string_data.extend_from_slice(&utf16);
            string_offsets.push((utf16.len() as u16, offset));
        }

        let count = records.len() as u16;
        let string_offset = 6 + records.len() * 12; // header(6) + records

        let mut name = Vec::new();
        write_u16_be(&mut name, 0); // format
        write_u16_be(&mut name, count);
        write_u16_be(&mut name, string_offset as u16);
        for ((name_id, _), (length, offset)) in records.iter().zip(string_offsets.iter()) {
            write_u16_be(&mut name, 3); // platformID (Windows)
            write_u16_be(&mut name, 1); // encodingID (Unicode BMP)
            write_u16_be(&mut name, 0x0409); // languageID (en-US)
            write_u16_be(&mut name, *name_id);
            write_u16_be(&mut name, *length);
            write_u16_be(&mut name, *offset);
        }
        name.extend_from_slice(&string_data);
        name
    }

    // ---- detect ----

    #[test]
    fn detect_truetype_magic() {
        let ttf = build_minimal_ttf();
        assert_eq!(detect_font_format(&ttf).unwrap(), FontFormat::Ttf);
    }

    #[test]
    fn detect_woff_magic() {
        let ttf = build_minimal_ttf();
        let woff = ttf_to_woff(&ttf).unwrap();
        assert_eq!(detect_font_format(&woff).unwrap(), FontFormat::Woff);
    }

    #[test]
    fn detect_invalid_rejected() {
        assert!(detect_font_format(b"not a font").is_err());
        assert!(detect_font_format(&[]).is_err());
        assert!(detect_font_format(&[0x00, 0x01]).is_err());
    }

    // ---- TTF ↔ WOFF roundtrip ----

    #[test]
    fn ttf_to_woff_roundtrip_preserves_tables() {
        let ttf = build_minimal_ttf();
        let (flavor_in, tables_in) = parse_sfnt(&ttf).unwrap();

        let woff = ttf_to_woff(&ttf).unwrap();
        assert_eq!(detect_font_format(&woff).unwrap(), FontFormat::Woff);

        let ttf_back = woff_to_ttf(&woff).unwrap();
        let (flavor_out, tables_out) = parse_sfnt(&ttf_back).unwrap();

        assert_eq!(flavor_in, flavor_out, "flavor 应保持");
        assert_eq!(tables_in.len(), tables_out.len(), "表数量应一致");
        for (a, b) in tables_in.iter().zip(tables_out.iter()) {
            assert_eq!(a.tag, b.tag, "表 tag 应一致");
            assert_eq!(a.length, b.length, "表 {:?} 长度应一致", a.tag);
            let data_a = &sfnt_table_data(&ttf, a);
            let data_b = &sfnt_table_data(&ttf_back, b);
            assert_eq!(data_a, data_b, "表 {:?} 数据应一致", a.tag);
        }
    }

    /// 取 sfnt 表数据切片
    fn sfnt_table_data(sfnt: &[u8], t: &SfntTable) -> Vec<u8> {
        sfnt[t.offset as usize..(t.offset + t.length) as usize].to_vec()
    }

    #[test]
    fn woff_header_total_length_matches() {
        let ttf = build_minimal_ttf();
        let woff = ttf_to_woff(&ttf).unwrap();
        let total_length = read_u32_be(&woff, 8);
        assert_eq!(
            total_length as usize,
            woff.len(),
            "WOFF total_length 应与实际字节一致"
        );
    }

    #[test]
    fn woff_to_ttf_invalid_signature_rejected() {
        let bad = vec![0u8; 44];
        // 签名不为 "wOFF"
        assert!(woff_to_ttf(&bad).is_err());
    }

    // ---- font_convert ----

    #[test]
    fn convert_ttf_to_woff_and_back() {
        let ttf = build_minimal_ttf();
        let woff = font_convert(&ttf, FontFormat::Woff).unwrap();
        let ttf_back = font_convert(&woff, FontFormat::Ttf).unwrap();
        // 表数据一致(见 roundtrip 测试)
        let (t_in, t_out) = (
            parse_sfnt(&ttf).unwrap().1,
            parse_sfnt(&ttf_back).unwrap().1,
        );
        assert_eq!(t_in.len(), t_out.len());
    }

    #[test]
    fn convert_same_format_rejected() {
        let ttf = build_minimal_ttf();
        assert!(font_convert(&ttf, FontFormat::Ttf).is_err());
    }

    // ---- font_metadata ----

    #[test]
    fn metadata_extracts_family_name() {
        let ttf = build_minimal_ttf();
        let meta = font_metadata(&ttf).unwrap();
        assert_eq!(meta.family.as_deref(), Some("TestFont"));
        assert_eq!(meta.subfamily.as_deref(), Some("Regular"));
        assert_eq!(meta.full_name.as_deref(), Some("TestFont"));
        assert_eq!(meta.post_script_name.as_deref(), Some("TestFont-Regular"));
        assert_eq!(meta.version.as_deref(), Some("Version 1.0"));
    }

    #[test]
    fn metadata_from_woff_input() {
        let ttf = build_minimal_ttf();
        let woff = ttf_to_woff(&ttf).unwrap();
        let meta = font_metadata(&woff).unwrap();
        assert_eq!(meta.family.as_deref(), Some("TestFont"));
    }

    #[test]
    fn metadata_invalid_input_rejected() {
        assert!(font_metadata(b"not a font").is_err());
    }

    #[test]
    fn meta_to_text_contains_fields() {
        let meta = FontMeta {
            family: Some("TestFont".into()),
            subfamily: Some("Regular".into()),
            full_name: Some("TestFont".into()),
            post_script_name: Some("TestFont-Regular".into()),
            version: Some("Version 1.0".into()),
            copyright: None,
            weight: Some(400),
            italic: false,
            units_per_em: Some(1000),
            num_glyphs: 10,
        };
        let text = font_meta_to_text(&meta);
        assert!(text.contains("字族: TestFont"));
        assert!(text.contains("字重: 400"));
        assert!(text.contains("UPM: 1000"));
        assert!(text.contains("版权: 无"));
    }

    // ---- sfnt_table_metrics ----

    #[test]
    fn table_metrics_two_tables() {
        let (sr, es, rs) = sfnt_table_metrics(2);
        // entry_selector = floor(log2(2)) = 1
        assert_eq!(es, 1);
        // search_range = 16 * 2^1 = 32
        assert_eq!(sr, 32);
        // range_shift = 2*16 - 32 = 0
        assert_eq!(rs, 0);
    }

    #[test]
    fn table_metrics_eight_tables() {
        let (sr, es, rs) = sfnt_table_metrics(8);
        assert_eq!(es, 3);
        assert_eq!(sr, 128); // 16 * 2^3
        assert_eq!(rs, 0); // 8*16 - 128 = 0
    }
}
