//! nextool-core:NexToolkit 核心逻辑库
//!
//! 纯 Rust 实现,不依赖 Tauri/clap/stdin/stdout,可被 CLI 与 GUI 共享调用。
//! 工具按域分模块,每个模块暴露类型化函数,输入输出为普通 Rust 类型,错误统一为 [`ToolError`]。
//! 各模块只写自己文件,不修改本文件;lib.rs 通过 glob 重导出全部公开函数。

pub mod convert;
pub mod crypto;
pub mod encode;
pub mod fileconv;
pub mod format;
pub mod generate;
pub mod http;
pub mod nettime;
pub mod registry;
pub mod text;
pub mod unit;

pub use convert::*;
pub use crypto::*;
pub use encode::*;
pub use fileconv::*;
pub use format::*;
pub use generate::*;
pub use http::*;
pub use nettime::*;
pub use registry::*;
pub use text::*;
pub use unit::*;

/// 核心错误类型:统一错误来源,供 CLI/GUI 转换为用户可读信息
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("UTF-8 转换失败: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Base64 错误: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML 错误: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML 错误: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("CSV 错误: {0}")]
    Csv(#[from] csv::Error),

    #[error("正则错误: {0}")]
    Regex(#[from] regex::Error),

    #[error("输入为空")]
    EmptyInput,

    #[error("无效输入: {0}")]
    InvalidInput(String),

    #[error("解析失败: {0}")]
    Parse(String),

    #[error("{0}")]
    Other(String),
}

pub type ToolResult<T> = Result<T, ToolError>;
