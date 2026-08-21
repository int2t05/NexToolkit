//! nextool-fileconv:NexToolkit 文件转换库
//!
//! 字节域转换(归档/图像/PDF)。[`archive`] 模块为纯内存逻辑(`&[u8]`/`Vec<u8>`,
//! 不碰文件系统,可独立单测);[`fs_util`] 模块为 IO 边界,组合纯逻辑 + `std::fs`
//! 落盘(产物落源目录 + 碰撞处理),供 CLI/GUI 共享。复用 [`nextool_core::ToolError`]
//! 统一错误。各模块只写自己文件,lib.rs 通过 glob 重导出。

#[cfg(feature = "archive")]
pub mod archive;
#[cfg(feature = "archive")]
pub mod fs_util;

#[cfg(feature = "archive")]
pub use archive::*;
#[cfg(feature = "archive")]
pub use fs_util::*;
