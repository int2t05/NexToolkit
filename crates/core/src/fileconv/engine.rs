//! 引擎层:外部引擎子进程桥接(ffmpeg/LibreOffice/calibre/pandoc/ghostscript/tesseract)
//!
//! 纯逻辑部分(引擎枚举、命令构造)可单测;子进程调用经 [`EngineRunner`] port 抽象,
//! prod 用 [`SubprocessRunner`],测试用桩实现。核心包不捆绑重引擎,运行时探测系统已装,
//! 未装时返回错误提示安装。

use crate::{ToolError, ToolResult};
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
    /// 标记语言互转(MD/HTML/RST/AsciiDoc/Org/Tex/...)
    Pandoc,
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
            Engine::Pandoc => "pandoc",
            Engine::Ghostscript => {
                // Windows 二进制为 gswin64c(64 位 CLI),Unix 为 gs
                if cfg!(windows) {
                    "gswin64c"
                } else {
                    "gs"
                }
            }
            Engine::Tesseract => "tesseract",
        }
    }

    /// 引擎用途说明
    pub fn desc(&self) -> &'static str {
        match self {
            Engine::Ffmpeg => "音视频转码",
            Engine::LibreOffice => "Office↔PDF",
            Engine::Calibre => "电子书转换",
            Engine::Pandoc => "标记语言转换",
            Engine::Ghostscript => "PDF 优化/PS/EPS",
            Engine::Tesseract => "OCR 文字识别",
        }
    }

    /// 全部引擎(展示与探测遍历用)
    pub fn all() -> &'static [Engine] {
        &[
            Engine::Ffmpeg,
            Engine::LibreOffice,
            Engine::Calibre,
            Engine::Pandoc,
            Engine::Ghostscript,
            Engine::Tesseract,
        ]
    }

    /// 构造转换命令参数(返回完整参数列表,不含二进制名)
    ///
    /// 各引擎的典型转换命令:input→output 格式由扩展名推断。
    pub fn convert_args(&self, input: &str, output: &str) -> Vec<String> {
        match self {
            Engine::Ffmpeg => vec!["-i".into(), input.into(), "-y".into(), output.into()],
            Engine::LibreOffice => {
                // -env:UserInstallation 指向临时配置目录,跳过首次运行的用户配置/许可弹窗
                let profile =
                    std::env::temp_dir().join(format!("nextool_lo_{}", std::process::id()));
                let user_inst = format!("file://{}", profile.to_string_lossy());
                vec![
                    "--headless".into(),
                    "-env:UserInstallation".into(),
                    user_inst,
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
                ]
            }
            Engine::Calibre => vec![input.into(), output.into()],
            Engine::Pandoc => vec![input.into(), "-o".into(), output.into()],
            Engine::Ghostscript => vec![
                "-sDEVICE=pdfwrite".into(),
                "-dPDFSETTINGS=/ebook".into(),
                "-o".into(),
                output.into(),
                input.into(),
            ],
            Engine::Tesseract => {
                // tesseract 自动给输出加扩展名(.txt),output 参数传 basename(去扩展名)
                let base = std::path::Path::new(output)
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| output.to_string());
                vec![input.into(), base]
            }
        }
    }
}

/// 引擎执行端口:抽象子进程调用,隔离 PATH 探测与 spawn 副作用以便测试
///
/// prod 用 [`SubprocessRunner`];测试用桩实现(返回受控结果,记录调用参数)。
pub trait EngineRunner: Send + Sync {
    /// 引擎是否已安装(探测 PATH)
    fn is_available(&self, engine: Engine) -> bool;
    /// 执行引擎命令(input→output 文件路径)
    fn run(&self, engine: Engine, input: &str, output: &str) -> ToolResult<()>;
}

/// 生产适配器:真实 PATH 探测 + 子进程 spawn
pub struct SubprocessRunner;
impl EngineRunner for SubprocessRunner {
    fn is_available(&self, engine: Engine) -> bool {
        Command::new(engine.binary())
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok()
    }
    fn run(&self, engine: Engine, input: &str, output: &str) -> ToolResult<()> {
        let status = Command::new(engine.binary())
            .args(engine.convert_args(input, output))
            .status()
            .map_err(|e| ToolError::Other(format!("启动 {} 失败: {e}", engine.binary())))?;
        if !status.success() {
            return Err(ToolError::Other(format!(
                "{} 执行失败(退出码 {:?})",
                engine.binary(),
                status.code()
            )));
        }
        Ok(())
    }
}

