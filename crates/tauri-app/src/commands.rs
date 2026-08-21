//! Tauri command 层:薄封装 nextool-core,供前端 invoke 调用
//!
//! 每个命令仅转发参数到 core 函数,不做业务逻辑;错误经 serde 序列化为前端可读字符串。

use nextool_core::ToolError;
use nextool_core::{CaseMode, HashAlgo, PasswordOpts};

/// 命令错误:序列化为字符串供前端展示
#[derive(Debug, serde::Serialize)]
pub struct CmdError(String);

impl From<ToolError> for CmdError {
    fn from(e: ToolError) -> Self {
        CmdError(e.to_string())
    }
}

type CmdResult<T> = Result<T, CmdError>;

// 字符串参数解析辅助:select/number 经前端传来的是字符串,core 需要 enum/数字
fn parse_hash_algo(s: &str) -> CmdResult<HashAlgo> {
    Ok(match s {
        "md5" => HashAlgo::Md5,
        "sha1" => HashAlgo::Sha1,
        "sha256" => HashAlgo::Sha256,
        "sha512" => HashAlgo::Sha512,
        _ => return Err(CmdError(format!("未知哈希算法: {s}"))),
    })
}

fn parse_case_mode(s: &str) -> CmdResult<CaseMode> {
    Ok(match s {
        "upper" => CaseMode::Upper,
        "lower" => CaseMode::Lower,
        "title" => CaseMode::Title,
        "snake" => CaseMode::Snake,
        "camel" => CaseMode::Camel,
        "kebab" => CaseMode::Kebab,
        _ => return Err(CmdError(format!("未知大小写模式: {s}"))),
    })
}

fn parse_bool(s: &str) -> bool {
    s.eq_ignore_ascii_case("true")
}

// 编解码

#[tauri::command]
pub fn base64_encode(input: String) -> CmdResult<String> {
    Ok(nextool_core::base64_encode(&input)?)
}
#[tauri::command]
pub fn base64_decode(input: String) -> CmdResult<String> {
    Ok(nextool_core::base64_decode(&input)?)
}
#[tauri::command]
pub fn url_encode(input: String) -> CmdResult<String> {
    Ok(nextool_core::url_encode(&input)?)
}
#[tauri::command]
pub fn url_decode(input: String) -> CmdResult<String> {
    Ok(nextool_core::url_decode(&input)?)
}
#[tauri::command]
pub fn html_encode(input: String) -> CmdResult<String> {
    Ok(nextool_core::html_encode(&input)?)
}
#[tauri::command]
pub fn html_decode(input: String) -> CmdResult<String> {
    Ok(nextool_core::html_decode(&input)?)
}
#[tauri::command]
pub fn hex_encode(input: String) -> CmdResult<String> {
    Ok(nextool_core::hex_encode(&input)?)
}
#[tauri::command]
pub fn hex_decode(input: String) -> CmdResult<String> {
    Ok(nextool_core::hex_decode(&input)?)
}
#[tauri::command]
pub fn jwt_decode(input: String) -> CmdResult<String> {
    Ok(nextool_core::jwt_decode(&input)?)
}

// 转换

#[tauri::command]
pub fn json_to_yaml(input: String) -> CmdResult<String> {
    Ok(nextool_core::json_to_yaml(&input)?)
}
#[tauri::command]
pub fn yaml_to_json(input: String) -> CmdResult<String> {
    Ok(nextool_core::yaml_to_json(&input)?)
}
#[tauri::command]
pub fn json_to_toml(input: String) -> CmdResult<String> {
    Ok(nextool_core::json_to_toml(&input)?)
}
#[tauri::command]
pub fn toml_to_json(input: String) -> CmdResult<String> {
    Ok(nextool_core::toml_to_json(&input)?)
}
#[tauri::command]
pub fn json_to_csv(input: String) -> CmdResult<String> {
    Ok(nextool_core::json_to_csv(&input)?)
}
#[tauri::command]
pub fn csv_to_json(input: String) -> CmdResult<String> {
    Ok(nextool_core::csv_to_json(&input)?)
}
#[tauri::command]
pub fn md_to_html(input: String) -> CmdResult<String> {
    Ok(nextool_core::md_to_html(&input)?)
}
#[tauri::command]
pub fn numbase_convert(input: String, from: u32, to: u32) -> CmdResult<String> {
    Ok(nextool_core::numbase_convert(&input, from, to)?)
}

// 格式化

#[tauri::command]
pub fn json_format(input: String) -> CmdResult<String> {
    Ok(nextool_core::json_format(&input)?)
}
#[tauri::command]
pub fn json_minify(input: String) -> CmdResult<String> {
    Ok(nextool_core::json_minify(&input)?)
}
#[tauri::command]
pub fn sql_format(input: String) -> CmdResult<String> {
    Ok(nextool_core::sql_format(&input)?)
}
#[tauri::command]
pub fn xml_format(input: String) -> CmdResult<String> {
    Ok(nextool_core::xml_format(&input)?)
}
#[tauri::command]
pub fn xml_minify(input: String) -> CmdResult<String> {
    Ok(nextool_core::xml_minify(&input)?)
}
#[tauri::command]
pub fn css_minify(input: String) -> CmdResult<String> {
    Ok(nextool_core::css_minify(&input)?)
}

