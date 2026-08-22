//! HTTP 探测模块:URL 状态码与响应头(纯 Rust TLS,无 OpenSSL)
//!
//! 发起 HEAD/GET 请求,返回状态码、最终 URL(重定向后)、关键响应头。

use crate::{ToolError, ToolResult};

/// HTTP 探测结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpProbe {
    /// 最终状态码(重定向后)
    pub status: u16,
    /// 最终 URL(经历重定向后的落地 URL)
    pub final_url: String,
    /// Content-Type 响应头(无则空)
    pub content_type: String,
    /// Content-Length 响应头(无则空)
    pub content_length: String,
    /// Server 响应头(无则空)
    pub server: String,
}

impl HttpProbe {
    /// 格式化为多行文本供 CLI/GUI 展示
    pub fn to_display(&self) -> String {
        format!(
            "状态码: {}\n最终 URL: {}\nContent-Type: {}\nContent-Length: {}\nServer: {}",
            self.status, self.final_url, self.content_type, self.content_length, self.server
        )
    }
}

/// 探测 URL:发起 HEAD 请求(跟随重定向),返回状态码与关键响应头
///
/// URL 须含 scheme(http/https),非法格式或连接失败返回 Err。
pub fn http_probe(url: &str) -> ToolResult<HttpProbe> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ToolError::InvalidInput(
            "URL 须以 http:// 或 https:// 开头".into(),
        ));
    }
    let resp = ureq::head(url)
        .call()
        .map_err(|e| ToolError::Other(format!("HTTP 请求失败: {e}")))?;
    let status = resp.status();
    let final_url = resp.get_url().to_string();
    let content_type = header_string(&resp, "content-type");
    let content_length = header_string(&resp, "content-length");
    let server = header_string(&resp, "server");
    Ok(HttpProbe {
        status,
        final_url,
        content_type,
        content_length,
        server,
    })
}

/// 读取响应头(大小写不敏感),无则空串
fn header_string(resp: &ureq::Response, name: &str) -> String {
    resp.header(name).unwrap_or_default().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_url_rejected() {
        assert!(http_probe("not a url").is_err());
        assert!(http_probe("ftp://example.com").is_err());
        assert!(http_probe("example.com").is_err());
    }

    #[test]
    #[ignore = "需真实网络: cargo test -p nextool-core http::tests::probe_real -- --ignored"]
    fn probe_real() {
        // 探测已知稳定站点,验证状态码 200 且有 content-type
        let probe = http_probe("https://example.com").unwrap();
        assert_eq!(probe.status, 200);
        assert!(!probe.content_type.is_empty(), "应返回 content-type");
        assert!(probe.final_url.starts_with("http"));
    }

    #[test]
    #[ignore = "需真实网络: cargo test -p nextool-core http::tests::probe_redirect -- --ignored"]
    fn probe_redirect() {
        // http://example.com 应跟随到 https(或保持),最终 URL 合法
        let probe = http_probe("http://example.com").unwrap();
        assert!(probe.final_url.starts_with("http"));
    }

    #[test]
    fn display_format() {
        let probe = HttpProbe {
            status: 200,
            final_url: "https://example.com".into(),
            content_type: "text/html".into(),
            content_length: "1234".into(),
            server: "nginx".into(),
        };
        let display = probe.to_display();
        assert!(display.contains("状态码: 200"));
        assert!(display.contains("Content-Type: text/html"));
    }
}
