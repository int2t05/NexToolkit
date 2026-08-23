//! nextool-gui 库:Tauri 应用装配
//!
//! 注册文件工具命令 + 通用文本工具入口(list_tools/run_tool/list_file_tools),启动桌面窗口加载前端。
//! 文本工具经 run_tool 通用分发,不再逐个注册命令。命令定义在 [`commands`] 模块。

mod commands;

use commands::{
    archive_compress, archive_convert, archive_extract, archive_list, av_convert, convert_file,
    docx_to_text, engine_install_infos, font_convert, font_meta, image_adjust, image_compress_jpeg,
    image_convert, image_crop, image_filter, image_flip, image_resize, install_engine,
    json_to_xlsx, list_engines, list_file_tools, list_tools, ocr, pdf_add_page_numbers,
    pdf_decrypt, pdf_delete_pages, pdf_encrypt, pdf_extract_pages, pdf_merge, pdf_rotate,
    pdf_set_metadata, pdf_split, pdf_split_every_n, pdf_split_parity, pdf_split_ranges, run_tool,
    svg_convert, xlsx_to_json,
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
            engine_install_infos,
            install_engine,
            // 文件转换命令(签名各异,独立注册)
            convert_file,
            archive_list,
            archive_extract,
            archive_compress,
            archive_convert,
            image_adjust,
            image_compress_jpeg,
            image_convert,
            image_crop,
            image_filter,
            image_flip,
            image_resize,
            pdf_split,
            pdf_rotate,
            pdf_encrypt,
            pdf_decrypt,
            pdf_split_ranges,
            pdf_split_every_n,
            pdf_split_parity,
            pdf_merge,
            pdf_delete_pages,
            pdf_extract_pages,
            pdf_set_metadata,
            pdf_add_page_numbers,
            // 字体转换
            font_convert,
            font_meta,
            // SVG 栅格化
            svg_convert,
            // 引擎转换(运行时探测系统已装引擎)
            av_convert,
            ocr,
            // 电子表格
            xlsx_to_json,
            json_to_xlsx,
            // 文本提取
            docx_to_text,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
