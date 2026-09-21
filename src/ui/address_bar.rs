#[derive(Debug, Clone)]
pub struct AddressBar {
    pub url_text: String,
    pub is_focused: bool,
}

impl AddressBar {
    pub fn new(initial_url: &str) -> Self {
        AddressBar {
            url_text: initial_url.to_string(),
            is_focused: false,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if !c.is_control() {
            self.url_text.push(c);
        }
    }

    pub fn backspace(&mut self) {
        self.url_text.pop();
    }

    pub fn set_text(&mut self, text: &str) {
        self.url_text = text.to_string();
    }

    pub fn resolve_query(&self) -> String {
        crate::net::format_url_or_search_query(&self.url_text)
    }

    pub fn generate_suggestions(&self) -> Vec<String> {
        let input = self.url_text.trim().to_ascii_lowercase();
        if input.is_empty() {
            return Vec::new();
        }

        let popular_sites = [
            ("google", "https://google.com"),
            ("youtube", "https://youtube.com"),
            ("github", "https://github.com"),
            ("wikipedia", "https://wikipedia.org"),
            ("rust", "https://rust-lang.org"),
            ("reddit", "https://reddit.com"),
            ("twitter", "https://twitter.com"),
            ("chatgpt", "https://chatgpt.com"),
        ];

        let mut suggestions = Vec::new();

        for (keyword, url) in &popular_sites {
            if keyword.starts_with(&input) || input.contains(keyword) {
                suggestions.push(format!("🌐 Visit: {}", url));
            }
        }

        suggestions.push(format!("🔍 Google Search: \"{}\"", self.url_text));

        if !input.contains('.') && !input.contains('/') {
            suggestions.push(format!("🌐 https://www.{}.com", input));
        }

        suggestions.truncate(4);
        suggestions
    }

    pub fn render_suggestions_html(&self) -> String {
        let input = self.url_text.trim();
        if input.is_empty() || input.starts_with("http://") || input.starts_with("https://") {
            return String::new();
        }

        let suggestions = self.generate_suggestions();
        if suggestions.is_empty() {
            return String::new();
        }

        let mut html = String::from(r#"<div class="suggestions" style="background-color: #111827; border: 1px solid #374151; border-radius: 8px; padding: 4px; margin-top: 4px; box-shadow: 0 8px 24px rgba(0,0,0,0.6);">"#);
        for item in &suggestions {
            html.push_str(&format!(
                r#"<div style="padding: 6px 12px; color: #cbd5e1; font-size: 13px; font-weight: bold; border-bottom: 1px solid #1f2937;">{}</div>"#,
                item
            ));
        }
        html.push_str("</div>");
        html
    }
}
