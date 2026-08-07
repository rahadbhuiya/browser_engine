#[derive(Debug, Clone, PartialEq)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

pub struct BookmarkManager {
    pub items: Vec<Bookmark>,
}

impl BookmarkManager {
    pub fn new() -> Self {
        BookmarkManager {
            items: vec![
                Bookmark {
                    title: "Google".to_string(),
                    url: "https://google.com".to_string(),
                },
                Bookmark {
                    title: "Rust Docs".to_string(),
                    url: "https://doc.rust-lang.org".to_string(),
                },
                Bookmark {
                    title: "GitHub".to_string(),
                    url: "https://github.com".to_string(),
                },
            ],
        }
    }

    pub fn add(&mut self, title: &str, url: &str) {
        let title = if title.is_empty() { url } else { title };
        if !self.items.iter().any(|b| b.url == url) {
            self.items.push(Bookmark {
                title: title.to_string(),
                url: url.to_string(),
            });
        }
    }

    pub fn render_bookmarks_bar_html(&self) -> String {
        let mut html = String::from(r#"<div class="bookmarks-bar" style="background-color: #f5f7fa; padding: 4px 10px; border-bottom: 1px solid #d0d7de;">"#);
        html.push_str(r#"<span style="font-weight: bold; color: #57606a; margin-right: 8px; font-size: 12px;">★ BOOKMARKS:</span>"#);
        for item in &self.items {
            html.push_str(&format!(
                r#"<span style="background-color: #ffffff; color: #0969da; border: 1px solid #d0d7de; padding: 2px 8px; margin-right: 6px; font-size: 12px; font-weight: bold;">★ {}</span>"#,
                item.title
            ));
        }
        html.push_str(r#"</div>"#);
        html
    }
}
