//! 图像模块:栅格格式互转与缩放(字节域,纯内存)
//!
//! 支持 PNG/JPEG/GIF/BMP/WebP/TIFF/ICO + DDS/Farbfeld/HDR/OpenEXR/PNM/QOI/TGA。
//! DDS 仅解码(image crate 无编码器),只可作转换源。输入输出为 `&[u8]`/`Vec<u8>`,
//! 不碰文件系统。格式经 image crate 魔术字节检测;转换经 `load_from_memory` → `encode_to`。

use crate::{ToolError, ToolResult};
use image::{GenericImageView, ImageEncoder};
use std::io::Cursor;

/// 图像格式
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr, strum::EnumString, strum::EnumIter,
)]
pub enum ImageFormat {
    #[strum(serialize = "png")]
    Png,
    #[strum(serialize = "jpg")]
    Jpeg,
    #[strum(serialize = "gif")]
    Gif,
    #[strum(serialize = "bmp")]
    Bmp,
    #[strum(serialize = "webp")]
    Webp,
    #[strum(serialize = "tiff")]
    Tiff,
    #[strum(serialize = "ico")]
    Ico,
    #[strum(serialize = "dds")]
    Dds,
    #[strum(serialize = "farbfeld")]
    Farbfeld,
    #[strum(serialize = "hdr")]
    Hdr,
    #[strum(serialize = "exr")]
    OpenExr,
    #[strum(serialize = "pnm")]
    Pnm,
    #[strum(serialize = "qoi")]
    Qoi,
    #[strum(serialize = "tga")]
    Tga,
}

impl ImageFormat {
    /// 扩展名(无点)
    pub fn ext(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Webp => "webp",
            ImageFormat::Tiff => "tiff",
            ImageFormat::Ico => "ico",
            ImageFormat::Dds => "dds",
            ImageFormat::Farbfeld => "ff",
            ImageFormat::Hdr => "hdr",
            ImageFormat::OpenExr => "exr",
            ImageFormat::Pnm => "pnm",
            ImageFormat::Qoi => "qoi",
            ImageFormat::Tga => "tga",
        }
    }

    /// 转为 image crate 的 ImageFormat
    fn to_image_format(self) -> image::ImageFormat {
        match self {
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::Jpeg => image::ImageFormat::Jpeg,
            ImageFormat::Gif => image::ImageFormat::Gif,
            ImageFormat::Bmp => image::ImageFormat::Bmp,
            ImageFormat::Webp => image::ImageFormat::WebP,
            ImageFormat::Tiff => image::ImageFormat::Tiff,
            ImageFormat::Ico => image::ImageFormat::Ico,
            ImageFormat::Dds => image::ImageFormat::Dds,
            ImageFormat::Farbfeld => image::ImageFormat::Farbfeld,
            ImageFormat::Hdr => image::ImageFormat::Hdr,
            ImageFormat::OpenExr => image::ImageFormat::OpenExr,
            ImageFormat::Pnm => image::ImageFormat::Pnm,
            ImageFormat::Qoi => image::ImageFormat::Qoi,
            ImageFormat::Tga => image::ImageFormat::Tga,
        }
    }

    /// 是否支持编码(DDS 在 image crate 仅解码,不可作转换目标)
    fn can_encode(self) -> bool {
        !matches!(self, ImageFormat::Dds)
    }
}

/// 翻转方向:水平(H,左右镜像)/垂直(V,上下镜像)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr, strum::EnumString, strum::EnumIter,
)]
pub enum FlipDirection {
    #[strum(serialize = "h")]
    Horizontal,
    #[strum(serialize = "v")]
    Vertical,
}

/// 滤镜种类:灰度/反相/棕褐/模糊
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr, strum::EnumString, strum::EnumIter,
)]
pub enum FilterKind {
    #[strum(serialize = "grayscale")]
    Grayscale,
    #[strum(serialize = "invert")]
    Invert,
    #[strum(serialize = "sepia")]
    Sepia,
    #[strum(serialize = "blur")]
    Blur,
}

