//! 引擎层:外部引擎子进程桥接(ffmpeg/LibreOffice/calibre/ghostscript/tesseract)
//!
//! 纯逻辑部分(引擎枚举、命令构造)可单测;实际子进程调用需引擎已安装,集成测试 `#[ignore]。
//! 核心包不捆绑重引擎,运行时探测系统已装,首次使用提示安装(见 todo/审计)。

use nextool_core::{ToolError, ToolResult};
use std::process::Command;

/// 外部引擎类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Engine {
    /// 音视频转码
    Ffmpeg,
    /// Office↔PDF(需探测系统已装,体积大)
    LibreOffice,
    /// 电子书转换
    Calibre,
    /// PDF 压缩优化/PS/EPS
    Ghostscript,
    /// OCR
    Tesseract,
}

impl Engine {
    /// 引擎可执行文件名(跨平台,Windows 自动加 .exe 由 PATH 解析)
    pub fn binary(&self) -> &'static str {
        match self {
            Engine::Ffmpeg => "ffmpeg",
            Engine::LibreOffice => "soffice",
            Engine::Calibre => "ebook-convert",
            Engine::Ghostscript => "gs",
            Engine::Tesseract => "tesseract",
        }
    }

    /// 引擎用途说明
    pub fn desc(&self) -> &'static str {
        match self {
            Engine::Ffmpeg => "音视频转码",
            Engine::LibreOffice => "Office↔PDF",
            Engine::Calibre => "电子书转换",
            Engine::Ghostscript => "PDF 优化/PS/EPS",
            Engine::Tesseract => "OCR 文字识别",
        }
    }

    /// 构造转换命令参数(返回完整参数列表,不含二进制名)
    ///
    /// 各引擎的典型转换命令:input→output 格式由扩展名推断。
    pub fn convert_args(&self, input: &str, output: &str) -> Vec<String> {
        match self {
            Engine::Ffmpeg => vec!["-i".into(), input.into(), "-y".into(), output.into()],
            Engine::LibreOffice => vec![
                "--headless".into(),
                "--convert-to".into(),
                std::path::Path::new(output)
                    .extension()
                    .map(|e| e.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "pdf".into()),
                "--outdir".into(),
                std::path::Path::new(output)
                    .parent()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|| ".".into()),
                input.into(),
            ],
            Engine::Calibre => vec![input.into(), output.into()],
            Engine::Ghostscript => vec![
                "-sDEVICE=pdfwrite".into(),
                "-dPDFSETTINGS=/ebook".into(),
                "-o".into(),
                output.into(),
                input.into(),
            ],
            Engine::Tesseract => vec![input.into(), output.into()],
        }
    }
}

/// 探测引擎是否已安装(检查 PATH 中可执行文件)
///
/// 返回 true 表示系统已装该引擎。无引擎时调用方应提示用户安装。
pub fn detect_engine(engine: Engine) -> bool {
    Command::new(engine.binary())
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

/// 调用引擎执行转换,返回产物路径
///
/// 引擎未安装返回 Err(提示安装);子进程失败返回 Err。产物落 output 指定路径。
pub fn engine_convert(engine: Engine, input: &str, output: &str) -> ToolResult<String> {
    if !detect_engine(engine) {
        return Err(ToolError::InvalidInput(format!(
            "未检测到 {}({}),请先安装",
            engine.binary(),
            engine.desc()
        )));
    }
    let args = engine.convert_args(input, output);
    let status = Command::new(engine.binary())
        .args(&args)
        .status()
        .map_err(|e| ToolError::Other(format!("启动 {} 失败: {e}", engine.binary())))?;
    if !status.success() {
        return Err(ToolError::Other(format!(
            "{} 执行失败(退出码 {:?})",
            engine.binary(),
            status.code()
        )));
    }
    Ok(output.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_names() {
        assert_eq!(Engine::Ffmpeg.binary(), "ffmpeg");
        assert_eq!(Engine::LibreOffice.binary(), "soffice");
        assert_eq!(Engine::Calibre.binary(), "ebook-convert");
        assert_eq!(Engine::Ghostscript.binary(), "gs");
        assert_eq!(Engine::Tesseract.binary(), "tesseract");
    }

    #[test]
    fn ffmpeg_convert_args() {
        let args = Engine::Ffmpeg.convert_args("in.mp4", "out.mp3");
        assert_eq!(args, vec!["-i", "in.mp4", "-y", "out.mp3"]);
    }

    #[test]
    fn calibre_convert_args() {
        let args = Engine::Calibre.convert_args("in.epub", "out.pdf");
        assert_eq!(args, vec!["in.epub", "out.pdf"]);
    }

    #[test]
    fn ghostscript_convert_args() {
        let args = Engine::Ghostscript.convert_args("in.pdf", "out.pdf");
        assert!(args.contains(&"-sDEVICE=pdfwrite".to_string()));
        assert!(args.contains(&"-o".to_string()));
    }

    #[test]
    fn libreoffice_convert_args_extract_format() {
        let args = Engine::LibreOffice.convert_args("in.docx", "/tmp/out.pdf");
        // --convert-to 应取 output 扩展名 pdf
        assert!(args.contains(&"pdf".to_string()));
        assert!(args.contains(&"--headless".to_string()));
    }

    #[test]
    fn engine_convert_missing_engine_fails() {
        // 用一个必然不存在的引擎二进制名测试:直接构造 detect 路径
        // detect_engine 对真实已装引擎返回 true,此处验证逻辑而非依赖环境
        // 若系统恰好装了某引擎,跳过断言(用 detect 先判)
        for e in [
            Engine::Ffmpeg,
            Engine::LibreOffice,
            Engine::Calibre,
            Engine::Ghostscript,
            Engine::Tesseract,
        ] {
            if !detect_engine(e) {
                assert!(
                    engine_convert(e, "in", "out").is_err(),
                    "{} 未安装时应报错",
                    e.binary()
                );
            }
        }
    }

    #[test]
    #[ignore = "需真实安装 ffmpeg: cargo test -p nextool-fileconv engine::tests::ffmpeg_real -- --ignored"]
    fn ffmpeg_real() {
        if !detect_engine(Engine::Ffmpeg) {
            return;
        }
        // 真实测试需音视频文件,此处仅验证 detect + 命令构造无 panic
        let _args = Engine::Ffmpeg.convert_args("in.mp4", "out.mp3");
    }
}
