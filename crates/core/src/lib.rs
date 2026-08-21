//! nextool-core:NexToolkit 核心逻辑库
//!
//! 纯 Rust 实现,不依赖 Tauri/clap/stdin/stdout,可被 CLI 与 GUI 共享调用。
//! 工具按域分模块(encode/convert/format/generate/text/crypto/nettime),
//! 每个模块暴露类型化函数,输入输出为普通 Rust 类型,错误统一为 [`ToolError`]。

pub mod encode;

pub use encode::{base64_decode, base64_encode};

/// 核心错误类型:统一错误来源,供 CLI/GUI 转换为用户可读信息
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("UTF-8 转换失败: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Base64 错误: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("输入为空")]
    EmptyInput,

    #[error("{0}")]
    Other(String),
}

pub type ToolResult<T> = Result<T, ToolError>;