/// 检测图像格式(基于 image crate 的魔术字节识别)
pub fn detect_image_format(data: &[u8]) -> ToolResult<ImageFormat> {
    let fmt = image::guess_format(data)
        .map_err(|e| ToolError::InvalidInput(format!("无法识别图像格式: {e}")))?;
    match fmt {
        image::ImageFormat::Png => Ok(ImageFormat::Png),
        image::ImageFormat::Jpeg => Ok(ImageFormat::Jpeg),
        image::ImageFormat::Gif => Ok(ImageFormat::Gif),
        image::ImageFormat::Bmp => Ok(ImageFormat::Bmp),
        image::ImageFormat::WebP => Ok(ImageFormat::Webp),
        image::ImageFormat::Tiff => Ok(ImageFormat::Tiff),
        image::ImageFormat::Ico => Ok(ImageFormat::Ico),
        image::ImageFormat::Dds => Ok(ImageFormat::Dds),
        image::ImageFormat::Farbfeld => Ok(ImageFormat::Farbfeld),
        image::ImageFormat::Hdr => Ok(ImageFormat::Hdr),
        image::ImageFormat::OpenExr => Ok(ImageFormat::OpenExr),
        image::ImageFormat::Pnm => Ok(ImageFormat::Pnm),
        image::ImageFormat::Qoi => Ok(ImageFormat::Qoi),
        image::ImageFormat::Tga => Ok(ImageFormat::Tga),
        other => Err(ToolError::InvalidInput(format!(
            "不支持的图像格式: {other:?}"
        ))),
    }
}

/// 图像格式互转:解码输入字节 → 按目标格式重新编码
pub fn image_convert(data: &[u8], target: ImageFormat) -> ToolResult<Vec<u8>> {
    let img = image::load_from_memory(data)
        .map_err(|e| ToolError::InvalidInput(format!("图像解码失败: {e}")))?;
    encode(&img, target)
}

/// 图像缩放:解码 → 按 `width`/`height` 重采样(保持宽高比时另一维传 0,按非零维等比缩放)
///
/// `width` 与 `height` 均为 0 时报错;均为非零时强制拉伸。
pub fn image_resize(
    data: &[u8],
    width: u32,
    height: u32,
    target: ImageFormat,
) -> ToolResult<Vec<u8>> {
    if width == 0 && height == 0 {
        return Err(ToolError::InvalidInput("缩放宽高不能同时为 0".into()));
    }
    let img = image::load_from_memory(data)
        .map_err(|e| ToolError::InvalidInput(format!("图像解码失败: {e}")))?;
    let (w, h) = scaled_size(img.dimensions(), width, height);
    let resized = img.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
    encode(&resized, target)
}

