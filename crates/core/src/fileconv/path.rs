//! 路径计算:产物输出路径与解压目录的纯字符串逻辑(无 IO,无文件系统访问)
//!
//! 供 [`super::fs_util`] IO 边界调用,计算碰撞后缀路径。独立于归档/图像域,可被各域复用。

/// 计算输出文件路径(纯逻辑,不访问文件系统)
///
/// `attempt` 0 → `bar.{ext}`;1 → `bar_converted.{ext}`;≥2 → `bar(n-1).{ext}`。
/// 保留输入路径的分隔符(`/` 或 `\`),支持复合扩展名如 `tar.gz`。
pub fn compute_output_path(input_path: &str, target_ext: &str, attempt: u32) -> String {
    let (dir, file_with_ext, sep) = split_path(input_path);
    let stem = strip_last_ext(file_with_ext);
    let name = match attempt {
        0 => format!("{stem}.{target_ext}"),
        1 => format!("{stem}_converted.{target_ext}"),
        n => format!("{stem}({}).{target_ext}", n - 1),
    };
    join_path(dir, sep, name)
}

/// 计算解压输出目录路径(纯逻辑)
///
/// `attempt` 0 → `bar_extracted`;≥1 → `bar_extracted(n)`。
pub fn compute_extract_dir(input_path: &str, attempt: u32) -> String {
    let (dir, file_with_ext, sep) = split_path(input_path);
    let stem = strip_last_ext(file_with_ext);
    let name = match attempt {
        0 => format!("{stem}_extracted"),
        n => format!("{stem}_extracted({n})"),
    };
    join_path(dir, sep, name)
}

/// 按最后一个 `/` 或 `\` 切分目录与文件名,返回 (dir, file, sep)
///
/// 无分隔符时 dir 为空、sep 为 `\0`(表示无分隔符);否则 sep 为该分隔符字符。
fn split_path(input: &str) -> (&str, &str, char) {
    match input.rfind(['/', '\\']) {
        Some(i) => {
            let sep = input.as_bytes()[i] as char;
            (&input[..i], &input[i + 1..], sep)
        }
        None => ("", input, '\0'),
    }
}

/// 去掉文件名最后一个扩展名;隐藏文件(`.bashrc`)保留,无扩展名保留原值
fn strip_last_ext(file: &str) -> &str {
    match file.rfind('.') {
        Some(i) if i > 0 => &file[..i],
        _ => file,
    }
}

/// 用原分隔符拼回路径;sep 为 `\0` 时表示无目录,直接返回 name
fn join_path(dir: &str, sep: char, name: String) -> String {
    if dir.is_empty() || sep == '\0' {
        name
    } else {
        format!("{dir}{sep}{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_path_unix_base() {
        assert_eq!(
            compute_output_path("/foo/bar.zip", "tar", 0),
            "/foo/bar.tar"
        );
    }

    #[test]
    fn output_path_collision_suffixes() {
        assert_eq!(
            compute_output_path("/foo/bar.zip", "tar", 1),
            "/foo/bar_converted.tar"
        );
        assert_eq!(
            compute_output_path("/foo/bar.zip", "tar", 2),
            "/foo/bar(1).tar"
        );
        assert_eq!(
            compute_output_path("/foo/bar.zip", "tar", 3),
            "/foo/bar(2).tar"
        );
    }

    #[test]
    fn output_path_compound_ext() {
        assert_eq!(
            compute_output_path("/foo/bar.zip", "tar.gz", 0),
            "/foo/bar.tar.gz"
        );
    }

    #[test]
    fn output_path_windows_separator_preserved() {
        assert_eq!(
            compute_output_path("C:\\foo\\bar.zip", "tar", 0),
            "C:\\foo\\bar.tar"
        );
    }

    #[test]
    fn output_path_no_dir() {
        assert_eq!(compute_output_path("bar.zip", "tar", 0), "bar.tar");
        assert_eq!(
            compute_output_path("bar.zip", "tar", 1),
            "bar_converted.tar"
        );
    }

    #[test]
    fn output_path_strips_last_ext_only() {
        assert_eq!(
            compute_output_path("/x/bar.tar.gz", "zip", 0),
            "/x/bar.tar.zip"
        );
    }

    #[test]
    fn extract_dir_base_and_collision() {
        assert_eq!(compute_extract_dir("/foo/bar.zip", 0), "/foo/bar_extracted");
        assert_eq!(
            compute_extract_dir("/foo/bar.zip", 1),
            "/foo/bar_extracted(1)"
        );
        assert_eq!(
            compute_extract_dir("/foo/bar.zip", 2),
            "/foo/bar_extracted(2)"
        );
    }
}
