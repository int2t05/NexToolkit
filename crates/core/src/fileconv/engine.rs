//! 引擎层:外部引擎子进程桥接(ffmpeg/LibreOffice/calibre/pandoc/ghostscript/tesseract)
//!
//! 纯逻辑部分(引擎枚举、命令构造)可单测;子进程调用经 [`EngineRunner`] port 抽象,
//! prod 用 [`SubprocessRunner`],测试用桩实现。核心包不捆绑重引擎,运行时探测系统已装,
//! 未装时返回错误提示安装。

use crate::{ToolError, ToolResult};
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

/// Windows 下隐藏子进程控制台窗口(CREATE_NO_WINDOW),避免引擎调用时弹出黑框
#[cfg(windows)]
fn hide_console(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_console(_cmd: &mut Command) {}

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

    /// 解析引擎可执行文件实际路径:先查 PATH,失败后查 Windows 常见安装路径。
    ///
    /// LibreOffice 在 Windows 下默认不加 PATH(装在 Program Files),需回退检查。
    /// 其他引擎 PATH 未装即返回 None,由调用方报"未安装"。
    pub fn resolve_binary(&self) -> Option<PathBuf> {
        let bin = self.binary();
        if which_lookup(bin).is_some() {
            return Some(PathBuf::from(bin));
        }
        // NexToolkit 安装目录(便携版引擎解压处)
        let installed = self.installed_path();
        if installed.is_file() {
            return Some(installed);
        }
        #[cfg(windows)]
        {
            for candidate in self.windows_common_paths() {
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        None
    }

    /// Windows 常见安装路径(LibreOffice 默认装在 Program Files 不加 PATH)
    #[cfg(windows)]
    fn windows_common_paths(&self) -> Vec<PathBuf> {
        match self {
            Engine::LibreOffice => vec![
                PathBuf::from(r"C:\Program Files\LibreOffice\program\soffice.exe"),
                PathBuf::from(r"C:\Program Files (x86)\LibreOffice\program\soffice.exe"),
            ],
            Engine::Tesseract => vec![
                PathBuf::from(r"C:\Program Files\Tesseract-OCR\tesseract.exe"),
                PathBuf::from(r"C:\Program Files (x86)\Tesseract-OCR\tesseract.exe"),
            ],
            _ => vec![],
        }
    }

    /// 是否有便携版(可自动下载解压到安装目录)
    pub fn is_portable(&self) -> bool {
        matches!(self, Engine::Ffmpeg | Engine::Pandoc)
    }

    /// 便携版/安装包下载页 URL
    pub fn download_url(&self) -> &'static str {
        match self {
            Engine::Ffmpeg => "https://github.com/BtbN/FFmpeg-Builds/releases",
            Engine::LibreOffice => "https://www.libreoffice.org/download/",
            Engine::Calibre => "https://calibre-ebook.com/download",
            Engine::Pandoc => "https://github.com/jgm/pandoc/releases/latest",
            Engine::Ghostscript => "https://www.ghostscript.com/releases.html",
            Engine::Tesseract => "https://github.com/UB-Mannheim/tesseract/releases",
        }
    }

    /// 引擎安装根目录:NexToolkit 数据目录下的 engines/
    pub fn install_root() -> PathBuf {
        #[cfg(windows)]
        {
            if let Some(appdata) = std::env::var_os("APPDATA") {
                return PathBuf::from(appdata).join("NexToolkit").join("engines");
            }
        }
        #[cfg(unix)]
        {
            if let Some(home) = std::env::var_os("HOME") {
                return PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("NexToolkit")
                    .join("engines");
            }
        }
        std::env::temp_dir().join("nextoolkit-engines")
    }

    /// 安装目录下该引擎的子目录
    pub fn install_subdir(&self) -> PathBuf {
        Self::install_root().join(self.install_name())
    }

    /// 安装目录下该引擎的二进制路径
    pub fn installed_path(&self) -> PathBuf {
        let bin = self.binary();
        let subdir = self.install_subdir();
        #[cfg(windows)]
        {
            // ffmpeg 解压后 bin/ffmpeg.exe;pandoc 解压后直接 pandoc.exe
            if matches!(self, Engine::Ffmpeg) {
                return subdir.join("bin").join(format!("{bin}.exe"));
            }
            subdir.join(format!("{bin}.exe"))
        }
        #[cfg(not(windows))]
        {
            subdir.join(bin)
        }
    }

    /// 安装目录名(用于子目录)
    fn install_name(&self) -> &'static str {
        match self {
            Engine::Ffmpeg => "ffmpeg",
            Engine::LibreOffice => "libreoffice",
            Engine::Calibre => "calibre",
            Engine::Pandoc => "pandoc",
            Engine::Ghostscript => "ghostscript",
            Engine::Tesseract => "tesseract",
        }
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
                // Windows 路径需 file:///C:/... 三斜杠;Unix file:///tmp/...
                let user_inst = if cfg!(windows) {
                    let p = profile.to_string_lossy().replace('\\', "/");
                    format!("file:///{p}")
                } else {
                    format!("file://{}", profile.to_string_lossy())
                };
                vec![
                    "--headless".into(),
                    format!("-env:UserInstallation={user_inst}"),
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

/// 查 PATH 中是否存在某二进制(通过尝试 `--version` 探测,不依赖 which 命令)
fn which_lookup(bin: &str) -> Option<PathBuf> {
    let mut cmd = Command::new(bin);
    cmd.arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    hide_console(&mut cmd);
    if cmd.status().is_ok() {
        Some(PathBuf::from(bin))
    } else {
        None
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
        engine.resolve_binary().is_some()
    }
    fn run(&self, engine: Engine, input: &str, output: &str) -> ToolResult<()> {
        let bin = engine
            .resolve_binary()
            .ok_or_else(|| ToolError::Other(format!("未找到 {} 可执行文件", engine.binary())))?;
        let mut cmd = Command::new(&bin);
        cmd.args(engine.convert_args(input, output));
        hide_console(&mut cmd);
        let output = cmd
            .output()
            .map_err(|e| ToolError::Other(format!("启动 {} 失败: {e}", bin.display())))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ToolError::Other(format!(
                "{} 执行失败(退出码 {:?}){}",
                bin.display(),
                output.status.code(),
                if stderr.trim().is_empty() {
                    String::new()
                } else {
                    format!(": {stderr}")
                }
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
    pub resolved_path: Option<String>,
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
            resolved_path: e.resolve_binary().map(|p| p.to_string_lossy().into_owned()),
        })
        .collect()
}

/// 引擎安装信息(供 CLI/GUI 展示安装选项)
#[derive(Debug, Clone)]
pub struct EngineInstallInfo {
    pub engine: Engine,
    pub binary: &'static str,
    pub desc: &'static str,
    pub available: bool,
    pub is_portable: bool,
    pub download_url: &'static str,
    pub install_path: Option<String>,
}

/// 返回所有引擎的安装信息
pub fn engine_install_infos() -> Vec<EngineInstallInfo> {
    Engine::all()
        .iter()
        .map(|&e| EngineInstallInfo {
            engine: e,
            binary: e.binary(),
            desc: e.desc(),
            available: SubprocessRunner.is_available(e),
            is_portable: e.is_portable(),
            download_url: e.download_url(),
            install_path: e.resolve_binary().map(|p| p.to_string_lossy().into_owned()),
        })
        .collect()
}

/// 解析引擎名(字符串 → Engine)
pub fn parse_engine(name: &str) -> ToolResult<Engine> {
    match name.to_ascii_lowercase().as_str() {
        "ffmpeg" => Ok(Engine::Ffmpeg),
        "libreoffice" | "soffice" => Ok(Engine::LibreOffice),
        "calibre" | "ebook-convert" => Ok(Engine::Calibre),
        "pandoc" => Ok(Engine::Pandoc),
        "ghostscript" | "gs" | "gswin64c" => Ok(Engine::Ghostscript),
        "tesseract" => Ok(Engine::Tesseract),
        _ => Err(ToolError::InvalidInput(format!("未知引擎: {name}"))),
    }
}

/// 下载并安装便携版引擎到安装目录,返回二进制路径
///
/// 仅支持便携版引擎(ffmpeg/pandoc)。安装包引擎返回 Err 提示手动安装。
pub fn install_engine(engine: Engine) -> ToolResult<String> {
    if !engine.is_portable() {
        return Err(ToolError::InvalidInput(format!(
            "{} 无便携版,请从 {} 下载安装包手动安装",
            engine.binary(),
            engine.download_url()
        )));
    }

    let install_dir = engine.install_subdir();
    if install_dir.exists() {
        std::fs::remove_dir_all(&install_dir)?;
    }
    std::fs::create_dir_all(&install_dir)?;

    let url = resolve_download_url(engine)?;
    eprintln!("下载 {url} ...");
    let zip_data = download(&url)?;
    eprintln!("解压到 {} ...", install_dir.display());
    extract_zip(&zip_data, &install_dir)?;

    let bin_path = engine.installed_path();
    if !bin_path.is_file() {
        return Err(ToolError::Other(format!(
            "安装后未找到二进制: {}(可能 zip 结构不同,请手动检查 {})",
            bin_path.display(),
            install_dir.display()
        )));
    }
    eprintln!("已安装 {}: {}", engine.binary(), bin_path.display());
    Ok(bin_path.to_string_lossy().into_owned())
}

/// 解析下载 URL(pandoc 需 GitHub API 查最新版本)
fn resolve_download_url(engine: Engine) -> ToolResult<String> {
    match engine {
        Engine::Ffmpeg => {
            #[cfg(windows)]
            {
                Ok("https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip".into())
            }
            #[cfg(not(windows))]
            {
                Err(ToolError::InvalidInput(
                    "ffmpeg 便携版仅支持 Windows,Linux 请用包管理器安装".into(),
                ))
            }
        }
        Engine::Pandoc => {
            // GitHub API 查最新 release 的 Windows zip 资产
            let api = "https://api.github.com/repos/jgm/pandoc/releases/latest";
            let resp = ureq::get(api)
                .set("User-Agent", "NexToolkit")
                .call()
                .map_err(|e| ToolError::Other(format!("查询 pandoc 最新版本失败: {e}")))?;
            let body: serde_json::Value = serde_json::from_reader(resp.into_reader())
                .map_err(|e| ToolError::Other(format!("解析 pandoc release 失败: {e}")))?;
            let assets = body["assets"]
                .as_array()
                .ok_or_else(|| ToolError::Other("pandoc release 无 assets 字段".into()))?;
            #[cfg(windows)]
            {
                let zip = assets.iter().find(|a| {
                    a["name"]
                        .as_str()
                        .map(|n| n.contains("windows") && n.ends_with(".zip"))
                        .unwrap_or(false)
                });
                let url = zip
                    .and_then(|a| a["browser_download_url"].as_str())
                    .ok_or_else(|| ToolError::Other("pandoc release 无 Windows zip 资产".into()))?;
                Ok(url.into())
            }
            #[cfg(not(windows))]
            {
                let _ = assets;
                Err(ToolError::InvalidInput(
                    "pandoc 便携版仅支持 Windows,其他平台请用包管理器安装".into(),
                ))
            }
        }
        _ => Err(ToolError::InvalidInput("该引擎无便携版下载 URL".into())),
    }
}

/// 下载文件到内存
fn download(url: &str) -> ToolResult<Vec<u8>> {
    let resp = ureq::get(url)
        .set("User-Agent", "NexToolkit")
        .call()
        .map_err(|e| ToolError::Other(format!("下载失败: {e}")))?;
    let mut buf = Vec::new();
    resp.into_reader()
        .read_to_end(&mut buf)
        .map_err(|e| ToolError::Other(format!("读取下载内容失败: {e}")))?;
    Ok(buf)
}

/// 解压 zip 到目录
fn extract_zip(data: &[u8], dest: &std::path::Path) -> ToolResult<()> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| ToolError::Other(format!("打开 zip 失败: {e}")))?;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| ToolError::Other(format!("读取 zip 条目失败: {e}")))?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest.join(path),
            None => continue,
        };
        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
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
        // --convert-to 应取 output 扩展名 pdf;-env:UserInstallation=... 跳过首次弹窗
        assert!(args.contains(&"pdf".to_string()));
        assert!(args.contains(&"--headless".to_string()));
        assert!(args.iter().any(|a| a.starts_with("-env:UserInstallation=")));
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
