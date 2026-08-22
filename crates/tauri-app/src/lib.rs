//! nextool-gui 库:Tauri 应用装配
//!
//! 注册文件工具命令 + 通用文本工具入口(list_tools/run_tool/list_file_tools),启动桌面窗口加载前端。
//! 文本工具经 run_tool 通用分发,不再逐个注册命令。命令定义在 [`commands`] 模块。

mod commands;

use commands::{
    archive_compress, archive_convert, archive_extract, archive_list, av_convert, ebook_convert,
    image_convert, image_resize, list_engines, list_file_tools, list_tools, markup_convert, ocr,
    office_to_pdf, pdf_compress, pdf_decrypt, pdf_encrypt, pdf_rotate, pdf_split, run_tool,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // 通用入口(文本工具动态渲染与执行)
            list_tools,
            run_tool,
            list_file_tools,
            list_engines,
            // 文件转换命令(签名各异,独立注册)
            archive_list,
            archive_extract,
            archive_compress,
            archive_convert,
            image_convert,
            image_resize,
            pdf_split,
            pdf_rotate,
            pdf_encrypt,
            pdf_decrypt,
            // 引擎转换(运行时探测系统已装引擎)
            av_convert,
            office_to_pdf,
            ebook_convert,
            markup_convert,
            pdf_compress,
            ocr,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
