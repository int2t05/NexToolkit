//! nextool-gui 库:Tauri 应用装配
//!
//! 注册全部核心工具命令,启动桌面窗口加载前端。

mod commands;

use commands::all_commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(all_commands!())
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
