#![allow(dead_code)]

pub mod downloader;
pub mod fetch;
pub mod http;
pub mod url_parser;
pub mod websocket;

pub use downloader::{DownloadItem, DownloadManager};
pub use fetch::fetch;
pub use http::{HttpRequest, HttpResponse, UserAgentMode};
pub use url_parser::{format_url_or_search_query, ParsedUrl, Scheme};
pub use websocket::{WebSocketClient, WebSocketFrame};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_search_queries_and_urls() {
        assert_eq!(format_url_or_search_query("rust tutorial"), "https://www.google.com/search?q=rust+tutorial");
        assert_eq!(format_url_or_search_query("example.com"), "https://example.com");
        assert_eq!(format_url_or_search_query("http://example.com"), "http://example.com");
    }

    #[test]
    fn parses_valid_http_and_https_urls() {

        let http_url = ParsedUrl::parse("http://example.com/page").unwrap();
        assert_eq!(http_url.scheme, Scheme::Http);
        assert_eq!(http_url.host, "example.com");
        assert_eq!(http_url.port, 80);
        assert_eq!(http_url.path_and_query, "/page");

        let https_url = ParsedUrl::parse("https://example.com:8443/api?query=1").unwrap();
        assert_eq!(https_url.scheme, Scheme::Https);
        assert_eq!(https_url.host, "example.com");
        assert_eq!(https_url.port, 8443);
        assert_eq!(https_url.path_and_query, "/api?query=1");
    }

    #[test]
    fn serializes_http_get_request() {
        let req = HttpRequest::new_get("example.com", "/index.html");
        let serialized = req.serialize();
        assert!(serialized.starts_with("GET /index.html HTTP/1.1\r\n"));
        assert!(serialized.contains("Host: example.com\r\n"));
    }

    #[test]
    fn test_user_agent_header_serialization() {
        let req = HttpRequest::new_get("google.com", "/search");
        let serialized = req.serialize();
        assert!(serialized.contains("User-Agent: Mozilla/5.0"));
        assert!(serialized.contains("Chrome/124.0.0.0"));
    }

    #[test]
    fn parses_http_response() {
        let raw_resp = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 13\r\n\r\nHello World!!";
        let resp = HttpResponse::parse(raw_resp).unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.status_text, "OK");
        assert_eq!(resp.headers.get("content-type"), Some(&"text/html".to_string()));
        assert_eq!(resp.body_as_string(), "Hello World!!");
    }

    #[test]
    fn test_download_manager() {
        let mut dm = DownloadManager::new();
        dm.downloads.push(DownloadItem {
            url: "https://example.com/file.zip".to_string(),
            filename: "file.zip".to_string(),
            bytes_downloaded: 1024,
            completed: true,
        });
        let html = dm.render_downloads_bar_html();
        assert!(html.contains("file.zip"));
        assert!(html.contains("1024 B"));
    }

    #[test]
    fn test_websocket_frame_serialization() {
        let frame = WebSocketFrame::new_text("Hello WebSocket");
        let bytes = frame.serialize();
        assert_eq!(bytes[0], 0x81);
        assert_eq!(bytes[1], 15);
    }
}
