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
        let mut html = String::from(r#"<div class="bookmarks-bar" style="background-color: #0f172a; padding: 6px 12px; border-bottom: 1px solid #1e293b;">"#);
        html.push_str(r#"<span style="font-weight: bold; color: #64748b; margin-right: 10px; font-size: 12px;">★ BOOKMARKS:</span>"#);
        for item in &self.items {
            html.push_str(&format!(
                r#"<span style="background-color: #1e293b; color: #38bdf8; border: 1px solid #334155; padding: 4px 10px; margin-right: 8px; font-size: 12px; font-weight: bold;">★ {}</span>"#,
                item.title
            ));
        }
        html.push_str(r#"</div>"#);
        html
    }

    pub fn get_url_at_click(&self, x: f32, y: f32) -> Option<String> {
        if y >= 75.0 && y <= 115.0 {
            if x >= 80.0 && x <= 160.0 && !self.items.is_empty() {
                return Some(self.items[0].url.clone());
            } else if x >= 165.0 && x <= 260.0 && self.items.len() > 1 {
                return Some(self.items[1].url.clone());
            } else if x >= 265.0 && x <= 340.0 && self.items.len() > 2 {
                return Some(self.items[2].url.clone());
            }
        }
        None
    }
}
