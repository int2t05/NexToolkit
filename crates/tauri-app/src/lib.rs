//! nextool-gui 库:Tauri 应用装配
//!
//! 注册全部核心工具命令,启动桌面窗口加载前端。
//! 命令定义在 [`commands`] 模块,此处统一 `use` 后经 `generate_handler!` 注册。

mod commands;

use commands::{
    aes_gcm_decrypt, aes_gcm_encrypt, archive_compress, archive_convert, archive_extract,
    archive_list, argon2, base64_decode, base64_encode, case_convert, cron_next, css_minify,
    csv_to_json, dedup_lines, diff_text, dns_lookup, hash, hex_decode, hex_encode, hmac_compute,
    html_decode, html_encode, http_probe, image_convert, image_resize, ipcalc, json_format,
    json_minify, json_to_csv, json_to_toml, json_to_yaml, jwt_decode, jwt_verify, lorem_ipsum,
    md_to_html, numbase_convert, password_generate, pbkdf2, pdf_decrypt, pdf_encrypt, pdf_rotate,
    pdf_split, qr_svg, regex_match, regex_replace, reverse_text, rsa_decrypt, rsa_encrypt,
    rsa_keygen, rsa_sign, rsa_verify, sort_lines, sql_format, timestamp_from_human,
    timestamp_to_human, toml_to_json, unit_convert, url_decode, url_encode, uuid_v4, uuid_v7,
    xml_format, xml_minify, yaml_to_json,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            base64_encode,
            base64_decode,
            url_encode,
            url_decode,
            html_encode,
            html_decode,
            hex_encode,
            hex_decode,
            jwt_decode,
            jwt_verify,
            json_to_yaml,
            yaml_to_json,
            json_to_toml,
            toml_to_json,
            json_to_csv,
            csv_to_json,
            md_to_html,
            numbase_convert,
            unit_convert,
            json_format,
            json_minify,
            sql_format,
            xml_format,
            xml_minify,
            css_minify,
            uuid_v4,
            uuid_v7,
            hash,
            hmac_compute,
            password_generate,
            lorem_ipsum,
            qr_svg,
            case_convert,
            sort_lines,
            dedup_lines,
            reverse_text,
            regex_match,
            regex_replace,
            diff_text,
            aes_gcm_encrypt,
            aes_gcm_decrypt,
            rsa_keygen,
            rsa_encrypt,
            rsa_decrypt,
            rsa_sign,
            rsa_verify,
            pbkdf2,
            argon2,
            ipcalc,
            timestamp_to_human,
            timestamp_from_human,
            cron_next,
            dns_lookup,
            http_probe,
            archive_list,
            archive_extract,
            archive_compress,
            archive_convert,
            image_convert,
            image_resize,
            pdf_split,
            pdf_rotate,
            pdf_encrypt,
            pdf_decrypt
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
