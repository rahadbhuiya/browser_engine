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
}

