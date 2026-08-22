//! 文件转换:nextool-core 的字节域工具子模块
//!
//! 归档/图像/PDF 字节域转换。[`archive`]、[`image`]、[`pdf`] 为纯内存逻辑(`&[u8]`/`Vec<u8>`,
//! 不碰文件系统,可独立单测);[`engine`] 为外部引擎子进程桥接;[`path`] 为纯字符串路径计算;
//! [`fs_util`] 为 IO 边界,组合纯逻辑 + `std::fs` 落盘(产物落源目录 + 碰撞处理),供 CLI/GUI 共享。
//! 复用 crate 根 [`crate::ToolError`] 统一错误。各模块只写自己文件,mod.rs 通过 glob 重导出。

pub mod engine;
pub mod path;

#[cfg(feature = "archive")]
pub mod archive;
#[cfg(any(feature = "archive", feature = "image", feature = "pdf"))]
pub mod fs_util;
#[cfg(feature = "image")]
pub mod image;
#[cfg(feature = "pdf")]
pub mod pdf;

pub use engine::*;
pub use path::*;

#[cfg(feature = "archive")]
pub use archive::*;
#[cfg(any(feature = "archive", feature = "image", feature = "pdf"))]
pub use fs_util::*;
#[cfg(feature = "image")]
pub use image::*;
#[cfg(feature = "pdf")]
pub use pdf::*;
