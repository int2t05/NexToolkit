//! 图像模块:常用栅格格式互转与缩放(字节域,纯内存)
//!
//! 支持 PNG/JPEG/GIF/BMP/WebP/TIFF/ICO。输入输出为 `&[u8]`/`Vec<u8>`,不碰文件系统。
//! 格式经 image crate 魔术字节检测;转换经 `load_from_memory` → `encode_to`。

use image::GenericImageView;
use nextool_core::{ToolError, ToolResult};
use std::io::Cursor;

/// 图像格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Bmp,
    Webp,
    Tiff,
    Ico,
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
        }
    }
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

/// 编码 DynamicImage 到目标格式字节
fn encode(img: &image::DynamicImage, target: ImageFormat) -> ToolResult<Vec<u8>> {
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
}
