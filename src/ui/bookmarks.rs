use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

impl Bookmark {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"title\":\"{}\",\"url\":\"{}\"}}",
            escape_json(&self.title),
            escape_json(&self.url)
        )
    }
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

    pub fn is_bookmarked(&self, url: &str) -> bool {
        let norm_url = url.trim().trim_end_matches('/');
        self.items
            .iter()
            .any(|b| b.url.trim().trim_end_matches('/') == norm_url)
    }

    pub fn add(&mut self, title: &str, url: &str) {
        let title = if title.is_empty() { url } else { title };
        if !self.is_bookmarked(url) {
            self.items.push(Bookmark {
                title: title.to_string(),
                url: url.to_string(),
            });
        }
    }

    pub fn remove_by_url(&mut self, url: &str) -> bool {
        let norm_url = url.trim().trim_end_matches('/');
        let orig_len = self.items.len();
        self.items
            .retain(|b| b.url.trim().trim_end_matches('/') != norm_url);
        self.items.len() < orig_len
    }

    pub fn toggle_bookmark(&mut self, title: &str, url: &str) -> bool {
        if self.is_bookmarked(url) {
            self.remove_by_url(url);
            false
        } else {
            self.add(title, url);
            true
        }
    }

    pub fn save_to_disk(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let entries: Vec<String> = self.items.iter().map(|b| b.to_json()).collect();
        let json = format!("[\n  {}\n]\n", entries.join(",\n  "));
        fs::write(path, json)
    }

    pub fn load_from_disk(path: &Path) -> Result<Self, std::io::Error> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let contents = fs::read_to_string(path)?;
        let objects = crate::net::cookie::parse_json_object_list(&contents);
        let mut mgr = BookmarkManager { items: Vec::new() };
        for obj in objects {
            if let (Some(title), Some(url)) = (obj.get("title"), obj.get("url")) {
                mgr.add(title, url);
            }
        }
        if mgr.items.is_empty() {
            mgr = Self::new();
        }
        Ok(mgr)
    }

    pub fn default_storage_path() -> PathBuf {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".browser_engine")
            .join("bookmarks.json")
    }

    pub fn render_bookmarks_bar_html(&self) -> String {
        let mut html = String::from(
            r#"<div class="bookmarks-bar" style="background-color: #111827; padding: 6px 16px; border-bottom: 1px solid #1f2937;">"#,
        );
        html.push_str(
            r#"<span style="font-weight: bold; color: #6b7280; margin-right: 12px; font-size: 12px;">★ BOOKMARKS:</span>"#,
        );
        for item in &self.items {
            html.push_str(&format!(
                r#"<span style="background-color: #1f2937; color: #60a5fa; border: 1px solid #374151; border-radius: 4px; padding: 4px 10px; margin-right: 8px; font-size: 12px; font-weight: bold;">{}</span>"#,
                item.title
            ));
        }
        html.push_str(r#"</div>"#);
        html
    }

    pub fn get_url_at_click(&self, x: f32, y: f32) -> Option<String> {
        if y >= 75.0 && y <= 115.0 {
            let mut cur_x = 80.0;
            for item in &self.items {
                let pill_width = ((item.title.len() * 8 + 20) as f32).max(60.0);
                if x >= cur_x && x <= (cur_x + pill_width) {
                    return Some(item.url.clone());
                }
                cur_x += pill_width + 8.0;
            }
        }
        None
    }
}

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_add_and_toggle() {
        let mut mgr = BookmarkManager::new();
        assert!(mgr.is_bookmarked("https://google.com"));
        assert!(!mgr.is_bookmarked("https://kernel.org"));

        let added = mgr.toggle_bookmark("Kernel", "https://kernel.org");
        assert!(added);
        assert!(mgr.is_bookmarked("https://kernel.org"));

        let removed = mgr.toggle_bookmark("Kernel", "https://kernel.org");
        assert!(!removed);
        assert!(!mgr.is_bookmarked("https://kernel.org"));
    }

    #[test]
    fn test_bookmark_disk_persistence() {
        let temp_dir = std::env::temp_dir().join(format!("test_bm_{}", std::process::id()));
        let bm_file = temp_dir.join("bookmarks.json");

        let mut mgr = BookmarkManager::new();
        mgr.add("DuckDuckGo", "https://duckduckgo.com");
        mgr.save_to_disk(&bm_file).unwrap();

        let loaded = BookmarkManager::load_from_disk(&bm_file).unwrap();
        assert!(loaded.is_bookmarked("https://duckduckgo.com"));
        assert!(loaded.is_bookmarked("https://google.com"));

        let _ = fs::remove_file(bm_file);
        let _ = fs::remove_dir(temp_dir);
    }
}