/// 图像裁剪:取 `(x, y)` 起 `width`×`height` 子区域,越界报错
pub fn image_crop(
    data: &[u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    target: ImageFormat,
) -> ToolResult<Vec<u8>> {
    if width == 0 || height == 0 {
        return Err(ToolError::InvalidInput("裁剪宽高不能为 0".into()));
    }
    let img = image::load_from_memory(data)
        .map_err(|e| ToolError::InvalidInput(format!("图像解码失败: {e}")))?;
    let (iw, ih) = img.dimensions();
    let right = x
        .checked_add(width)
        .ok_or_else(|| ToolError::InvalidInput("裁剪坐标溢出".into()))?;
    let bottom = y
        .checked_add(height)
        .ok_or_else(|| ToolError::InvalidInput("裁剪坐标溢出".into()))?;
    if right > iw || bottom > ih {
        return Err(ToolError::InvalidInput(format!(
            "裁剪区域({x},{y},{width}x{height})超出图像尺寸({iw}x{ih})"
        )));
    }
    let cropped = img.crop_imm(x, y, width, height);
    encode(&cropped, target)
}

/// 图像翻转:水平(左右镜像)/垂直(上下镜像),原地操作后按目标格式编码
pub fn image_flip(
    data: &[u8],
    direction: FlipDirection,
    target: ImageFormat,
) -> ToolResult<Vec<u8>> {
    let mut img = image::load_from_memory(data)
        .map_err(|e| ToolError::InvalidInput(format!("图像解码失败: {e}")))?;
    match direction {
        FlipDirection::Horizontal => img = img.fliph(),
        FlipDirection::Vertical => img = img.flipv(),
    }
    encode(&img, target)
}

/// 图像滤镜:灰度/反相/棕褐/模糊(模糊用固定 sigma=2.0)
pub fn image_filter(data: &[u8], filter: FilterKind, target: ImageFormat) -> ToolResult<Vec<u8>> {
    let mut img = image::load_from_memory(data)
        .map_err(|e| ToolError::InvalidInput(format!("图像解码失败: {e}")))?;
    match filter {
        FilterKind::Grayscale => img = img.grayscale(),
        FilterKind::Invert => img.invert(),
        FilterKind::Sepia => img = sepia(&img),
        FilterKind::Blur => img = img.blur(2.0),
    }
    encode(&img, target)
}

/// 亮度/对比度调整:`brightness` 正值增亮、负值变暗(i32 加到每通道);
/// `contrast` 1.0 为原值,>1 增强、<1 降低(以 128 为中心缩放)
pub fn image_adjust(
    data: &[u8],
    brightness: i32,
    contrast: f32,
    target: ImageFormat,
) -> ToolResult<Vec<u8>> {
    let img = image::load_from_memory(data)
        .map_err(|e| ToolError::InvalidInput(format!("图像解码失败: {e}")))?;
    let out = img.brighten(brightness).adjust_contrast(contrast);
    encode(&out, target)
}

/// JPEG 压缩:按 `quality`(1..=100)重新编码为 JPEG;quality 越低文件越小
pub fn image_compress_jpeg(data: &[u8], quality: u8) -> ToolResult<Vec<u8>> {
    if quality == 0 || quality > 100 {
        return Err(ToolError::InvalidInput("JPEG 质量须在 1..=100".into()));
    }
    let img = image::load_from_memory(data)
        .map_err(|e| ToolError::InvalidInput(format!("图像解码失败: {e}")))?;
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let mut buf = Cursor::new(Vec::new());
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
    encoder
        .write_image(rgb.as_raw(), w, h, image::ExtendedColorType::Rgb8)
        .map_err(|e| ToolError::Other(format!("JPEG 编码失败: {e}")))?;
    Ok(buf.into_inner())
}

/// 编码 DynamicImage 到目标格式字节(DDS 不可编码,内部经 `can_encode` 校验返回 InvalidInput)
fn encode(img: &image::DynamicImage, target: ImageFormat) -> ToolResult<Vec<u8>> {
    if !target.can_encode() {
        return Err(ToolError::InvalidInput(format!(
            "{} 格式不支持编码(仅可作转换源)",
            target.ext()
        )));
    }
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, target.to_image_format())
        .map_err(|e| ToolError::Other(format!("图像编码失败: {e}")))?;
    Ok(buf.into_inner())
}

/// 计算实际缩放尺寸:均非零→强制;一维为 0→按另一维等比
fn scaled_size(orig: (u32, u32), width: u32, height: u32) -> (u32, u32) {
    let (ow, oh) = orig;
    match (width, height) {
        (0, 0) => (ow, oh),
        (w, 0) => {
            let h = if ow == 0 {
                0
            } else {
                oh.checked_mul(w).map(|p| p / ow).unwrap_or(0)
            };
            (w, h.max(1))
        }
        (0, h) => {
            let w = if oh == 0 {
                0
            } else {
                ow.checked_mul(h).map(|p| p / oh).unwrap_or(0)
            };
            (w.max(1), h)
        }
        (w, h) => (w, h),
    }
}

/// 棕褐色调:对每个 RGB 像素应用经典 sepia 加权矩阵(输出 RGB8,丢弃 alpha)
fn sepia(img: &image::DynamicImage) -> image::DynamicImage {
    let mut rgb = img.to_rgb8();
    for p in rgb.pixels_mut() {
        let [r, g, b] = p.0;
        let nr = sepia_sum(r, g, b, 0.393, 0.769, 0.189);
        let ng = sepia_sum(r, g, b, 0.349, 0.686, 0.168);
        let nb = sepia_sum(r, g, b, 0.272, 0.534, 0.131);
        *p = image::Rgb([nr, ng, nb]);
    }
    image::DynamicImage::ImageRgb8(rgb)
}