/// 调引擎转换:检查可用性 → 委托执行 → 返回产物路径
///
/// 引擎未安装返回 Err(提示安装);执行失败返回 Err。产物落 output 指定路径。
/// 文件 IO(碰撞处理、落源目录)由 [`super::fs_util::engine_convert_file`] 包装。
pub fn engine_convert(
    runner: &dyn EngineRunner,
    engine: Engine,
    input: &str,
    output: &str,
) -> ToolResult<String> {
    if !runner.is_available(engine) {
        return Err(ToolError::InvalidInput(format!(
            "未检测到 {}({}),请先安装",
            engine.binary(),
            engine.desc()
        )));
    }
    runner.run(engine, input, output)?;
    Ok(output.to_string())
}

/// 引擎状态(探测结果):供 CLI/GUI 展示哪些引擎已装
#[derive(Debug, Clone)]
pub struct EngineStatus {
    pub engine: Engine,
    pub available: bool,
    pub binary: &'static str,
    pub desc: &'static str,
}

/// 探测所有引擎可用性,返回状态列表(供 CLI/GUI 展示引擎检查结果)
pub fn engine_statuses(runner: &dyn EngineRunner) -> Vec<EngineStatus> {
    Engine::all()
        .iter()
        .map(|&e| EngineStatus {
            engine: e,
            available: runner.is_available(e),
            binary: e.binary(),
            desc: e.desc(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn binary_names() {
        assert_eq!(Engine::Ffmpeg.binary(), "ffmpeg");
        assert_eq!(Engine::LibreOffice.binary(), "soffice");
        assert_eq!(Engine::Calibre.binary(), "ebook-convert");
        assert_eq!(Engine::Pandoc.binary(), "pandoc");
        assert_eq!(
            Engine::Ghostscript.binary(),
            if cfg!(windows) { "gswin64c" } else { "gs" }
        );
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
        // --convert-to 应取 output 扩展名 pdf;-env:UserInstallation 跳过首次弹窗
        assert!(args.contains(&"pdf".to_string()));
        assert!(args.contains(&"--headless".to_string()));
        assert!(args.iter().any(|a| a == "-env:UserInstallation"));
    }

    #[test]
    fn tesseract_convert_args_strips_ext() {
        // tesseract 自动加 .txt,output 须传 basename,否则生成 .txt.txt
        let args = Engine::Tesseract.convert_args("in.png", "out.txt");
        assert_eq!(args, vec!["in.png", "out"]);
    }

    #[test]
    fn pandoc_convert_args() {
        let args = Engine::Pandoc.convert_args("in.md", "out.html");
        assert_eq!(args, vec!["in.md", "-o", "out.html"]);
    }

    /// 测试桩:受控 available/succeed,记录最后一次 run 调用的引擎与路径
    struct FakeRunner {
        available: bool,
        succeed: bool,
        last: Mutex<Option<(Engine, String, String)>>,
    }
    impl FakeRunner {
        fn new(available: bool, succeed: bool) -> Self {
            Self {
                available,
                succeed,
                last: Mutex::new(None),
            }
        }
    }
    impl EngineRunner for FakeRunner {
        fn is_available(&self, _engine: Engine) -> bool {
            self.available
        }
        fn run(&self, engine: Engine, input: &str, output: &str) -> ToolResult<()> {
            *self.last.lock().unwrap() = Some((engine, input.into(), output.into()));
            if self.succeed {
                Ok(())
            } else {
                Err(ToolError::Other("fake failure".into()))
            }
        }
    }

    #[test]
    fn engine_convert_unavailable_fails() {
        let runner = FakeRunner::new(false, true);
        assert!(engine_convert(&runner, Engine::Ffmpeg, "in", "out").is_err());
    }

    #[test]
    fn engine_convert_available_runs_and_returns_output() {
        let runner = FakeRunner::new(true, true);
        let out = engine_convert(&runner, Engine::Ffmpeg, "in.mp4", "out.mp3").unwrap();
        assert_eq!(out, "out.mp3");
        let last = runner.last.lock().unwrap();
        let (engine, input, output) = last.as_ref().unwrap();
        assert_eq!(*engine, Engine::Ffmpeg);
        assert_eq!(input, "in.mp4");
        assert_eq!(output, "out.mp3");
    }

    #[test]
    fn engine_convert_run_failure_propagates() {
        let runner = FakeRunner::new(true, false);
        assert!(engine_convert(&runner, Engine::Ffmpeg, "in", "out").is_err());
    }

    #[test]
    #[ignore = "需真实安装 ffmpeg: cargo test -p nextool-core fileconv::engine::tests::ffmpeg_real -- --ignored"]
    fn ffmpeg_real() {
        if !SubprocessRunner.is_available(Engine::Ffmpeg) {
            return;
        }
        // 真实测试需音视频文件,此处仅验证探测 + 命令构造无 panic
        let _args = Engine::Ffmpeg.convert_args("in.mp4", "out.mp3");
    }
}
