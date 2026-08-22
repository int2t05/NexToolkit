//! SVG 模块:SVG 栅格化为 PNG/JPG(字节域,纯内存)
//!
//! 经 `resvg` 解析 SVG → `tiny-skia` 栅格化 → 编码目标格式。PNG 经 tiny-skia 的 `encode_png`
//! (正确处理 premultiplied alpha 的 demultiply);JPG 经 SVG→PNG→`image` crate 重编码为 JPEG
//! (JPEG 无 alpha 通道,内部转 RGB)。输入输出为 `&[u8]`/`Vec<u8>`,不碰文件系统。
//! SVG→PDF(VC-03)需 printpdf,归另一批,暂跳过。

use crate::{ToolError, ToolResult};

/// SVG 栅格化目标格式
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr, strum::EnumString, strum::EnumIter,
)]
pub enum SvgFormat {
    #[strum(serialize = "png")]
    Png,
    #[strum(serialize = "jpg")]
    Jpg,
}

impl SvgFormat {
    /// 扩展名(无点)
    pub fn ext(&self) -> &'static str {
        match self {
            SvgFormat::Png => "png",
            SvgFormat::Jpg => "jpg",
        }
    }
}

/// SVG 栅格化为指定位图格式(PNG 或 JPG)
///
/// 使用 SVG 固有尺寸渲染;加载系统字体以支持 `<text>` 元素。
pub fn svg_render(data: &[u8], target: SvgFormat) -> ToolResult<Vec<u8>> {
    let pixmap = render_svg(data)?;
    match target {
        SvgFormat::Png => pixmap
            .encode_png()
            .map_err(|e| ToolError::Other(format!("PNG 编码失败: {e}"))),
        SvgFormat::Jpg => {
            // SVG→PNG→JPG:复用 tiny-skia 的 PNG 编码(正确 demultiply),再经 image crate 转 JPEG
            let png = pixmap
                .encode_png()
                .map_err(|e| ToolError::Other(format!("PNG 编码失败: {e}")))?;
            let img = image::load_from_memory(&png)
                .map_err(|e| ToolError::Other(format!("PNG 解码失败: {e}")))?;
            let mut buf = Vec::new();
            img.write_to(
                &mut std::io::Cursor::new(&mut buf),
                image::ImageFormat::Jpeg,
            )
            .map_err(|e| ToolError::Other(format!("JPEG 编码失败: {e}")))?;
            Ok(buf)
        }
    }
}

/// 解析 SVG 并渲染为 tiny-skia Pixmap
fn render_svg(data: &[u8]) -> ToolResult<resvg::tiny_skia::Pixmap> {
    let mut opt = resvg::usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    let tree = resvg::usvg::Tree::from_data(data, &opt)
        .map_err(|e| ToolError::Parse(format!("SVG 解析失败: {e}")))?;
    let size = tree.size();
    // 零尺寸 SVG(无 width/height 且无 viewBox)无法栅格化
    let int_size = size.to_int_size();
    let (w, h) = (int_size.width(), int_size.height());
    if w == 0 || h == 0 {
        return Err(ToolError::InvalidInput(
            "SVG 无有效尺寸(width/height/viewBox 缺失或为 0)".into(),
        ));
    }
    // 限制最大尺寸,防止恶意 SVG 导致超大分配
    const MAX_DIM: u32 = 16384;
    if w > MAX_DIM || h > MAX_DIM {
        return Err(ToolError::InvalidInput(format!(
            "SVG 尺寸 {w}x{h} 超过最大限制 {MAX_DIM}x{MAX_DIM}"
        )));
    }
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h)
        .ok_or_else(|| ToolError::Other("Pixmap 分配失败".into()))?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 最小合法 SVG:100×60 红色矩形 + 黑色描边
    const SAMPLE_SVG: &[u8] =
        b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"60\">\
<rect width=\"100\" height=\"60\" fill=\"#ff0000\" stroke=\"#000\" stroke-width=\"2\"/></svg>";

    #[test]
    fn render_png_produces_valid_png() {
        let png = svg_render(SAMPLE_SVG, SvgFormat::Png).unwrap();
        // PNG 魔术字节:89 50 4E 47 0D 0A 1A 0A
        assert_eq!(&png[..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        // 用 image crate 验证可解码且尺寸正确
        let img = image::load_from_memory(&png).unwrap();
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 60);
    }

    #[test]
    fn render_jpg_produces_valid_jpeg() {
        let jpg = svg_render(SAMPLE_SVG, SvgFormat::Jpg).unwrap();
        // JPEG 魔术字节:FF D8 FF
        assert_eq!(&jpg[..3], &[0xFF, 0xD8, 0xFF]);
        let img = image::load_from_memory(&jpg).unwrap();
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 60);
    }

    #[test]
    fn render_png_has_red_pixels() {
        // 红色矩形中心应近似红色(JPEG 有损,PNG 无损,故用 PNG 验证颜色)
        let png = svg_render(SAMPLE_SVG, SvgFormat::Png).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgba8();
        // 中心像素 (50, 30) 应为红色(忽略描边影响,取偏内位置 40,20)
        let pixel = img.get_pixel(40, 20);
        assert!(pixel[0] > 200, "R 分量应接近 255,实际 {}", pixel[0]);
        assert!(pixel[1] < 60, "G 分量应接近 0,实际 {}", pixel[1]);
        assert!(pixel[2] < 60, "B 分量应接近 0,实际 {}", pixel[2]);
    }

    #[test]
    fn render_invalid_svg_rejected() {
        // 非 SVG 文本应解析失败
        assert!(svg_render(b"not svg", SvgFormat::Png).is_err());
        // 损坏 SVG(未闭合)应解析失败
        assert!(svg_render(b"<svg><rect", SvgFormat::Png).is_err());
    }

    #[test]
    fn render_empty_svg_succeeds() {
        // 空 SVG(无内容)合法,usvg 给默认尺寸,应渲染成功而非报错
        let png = svg_render(b"<svg></svg>", SvgFormat::Png).unwrap();
        assert_eq!(&png[..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    #[test]
    fn render_empty_input_rejected() {
        assert!(svg_render(&[], SvgFormat::Png).is_err());
    }

    #[test]
    fn render_svg_with_viewbox() {
        // 无 width/height 但有 viewBox:resvg 应据 viewBox 推导尺寸
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 50 50\">\
<circle cx=\"25\" cy=\"25\" r=\"20\" fill=\"blue\"/></svg>";
        let png = svg_render(svg, SvgFormat::Png).unwrap();
        let img = image::load_from_memory(&png).unwrap();
        // viewBox 推导的尺寸(resvg 默认 100x100 或按 viewBox;验证 > 0 即可)
        assert!(img.width() > 0 && img.height() > 0);
    }
}
