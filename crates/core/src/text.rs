//! 文本工具模块:大小写/排序去重/反转/正则/Diff

use crate::{ToolError, ToolResult};

/// 大小写转换模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseMode {
    Upper,
    Lower,
    Title,
    Snake,
    Camel,
    Kebab,
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
    let re = regex::Regex::new(pattern).map_err(|e| ToolError::Parse(e.to_string()))?;
    let matches: Vec<String> = re
        .find_iter(input)
        .map(|m| m.as_str().to_string())
        .collect();
    Ok(matches.join("\n"))
}

/// 正则全局替换,支持 $0/$1 捕获组
pub fn regex_replace(pattern: &str, replacement: &str, input: &str) -> ToolResult<String> {
    let re = regex::Regex::new(pattern).map_err(|e| ToolError::Parse(e.to_string()))?;
    Ok(re.replace_all(input, replacement).into_owned())
}

/// 生成 unified diff 文本,带 +/- 行;a==b 返回空串
pub fn diff_text(a: &str, b: &str) -> ToolResult<String> {
    let diff = similar::TextDiff::from_lines(a, b);
    Ok(diff.unified_diff().to_string())
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
}
