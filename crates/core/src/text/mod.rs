//! 文本工具模块:大小写/排序去重/反转/正则/Diff/统计/修剪/制表/对齐/替换/转义/行号

mod tools;
pub use tools::*;

use crate::ToolResult;

/// 大小写转换模式
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr, strum::EnumString, strum::EnumIter,
)]
pub enum CaseMode {
    #[strum(serialize = "upper")]
    Upper,
    #[strum(serialize = "lower")]
    Lower,
    #[strum(serialize = "title")]
    Title,
    #[strum(serialize = "snake")]
    Snake,
    #[strum(serialize = "camel")]
    Camel,
    #[strum(serialize = "kebab")]
    Kebab,
}

/// 对齐方向:左/右/居中填充
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr, strum::EnumString, strum::EnumIter,
)]
pub enum Align {
    #[strum(serialize = "left")]
    Left,
    #[strum(serialize = "right")]
    Right,
    #[strum(serialize = "center")]
    Center,
}

/// 转义类型:POSIX Shell / C 字符串 / 正则元字符
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr, strum::EnumString, strum::EnumIter,
)]
pub enum EscapeKind {
    #[strum(serialize = "shell")]
    Shell,
    #[strum(serialize = "c")]
    C,
    #[strum(serialize = "regex")]
    Regex,
}

/// 按单词边界拆分:空格/下划线/连字符为分隔符,小写→大写为 camelCase 边界
fn split_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut prev: Option<char> = None;
    for ch in s.chars() {
        if ch == ' ' || ch == '_' || ch == '-' {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            prev = None;
            continue;
        }
        // 小写→大写:camelCase 边界,在此切分新词
        if let Some(p) = prev {
            if p.is_lowercase() && ch.is_uppercase() && !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
        }
        current.push(ch);
        prev = Some(ch);
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// 首字母大写,其余小写
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => {
            let mut result: String = first.to_uppercase().collect();
            for ch in chars {
                result.extend(ch.to_lowercase());
            }
            result
        }
        None => String::new(),
    }
}

/// 大小写转换:Upper/Lower/Title 直接处理;Snake/Camel/Kebab 按单词边界
pub fn case_convert(s: &str, mode: CaseMode) -> ToolResult<String> {
    let result = match mode {
        CaseMode::Upper => s.to_uppercase(),
        CaseMode::Lower => s.to_lowercase(),
        CaseMode::Title => {
            let mut out = String::with_capacity(s.len());
            let mut new_word = true;
            for ch in s.chars() {
                if ch.is_whitespace() {
                    out.push(ch);
                    new_word = true;
                } else if new_word {
                    out.extend(ch.to_uppercase());
                    new_word = false;
                } else {
                    out.extend(ch.to_lowercase());
                }
            }
            out
        }
        CaseMode::Snake => split_words(s)
            .iter()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join("_"),
        CaseMode::Camel => split_words(s)
            .iter()
            .enumerate()
            .map(|(i, w)| {
                if i == 0 {
                    w.to_lowercase()
                } else {
                    capitalize(w)
                }
            })
            .collect::<Vec<_>>()
            .join(""),
        CaseMode::Kebab => split_words(s)
            .iter()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join("-"),
    };
    Ok(result)
}

/// 按行排序(升序,Unicode 码点序),保留末尾换行行为
pub fn sort_lines(s: &str) -> ToolResult<String> {
    let has_trailing = s.ends_with('\n');
    let mut lines: Vec<&str> = s.lines().collect();
    lines.sort();
    let mut result = lines.join("\n");
    if has_trailing {
        result.push('\n');
    }
    Ok(result)
}

/// 按行去重,保留首次出现顺序,保留末尾换行行为
pub fn dedup_lines(s: &str) -> ToolResult<String> {
    let has_trailing = s.ends_with('\n');
    let lines: Vec<&str> = s.lines().collect();
    let mut seen = std::collections::HashSet::new();
    let mut result_lines: Vec<&str> = Vec::new();
    for line in lines {
        if seen.insert(line) {
            result_lines.push(line);
        }
    }
    let mut result = result_lines.join("\n");
    if has_trailing {
        result.push('\n');
    }
    Ok(result)
}

/// 按 Unicode 标量值反转字符(非字节)
pub fn reverse_text(s: &str) -> ToolResult<String> {
    Ok(s.chars().rev().collect())
}

/// 正则匹配:每个匹配各占一行,无匹配返回空串,非法正则返回 Err
pub fn regex_match(pattern: &str, input: &str) -> ToolResult<String> {
    let re = regex::Regex::new(pattern)?;
    let matches: Vec<String> = re
        .find_iter(input)
        .map(|m| m.as_str().to_string())
        .collect();
    Ok(matches.join("\n"))
}