/// sepia 单通道加权和(截断到 255)
fn sepia_sum(r: u8, g: u8, b: u8, wr: f32, wg: f32, wb: f32) -> u8 {
    (wr * r as f32 + wg * g as f32 + wb * b as f32).min(255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 造一个 2x2 红色 PNG 字节,供转换/缩放测试(程序化,无外部 fixture)
    fn red_png() -> Vec<u8> {
        let img = image::RgbImage::from_pixel(2, 2, image::Rgb([255, 0, 0]));
        let dyn_img = image::DynamicImage::ImageRgb8(img);
        let mut buf = Cursor::new(Vec::new());
        dyn_img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        buf.into_inner()
    }

    fn sample_bytes() -> Vec<u8> {
        let img = image::RgbImage::from_pixel(4, 3, image::Rgb([10, 20, 30]));
        let dyn_img = image::DynamicImage::ImageRgb8(img);
        let mut buf = Cursor::new(Vec::new());
        dyn_img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        buf.into_inner()
    }

    // ---- detect_image_format ----

    #[test]
    fn detect_png() {
        assert_eq!(detect_image_format(&red_png()).unwrap(), ImageFormat::Png);
    }

    #[test]
    fn detect_png_magic_bytes() {
        // PNG 魔术字节
        assert_eq!(
            detect_image_format(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).unwrap(),
            ImageFormat::Png
        );
    }

    #[test]
    fn detect_jpeg_magic() {
        assert_eq!(
            detect_image_format(&[0xFF, 0xD8, 0xFF, 0xE0]).unwrap(),
            ImageFormat::Jpeg
        );
    }

    #[test]
    fn detect_non_image_rejected() {
        assert!(detect_image_format(b"plain text").is_err());
        assert!(detect_image_format(&[]).is_err());
    }

    // ---- image_convert ----

    #[test]
    fn convert_png_to_jpeg() {
        let out = image_convert(&red_png(), ImageFormat::Jpeg).unwrap();
        assert_eq!(detect_image_format(&out).unwrap(), ImageFormat::Jpeg);
    }

    #[test]
    fn convert_png_to_bmp() {
        let out = image_convert(&red_png(), ImageFormat::Bmp).unwrap();
        assert_eq!(detect_image_format(&out).unwrap(), ImageFormat::Bmp);
    }

    #[test]
    fn convert_png_to_webp() {
        let out = image_convert(&red_png(), ImageFormat::Webp).unwrap();
        assert_eq!(detect_image_format(&out).unwrap(), ImageFormat::Webp);
    }

    #[test]
    fn convert_jpeg_to_png() {
        let jpeg = image_convert(&red_png(), ImageFormat::Jpeg).unwrap();
        let png = image_convert(&jpeg, ImageFormat::Png).unwrap();
        assert_eq!(detect_image_format(&png).unwrap(), ImageFormat::Png);
    }

    #[test]
    fn convert_invalid_input_rejected() {
        assert!(image_convert(b"not an image", ImageFormat::Png).is_err());
    }

    // ---- 扩展格式(DDS/Farbfeld/HDR/EXR/PNM/QOI/TGA)----

    #[test]
    fn convert_png_to_qoi() {
        let out = image_convert(&red_png(), ImageFormat::Qoi).unwrap();
        assert_eq!(detect_image_format(&out).unwrap(), ImageFormat::Qoi);
    }

    #[test]
    fn convert_png_to_tga() {
        // TGA 无固定魔术字节且 image 0.25 编解码不对称(默认编码产物 guess_format 不可靠);
        // 验证编码产出非空字节(转换链路通)
        let out = image_convert(&red_png(), ImageFormat::Tga).unwrap();
        assert!(!out.is_empty());
    }

    #[test]
    fn convert_png_to_pnm() {
        let out = image_convert(&red_png(), ImageFormat::Pnm).unwrap();
        assert_eq!(detect_image_format(&out).unwrap(), ImageFormat::Pnm);
    }

    #[test]
    fn convert_png_to_farbfeld() {
        // Farbfeld 编码器要求 16-bit/channel;用 Rgba<u16> 缓冲构造 PNG16 输入验证编码往返
        let img: image::ImageBuffer<image::Rgba<u16>, Vec<u16>> =
            image::ImageBuffer::from_pixel(2, 2, image::Rgba([65535, 0, 0, 65535]));
        let dyn_img = image::DynamicImage::from(img);
        let mut buf = Cursor::new(Vec::new());
        dyn_img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        let png16 = buf.into_inner();

        let out = image_convert(&png16, ImageFormat::Farbfeld).unwrap();
        assert_eq!(detect_image_format(&out).unwrap(), ImageFormat::Farbfeld);
    }

    #[test]
    fn convert_to_dds_rejected() {
        // DDS 仅解码,image crate 无编码器,作转换目标应报错
        assert!(image_convert(&red_png(), ImageFormat::Dds).is_err());
    }

    // ---- image_resize ----

    #[test]
    fn resize_to_smaller() {
        let out = image_resize(&sample_bytes(), 2, 2, ImageFormat::Png).unwrap();
        let img = image::load_from_memory(&out).unwrap();
        assert_eq!(img.dimensions(), (2, 2));
    }

    #[test]
    fn resize_keep_aspect_by_width() {
        // 4x3 → width=8,height=0 应等比到 8x6
        let out = image_resize(&sample_bytes(), 8, 0, ImageFormat::Png).unwrap();
        let img = image::load_from_memory(&out).unwrap();
        assert_eq!(img.dimensions(), (8, 6));
    }

    #[test]
    fn resize_keep_aspect_by_height() {
        // 4x3 → width=0,height=6 应等比到 8x6
        let out = image_resize(&sample_bytes(), 0, 6, ImageFormat::Png).unwrap();
        let img = image::load_from_memory(&out).unwrap();
        assert_eq!(img.dimensions(), (8, 6));
    }

    #[test]
    fn resize_zero_both_rejected() {
        assert!(image_resize(&sample_bytes(), 0, 0, ImageFormat::Png).is_err());
    }

    #[test]
    fn resize_invalid_input_rejected() {
        assert!(image_resize(b"not an image", 10, 10, ImageFormat::Png).is_err());
    }

    // ---- scaled_size(纯逻辑)----

    #[test]
    fn scaled_size_both_nonzero_forces() {
        assert_eq!(scaled_size((100, 50), 10, 20), (10, 20));
    }

    #[test]
    fn scaled_size_by_width_keeps_aspect() {
        assert_eq!(scaled_size((100, 50), 20, 0), (20, 10));
    }

    #[test]
    fn scaled_size_by_height_keeps_aspect() {
        assert_eq!(scaled_size((100, 50), 0, 10), (20, 10));
    }

    // ---- 测试 helper:非对称 2x2(左上红/右上蓝/左下绿/右下白)----

    fn checker_png() -> Vec<u8> {
        let mut img = image::RgbImage::new(2, 2);
        img.put_pixel(0, 0, image::Rgb([255, 0, 0]));
        img.put_pixel(1, 0, image::Rgb([0, 0, 255]));
        img.put_pixel(0, 1, image::Rgb([0, 255, 0]));
        img.put_pixel(1, 1, image::Rgb([255, 255, 255]));
        let mut buf = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut buf, image::ImageFormat::Png)
            .unwrap();
        buf.into_inner()
    }

    // ---- image_crop ----

    #[test]
    fn crop_returns_subregion() {
        // 4x3 → crop (1,0,2,2) 得 2x2
        let out = image_crop(&sample_bytes(), 1, 0, 2, 2, ImageFormat::Png).unwrap();
        let img = image::load_from_memory(&out).unwrap();
        assert_eq!(img.dimensions(), (2, 2));
    }

    #[test]
    fn crop_out_of_bounds_rejected() {
        // 4x3 → crop (2,0,3,1) 宽度越界
        assert!(image_crop(&sample_bytes(), 2, 0, 3, 1, ImageFormat::Png).is_err());
    }

    #[test]
    fn crop_zero_size_rejected() {
        assert!(image_crop(&sample_bytes(), 0, 0, 0, 1, ImageFormat::Png).is_err());
        assert!(image_crop(&sample_bytes(), 0, 0, 1, 0, ImageFormat::Png).is_err());
    }

    #[test]
    fn crop_invalid_input_rejected() {
        assert!(image_crop(b"not an image", 0, 0, 1, 1, ImageFormat::Png).is_err());
    }

    // ---- image_flip ----

    #[test]
    fn flip_horizontal_swaps_columns() {
        let out = image_flip(&checker_png(), FlipDirection::Horizontal, ImageFormat::Png).unwrap();
        let rgb = image::load_from_memory(&out).unwrap().to_rgb8();
        // 原左红(255,0,0)右蓝(0,0,255) → 翻转后左蓝右红
        assert_eq!(rgb.get_pixel(0, 0), &image::Rgb([0, 0, 255]));
        assert_eq!(rgb.get_pixel(1, 0), &image::Rgb([255, 0, 0]));
    }

    #[test]
    fn flip_vertical_swaps_rows() {
        let out = image_flip(&checker_png(), FlipDirection::Vertical, ImageFormat::Png).unwrap();
        let rgb = image::load_from_memory(&out).unwrap().to_rgb8();
        // 原上红下绿 → 翻转后上绿下红
        assert_eq!(rgb.get_pixel(0, 0), &image::Rgb([0, 255, 0]));
        assert_eq!(rgb.get_pixel(0, 1), &image::Rgb([255, 0, 0]));
    }

    #[test]
    fn flip_invalid_input_rejected() {
        assert!(image_flip(b"not an image", FlipDirection::Horizontal, ImageFormat::Png).is_err());
    }

    // ---- image_filter ----

    #[test]
    fn filter_grayscale_produces_gray() {
        let out = image_filter(&checker_png(), FilterKind::Grayscale, ImageFormat::Png).unwrap();
        let rgb = image::load_from_memory(&out).unwrap().to_rgb8();
        // 红色像素灰度后三通道应相等
        let p = rgb.get_pixel(0, 0);
        assert_eq!(p.0[0], p.0[1]);
        assert_eq!(p.0[1], p.0[2]);
    }

    #[test]
    fn filter_invert_negates() {
        let out = image_filter(&checker_png(), FilterKind::Invert, ImageFormat::Png).unwrap();
        let rgb = image::load_from_memory(&out).unwrap().to_rgb8();
        // 红(255,0,0)反相为(0,255,255)
        assert_eq!(rgb.get_pixel(0, 0).0, [0, 255, 255]);
    }

    #[test]
    fn filter_sepia_reddish() {
        let out = image_filter(&checker_png(), FilterKind::Sepia, ImageFormat::Png).unwrap();
        let rgb = image::load_from_memory(&out).unwrap().to_rgb8();
        // 白色 sepia 后蓝通道衰减(呈暖色),R 不小于 B
        let p = rgb.get_pixel(1, 1);
        assert!(p.0[2] < 255, "sepia 后蓝通道应衰减: {:?}", p.0);
        assert!(p.0[0] >= p.0[2], "R 应不小于 B: {:?}", p.0);
    }

    #[test]
    fn filter_blur_preserves_size() {
        let out = image_filter(&checker_png(), FilterKind::Blur, ImageFormat::Png).unwrap();
        let img = image::load_from_memory(&out).unwrap();
        assert_eq!(img.dimensions(), (2, 2));
    }

    #[test]
    fn filter_invalid_input_rejected() {
        assert!(image_filter(b"not an image", FilterKind::Grayscale, ImageFormat::Png).is_err());
    }

    // ---- image_adjust ----

    #[test]
    fn adjust_brightness_increases() {
        // sample_bytes 像素 [10,20,30],brightness=50 应增亮(每通道值变大)
        let out = image_adjust(&sample_bytes(), 50, 1.0, ImageFormat::Png).unwrap();
        let rgb = image::load_from_memory(&out).unwrap().to_rgb8();
        let p = rgb.get_pixel(0, 0);
        assert!(p.0[0] > 10, "亮度增加后 R 应大于 10: {:?}", p.0);
        assert!(p.0[1] > 20, "亮度增加后 G 应大于 20: {:?}", p.0);
        assert!(p.0[2] > 30, "亮度增加后 B 应大于 30: {:?}", p.0);
    }

    #[test]
    fn adjust_contrast_darkens() {
        // [10,20,30] contrast=2.0:暗值(<128)应变更暗
        let out = image_adjust(&sample_bytes(), 0, 2.0, ImageFormat::Png).unwrap();
        let rgb = image::load_from_memory(&out).unwrap().to_rgb8();
        let p = rgb.get_pixel(0, 0);
        assert!(p.0[0] < 10, "对比度 2.0 后 R 应小于原值 10: {:?}", p.0);
    }

    #[test]
    fn adjust_invalid_input_rejected() {
        assert!(image_adjust(b"not an image", 10, 1.0, ImageFormat::Png).is_err());
    }

    // ---- image_compress_jpeg ----

    #[test]
    fn compress_jpeg_produces_jpeg() {
        let out = image_compress_jpeg(&red_png(), 80).unwrap();
        assert_eq!(detect_image_format(&out).unwrap(), ImageFormat::Jpeg);
    }

    #[test]
    fn compress_jpeg_quality_zero_rejected() {
        assert!(image_compress_jpeg(&red_png(), 0).is_err());
    }

    #[test]
    fn compress_jpeg_quality_over_100_rejected() {
        assert!(image_compress_jpeg(&red_png(), 101).is_err());
    }

    #[test]
    fn compress_jpeg_invalid_input_rejected() {
        assert!(image_compress_jpeg(b"not an image", 80).is_err());
    }
}