// 生成器

#[tauri::command]
pub fn uuid_v4() -> CmdResult<String> {
    Ok(nextool_core::uuid_v4()?)
}
#[tauri::command]
pub fn uuid_v7() -> CmdResult<String> {
    Ok(nextool_core::uuid_v7()?)
}
#[tauri::command]
pub fn hash(input: String, algo: String) -> CmdResult<String> {
    Ok(nextool_core::hash(&input, parse_hash_algo(&algo)?)?)
}
#[tauri::command]
pub fn hmac_compute(input: String, algo: String, key: String) -> CmdResult<String> {
    Ok(nextool_core::hmac_compute(
        &input,
        &key,
        parse_hash_algo(&algo)?,
    )?)
}
#[tauri::command]
pub fn password_generate(
    length: usize,
    upper: String,
    lower: String,
    digits: String,
    symbols: String,
) -> CmdResult<String> {
    let opts = PasswordOpts {
        upper: parse_bool(&upper),
        lower: parse_bool(&lower),
        digits: parse_bool(&digits),
        symbols: parse_bool(&symbols),
    };
    Ok(nextool_core::password_generate(length, &opts)?)
}
#[tauri::command]
pub fn lorem_ipsum(paragraphs: usize) -> CmdResult<String> {
    Ok(nextool_core::lorem_ipsum(paragraphs)?)
}
#[tauri::command]
pub fn qr_svg(input: String) -> CmdResult<String> {
    Ok(nextool_core::qr_svg(&input)?)
}

// 文本

#[tauri::command]
pub fn case_convert(input: String, mode: String) -> CmdResult<String> {
    Ok(nextool_core::case_convert(&input, parse_case_mode(&mode)?)?)
}
#[tauri::command]
pub fn sort_lines(input: String) -> CmdResult<String> {
    Ok(nextool_core::sort_lines(&input)?)
}
#[tauri::command]
pub fn dedup_lines(input: String) -> CmdResult<String> {
    Ok(nextool_core::dedup_lines(&input)?)
}
#[tauri::command]
pub fn reverse_text(input: String) -> CmdResult<String> {
    Ok(nextool_core::reverse_text(&input)?)
}
#[tauri::command]
pub fn regex_match(input: String, pattern: String) -> CmdResult<String> {
    Ok(nextool_core::regex_match(&pattern, &input)?)
}
#[tauri::command]
pub fn regex_replace(input: String, pattern: String, replacement: String) -> CmdResult<String> {
    Ok(nextool_core::regex_replace(&pattern, &replacement, &input)?)
}
#[tauri::command]
pub fn diff_text(input: String, other: String) -> CmdResult<String> {
    Ok(nextool_core::diff_text(&input, &other)?)
}

// 加密

#[tauri::command]
pub fn aes_gcm_encrypt(input: String, password: String) -> CmdResult<String> {
    Ok(nextool_core::aes_gcm_encrypt(&input, &password)?)
}
#[tauri::command]
pub fn aes_gcm_decrypt(input: String, password: String) -> CmdResult<String> {
    Ok(nextool_core::aes_gcm_decrypt(&input, &password)?)
}
#[tauri::command]
pub fn rsa_keygen(bits: usize) -> CmdResult<String> {
    Ok(nextool_core::rsa_keygen(bits)?)
}
#[tauri::command]
pub fn rsa_encrypt(input: String, pub_pem: String) -> CmdResult<String> {
    Ok(nextool_core::rsa_encrypt(&input, &pub_pem)?)
}
#[tauri::command]
pub fn rsa_decrypt(input: String, priv_pem: String) -> CmdResult<String> {
    Ok(nextool_core::rsa_decrypt(&input, &priv_pem)?)
}
#[tauri::command]
pub fn pbkdf2(input: String, salt: String, iterations: u32) -> CmdResult<String> {
    Ok(nextool_core::kdf_pbkdf2(&input, &salt, iterations)?)
}
#[tauri::command]
pub fn argon2(input: String, salt: String) -> CmdResult<String> {
    Ok(nextool_core::kdf_argon2(&input, &salt)?)
}

// 网络/时间

#[tauri::command]
pub fn ipcalc(input: String) -> CmdResult<String> {
    Ok(nextool_core::ipcalc(&input)?)
}
#[tauri::command]
pub fn timestamp_to_human(input: String, tz: String) -> CmdResult<String> {
    let ts: i64 = input
        .trim()
        .parse()
        .map_err(|_| CmdError("时间戳需为整数".into()))?;
    Ok(nextool_core::timestamp_to_human(ts, &tz)?)
}
#[tauri::command]
pub fn timestamp_from_human(input: String, tz: String) -> CmdResult<String> {
    Ok(nextool_core::timestamp_from_human(&input, &tz)?)
}
#[tauri::command]
pub fn cron_next(input: String, count: usize) -> CmdResult<String> {
    Ok(nextool_core::cron_next(&input, count)?)
}
#[tauri::command]
pub fn dns_lookup(input: String, rtype: String) -> CmdResult<String> {
    Ok(nextool_core::dns_lookup(&input, &rtype)?)
}