/// 正则全局替换,支持 $0/$1 捕获组
pub fn regex_replace(pattern: &str, replacement: &str, input: &str) -> ToolResult<String> {
    let re = regex::Regex::new(pattern)?;
    Ok(re.replace_all(input, replacement).into_owned())
}

/// 生成 unified diff 文本,带 +/- 行;a==b 返回空串
pub fn diff_text(a: &str, b: &str) -> ToolResult<String> {
    let diff = similar::TextDiff::from_lines(a, b);
    Ok(diff.unified_diff().to_string())
}

/// 文本统计:字符数/字数/行数/字节数,格式化为多行文本
///
/// 字数按 Unicode 空白分隔;行数按 `\n` 切分(`lines()` 语义);字符数为 Unicode 标量值数;
/// 字节数为 UTF-8 编码字节数。
pub fn text_stats(input: &str) -> ToolResult<String> {
    let chars = input.chars().count();
    let bytes = input.len();
    let words = input.split_whitespace().count();
    let lines = input.lines().count();
    Ok(format!(
        "字符数: {chars}\n字数: {words}\n行数: {lines}\n字节数: {bytes}"
    ))
}

/// 删除空行并对每行去首尾空白,保留末尾换行行为
pub fn text_trim_blank(input: &str) -> ToolResult<String> {
    let has_trailing = input.ends_with('\n');
    let lines: Vec<&str> = input
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let mut result = lines.join("\n");
    if has_trailing && !lines.is_empty() {
        result.push('\n');
    }
    Ok(result)
}

/// Tab → 空格:每个制表符替换为 n 个空格;n == 0 返回 Err
pub fn text_tab_to_space(input: &str, n: usize) -> ToolResult<String> {
    if n == 0 {
        return Err(crate::ToolError::InvalidInput("空格数不能为 0".into()));
    }
    let spaces = " ".repeat(n);
    Ok(input.replace('\t', &spaces))
}

/// 空格 → Tab:每 n 个连续空格替换为 1 个制表符;n == 0 返回 Err
pub fn text_space_to_tab(input: &str, n: usize) -> ToolResult<String> {
    if n == 0 {
        return Err(crate::ToolError::InvalidInput("空格数不能为 0".into()));
    }
    let pattern = " ".repeat(n);
    Ok(input.replace(&pattern, "\t"))
}

/// 按指定方向与宽度对齐每行:不足宽度补空格,已超宽行原样保留
pub fn text_align(input: &str, direction: Align, width: usize) -> ToolResult<String> {
    let has_trailing = input.ends_with('\n');
    let lines: Vec<String> = input
        .lines()
        .map(|l| {
            let len = l.chars().count();
            if len >= width {
                return l.to_string();
            }
            let pad = width - len;
            match direction {
                Align::Left => format!("{l}{}", " ".repeat(pad)),
                Align::Right => format!("{}{l}", " ".repeat(pad)),
                Align::Center => {
                    let left = pad / 2;
                    let right = pad - left;
                    format!("{}{l}{}", " ".repeat(left), " ".repeat(right))
                }
            }
        })
        .collect();
    let mut result = lines.join("\n");
    if has_trailing {
        result.push('\n');
    }
    Ok(result)
}

/// 查找替换:`use_regex` 为真时按正则全局替换(支持 `$0`/`$1` 捕获组、`(?i)`/`(?m)` 标志),
/// 否则按字面量全局替换
pub fn text_replace(input: &str, find: &str, replace: &str, use_regex: bool) -> ToolResult<String> {
    if use_regex {
        let re = regex::Regex::new(find)?;
        Ok(re.replace_all(input, replace).into_owned())
    } else {
        Ok(input.replace(find, replace))
    }
}

/// 转义:Shell(POSIX 单引号包裹)、C(反斜杠序列)、Regex(正则元字符)
pub fn text_escape(input: &str, kind: EscapeKind) -> ToolResult<String> {
    match kind {
        // POSIX shell:单引号包裹,内部单引号以 '\'' 转义
        EscapeKind::Shell => {
            let escaped = input.replace('\'', "'\\''");
            Ok(format!("'{escaped}'"))
        }
        // C 字符串:反斜杠转义控制字符与特殊符号
        EscapeKind::C => {
            let mut out = String::with_capacity(input.len() + 8);
            for ch in input.chars() {
                match ch {
                    '\\' => out.push_str("\\\\"),
                    '"' => out.push_str("\\\""),
                    '\n' => out.push_str("\\n"),
                    '\t' => out.push_str("\\t"),
                    '\r' => out.push_str("\\r"),
                    c if c.is_control() => out.push_str(&format!("\\x{:02x}", c as u32)),
                    c => out.push(c),
                }
            }
            Ok(out)
        }
        EscapeKind::Regex => Ok(regex::escape(input)),
    }
}

