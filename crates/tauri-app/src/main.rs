//! nextool-gui:Tauri 桌面 GUI 二进制入口

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    nextool_gui_lib::run();
}