/// 行号添加:每行前缀右对齐的行号(从 `start` 起),格式 `{:>6}: 行内容`
pub fn text_number_lines(input: &str, start: u32) -> ToolResult<String> {
    let has_trailing = input.ends_with('\n');
    let mut n = start;
    let lines: Vec<String> = input
        .lines()
        .map(|l| {
            let s = format!("{n:>6}: {l}");
            n += 1;
            s
        })
        .collect();
    let mut result = lines.join("\n");
    if has_trailing {
        result.push('\n');
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- case_convert ----

    #[test]
    fn case_convert_upper_lower_title() {
        assert_eq!(case_convert("hello", CaseMode::Upper).unwrap(), "HELLO");
        assert_eq!(case_convert("HELLO", CaseMode::Lower).unwrap(), "hello");
        assert_eq!(
            case_convert("hello world", CaseMode::Title).unwrap(),
            "Hello World"
        );
        assert_eq!(
            case_convert("HELLO WORLD", CaseMode::Title).unwrap(),
            "Hello World"
        );
    }

    #[test]
    fn case_convert_snake() {
        assert_eq!(
            case_convert("hello world", CaseMode::Snake).unwrap(),
            "hello_world"
        );
        assert_eq!(
            case_convert("HelloWorld", CaseMode::Snake).unwrap(),
            "hello_world"
        );
        assert_eq!(
            case_convert("hello-world", CaseMode::Snake).unwrap(),
            "hello_world"
        );
    }

    #[test]
    fn case_convert_camel() {
        assert_eq!(
            case_convert("hello_world", CaseMode::Camel).unwrap(),
            "helloWorld"
        );
        assert_eq!(
            case_convert("hello world", CaseMode::Camel).unwrap(),
            "helloWorld"
        );
        assert_eq!(
            case_convert("hello-world", CaseMode::Camel).unwrap(),
            "helloWorld"
        );
    }

    #[test]
    fn case_convert_kebab() {
        assert_eq!(
            case_convert("HelloWorld", CaseMode::Kebab).unwrap(),
            "hello-world"
        );
        assert_eq!(
            case_convert("hello_world", CaseMode::Kebab).unwrap(),
            "hello-world"
        );
        assert_eq!(
            case_convert("hello-world", CaseMode::Kebab).unwrap(),
            "hello-world"
        );
    }

    // ---- sort_lines ----

    #[test]
    fn sort_lines_basic() {
        assert_eq!(sort_lines("b\na\nc").unwrap(), "a\nb\nc");
        assert_eq!(sort_lines("c\na\nb\n").unwrap(), "a\nb\nc\n");
    }

    #[test]
    fn sort_lines_unicode() {
        assert_eq!(sort_lines("b\na\n中").unwrap(), "a\nb\n中");
    }

    #[test]
    fn sort_lines_edge() {
        assert_eq!(sort_lines("").unwrap(), "");
        assert_eq!(sort_lines("single").unwrap(), "single");
    }

    // ---- dedup_lines ----

    #[test]
    fn dedup_lines_basic() {
        assert_eq!(dedup_lines("a\na\nb").unwrap(), "a\nb");
    }

    #[test]
    fn dedup_lines_trailing() {
        assert_eq!(dedup_lines("a\na\nb\n").unwrap(), "a\nb\n");
    }

    #[test]
    fn dedup_lines_preserve_order() {
        assert_eq!(dedup_lines("c\na\nc\nb\na").unwrap(), "c\na\nb");
    }

    #[test]
    fn dedup_lines_all_same() {
        assert_eq!(dedup_lines("x\nx\nx").unwrap(), "x");
    }

    // ---- reverse_text ----

    #[test]
    fn reverse_text_basic() {
        assert_eq!(reverse_text("abc").unwrap(), "cba");
    }

    #[test]
    fn reverse_text_unicode() {
        assert_eq!(reverse_text("中文").unwrap(), "文中");
    }

    #[test]
    fn reverse_text_edge() {
        assert_eq!(reverse_text("").unwrap(), "");
        assert_eq!(reverse_text("a").unwrap(), "a");
    }

    // ---- regex_match ----

    #[test]
    fn regex_match_basic() {
        assert_eq!(regex_match(r"\d+", "a12b3").unwrap(), "12\n3");
    }

    #[test]
    fn regex_match_no_match() {
        assert_eq!(regex_match(r"\d+", "abc").unwrap(), "");
    }

    #[test]
    fn regex_match_multiple() {
        assert_eq!(regex_match(r"[a-z]+", "Hello World").unwrap(), "ello\norld");
    }

    #[test]
    fn regex_match_invalid() {
        assert!(regex_match(r"(", "abc").is_err());
    }

    // ---- regex_replace ----

    #[test]
    fn regex_replace_capture() {
        assert_eq!(regex_replace(r"\d+", "#$0", "a12b3").unwrap(), "a#12b#3");
    }

    #[test]
    fn regex_replace_plain() {
        assert_eq!(regex_replace(r"\d+", "#", "a12b3").unwrap(), "a#b#");
    }

    #[test]
    fn regex_replace_group() {
        assert_eq!(
            regex_replace(r"(\w+)@(\w+)", "$2@$1", "user@host").unwrap(),
            "host@user"
        );
    }

    #[test]
    fn regex_replace_invalid() {
        assert!(regex_replace(r"[", "x", "abc").is_err());
    }

    // ---- diff_text ----

    #[test]
    fn diff_text_change() {
        let result = diff_text("a\nb", "a\nc").unwrap();
        assert!(result.contains("-b"), "应包含 -b: {result}");
        assert!(result.contains("+c"), "应包含 +c: {result}");
    }

    #[test]
    fn diff_text_identical() {
        assert_eq!(diff_text("same", "same").unwrap(), "");
        assert_eq!(diff_text("", "").unwrap(), "");
    }

    #[test]
    fn diff_text_addition() {
        let result = diff_text("", "new").unwrap();
        assert!(result.contains("+new"), "应包含 +new: {result}");
    }

    #[test]
    fn diff_text_deletion() {
        let result = diff_text("old", "").unwrap();
        assert!(result.contains("-old"), "应包含 -old: {result}");
    }

    // ---- text_stats ----

    #[test]
    fn text_stats_basic() {
        let s = "hello world\n中文";
        let out = text_stats(s).unwrap();
        assert!(out.contains("字符数: 14"), "{out}"); // 11 ASCII + 1 换行 + 2 CJK
        assert!(out.contains("字数: 3"), "{out}"); // hello / world / 中文
        assert!(out.contains("行数: 2"), "{out}");
        assert!(out.contains("字节数: 18"), "{out}"); // 11 + 1(\n) + 2*3
    }

    #[test]
    fn text_stats_empty() {
        let out = text_stats("").unwrap();
        assert!(out.contains("字符数: 0"), "{out}");
        assert!(out.contains("字数: 0"), "{out}");
        assert!(out.contains("行数: 0"), "{out}");
        assert!(out.contains("字节数: 0"), "{out}");
    }

    #[test]
    fn text_stats_trailing_newline() {
        // "a\n" → lines() 得 1 行
        let out = text_stats("a\n").unwrap();
        assert!(out.contains("行数: 1"), "{out}");
        assert!(out.contains("字数: 1"), "{out}");
    }

    // ---- text_trim_blank ----

    #[test]
    fn text_trim_blank_removes_blank_and_trims() {
        assert_eq!(
            text_trim_blank("  hello  \n\n  world  ").unwrap(),
            "hello\nworld"
        );
    }

    #[test]
    fn text_trim_blank_all_blank() {
        assert_eq!(text_trim_blank("  \n\n  \n").unwrap(), "");
    }

    #[test]
    fn text_trim_blank_preserves_trailing() {
        assert_eq!(text_trim_blank("a\n\nb\n").unwrap(), "a\nb\n");
    }

    #[test]
    fn text_trim_blank_empty() {
        assert_eq!(text_trim_blank("").unwrap(), "");
    }

    // ---- tab/space ----

    #[test]
    fn tab_to_space_basic() {
        assert_eq!(text_tab_to_space("a\tb\tc", 4).unwrap(), "a    b    c");
    }

    #[test]
    fn space_to_tab_basic() {
        assert_eq!(text_space_to_tab("a    b    c", 4).unwrap(), "a\tb\tc");
    }

    #[test]
    fn tab_space_roundtrip() {
        let s = "a\tb\tc";
        let spaced = text_tab_to_space(s, 2).unwrap();
        assert_eq!(text_space_to_tab(&spaced, 2).unwrap(), s);
    }

    #[test]
    fn tab_space_zero_rejected() {
        assert!(text_tab_to_space("a", 0).is_err());
        assert!(text_space_to_tab("a", 0).is_err());
    }

    #[test]
    fn space_to_tab_uneven() {
        // 3 空格、n=2 → 1 tab + 1 残留空格
        assert_eq!(text_space_to_tab("a   b", 2).unwrap(), "a\t b");
    }

    // ---- text_align ----

    #[test]
    fn text_align_left() {
        assert_eq!(
            text_align("ab\nabc", Align::Left, 5).unwrap(),
            "ab   \nabc  "
        );
    }

    #[test]
    fn text_align_right() {
        assert_eq!(
            text_align("ab\nabc", Align::Right, 5).unwrap(),
            "   ab\n  abc"
        );
    }

    #[test]
    fn text_align_center() {
        assert_eq!(text_align("ab", Align::Center, 6).unwrap(), "  ab  ");
        assert_eq!(text_align("abc", Align::Center, 6).unwrap(), " abc  "); // pad=3 → left=1,right=2
    }

    #[test]
    fn text_align_wider_than_width_unchanged() {
        assert_eq!(text_align("hello", Align::Left, 3).unwrap(), "hello");
    }

    #[test]
    fn text_align_preserves_trailing() {
        assert_eq!(text_align("ab\n", Align::Right, 4).unwrap(), "  ab\n");
    }

    // ---- text_replace ----

    #[test]
    fn text_replace_plain() {
        assert_eq!(
            text_replace("foo bar foo", "foo", "baz", false).unwrap(),
            "baz bar baz"
        );
    }

    #[test]
    fn text_replace_regex_capture() {
        assert_eq!(
            text_replace("a12b3", r"\d+", "#$0", true).unwrap(),
            "a#12b#3"
        );
    }

    #[test]
    fn text_replace_regex_case_insensitive() {
        assert_eq!(
            text_replace("Hello hello", r"(?i)hello", "HI", true).unwrap(),
            "HI HI"
        );
    }

    #[test]
    fn text_replace_regex_multiline() {
        // (?m) 使 ^ 匹配行首
        assert_eq!(
            text_replace("a\nb\nc", r"(?m)^", "X", true).unwrap(),
            "Xa\nXb\nXc"
        );
    }

    #[test]
    fn text_replace_regex_invalid() {
        assert!(text_replace("abc", r"[", "x", true).is_err());
    }

    // ---- text_escape ----

    #[test]
    fn text_escape_shell_simple() {
        assert_eq!(
            text_escape("hello world", EscapeKind::Shell).unwrap(),
            "'hello world'"
        );
    }

    #[test]
    fn text_escape_shell_single_quote() {
        // it's → 'it'\''s'
        assert_eq!(
            text_escape("it's", EscapeKind::Shell).unwrap(),
            "'it'\\''s'"
        );
    }

    #[test]
    fn text_escape_shell_empty() {
        assert_eq!(text_escape("", EscapeKind::Shell).unwrap(), "''");
    }

    #[test]
    fn text_escape_c_basic() {
        assert_eq!(
            text_escape("a\"b\\c\n\t", EscapeKind::C).unwrap(),
            "a\\\"b\\\\c\\n\\t"
        );
    }

    #[test]
    fn text_escape_c_control() {
        assert_eq!(text_escape("\x01", EscapeKind::C).unwrap(), "\\x01");
    }

    #[test]
    fn text_escape_c_preserves_unicode() {
        assert_eq!(text_escape("中文", EscapeKind::C).unwrap(), "中文");
    }

    #[test]
    fn text_escape_regex_metachars() {
        // regex::escape 转义所有非字母数字
        let out = text_escape("a.b*c", EscapeKind::Regex).unwrap();
        assert_eq!(out, r"a\.b\*c");
    }

    // ---- text_number_lines ----

    #[test]
    fn text_number_lines_basic() {
        let out = text_number_lines("a\nb\nc", 1).unwrap();
        assert_eq!(out, "     1: a\n     2: b\n     3: c");
    }

    #[test]
    fn text_number_lines_start_zero() {
        let out = text_number_lines("x\ny", 0).unwrap();
        assert_eq!(out, "     0: x\n     1: y");
    }

    #[test]
    fn text_number_lines_preserves_trailing() {
        let out = text_number_lines("a\n", 1).unwrap();
        assert_eq!(out, "     1: a\n");
    }

    #[test]
    fn text_number_lines_large_number_width() {
        // 行号 ≥ 7 位时不截断(右对齐 {:>6} 仅设最小宽度)
        let out = text_number_lines("x", 1_000_000).unwrap();
        assert_eq!(out, "1000000: x");
    }
}
