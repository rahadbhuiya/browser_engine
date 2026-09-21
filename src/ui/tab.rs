use super::navbar::NavBarHistory;

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub page_body: String,
    pub scroll_y: f32,
    pub zoom_level: f32,
    pub history: NavBarHistory,
}

impl Tab {
    pub fn new(id: usize, title: &str, url: &str) -> Self {
        let mut history = NavBarHistory::default();
        history.navigate_to(url);
        Tab {
            id,
            title: title.to_string(),
            url: url.to_string(),
            page_body: format!(
                r#"
                <div style="background-color: #0b0f19; padding: 30px 16px; text-align: center;">
                    <h1 style="color: #60a5fa; font-size: 32px; margin-bottom: 8px;">🌐 Diaz's Web Browser</h1>
                    <p style="color: #9ca3af; font-size: 15px; margin-bottom: 24px;">Production-Ready Rust Browser Engine • GPU Accelerated • CSS3 & JS Runtime</p>
                    
                    <div class="card" style="background-color: #111827; border-radius: 12px; padding: 20px; border: 1px solid #1f2937; margin-bottom: 16px;">
                        <p style="color: #38bdf8; font-weight: bold; font-size: 16px; margin-bottom: 10px;">⚡ SPEED DIAL & POPULAR SITES</p>
                        <p style="color: #e2e8f0; font-size: 14px; margin: 8px 0;">[ 🔍 Google ]   [ 🐙 GitHub ]   [ 📺 YouTube ]   [ 🦀 Rust Docs ]   [ 📖 Wikipedia ]</p>
                        <p style="color: #6b7280; font-size: 13px; margin-top: 10px;">Click any bookmark above or type in the Omnibox to navigate.</p>
                    </div>
                </div>
                "#
            ),
            scroll_y: 0.0,
            zoom_level: 1.0,
            history,
        }
    }

    pub fn navigate_to(&mut self, title: &str, url: &str, body: &str) {
        self.title = title.to_string();
        self.url = url.to_string();
        self.page_body = body.to_string();
        self.scroll_y = 0.0;
        self.history.navigate_to(url);
    }

    pub fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level + 0.10).min(2.5);
    }

    pub fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level - 0.10).max(0.5);
    }

    pub fn reset_zoom(&mut self) {
        self.zoom_level = 1.0;
    }

    pub fn find_in_page(&self, query: &str) -> usize {
        if query.trim().is_empty() {
            return 0;
        }
        let q = query.to_ascii_lowercase();
        let body = self.page_body.to_ascii_lowercase();
        body.matches(&q).count()
    }
}

pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub active_index: usize,
    next_id: usize,
}

impl TabManager {
    pub fn new() -> Self {
        let default_tab = Tab::new(1, "New Tab", "https://google.com");
        TabManager {
            tabs: vec![default_tab],
            active_index: 0,
            next_id: 2,
        }
    }

    pub fn new_tab(&mut self, title: &str, url: &str) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let tab_title = if title.is_empty() { format!("Tab {}", id) } else { title.to_string() };
        self.tabs.push(Tab::new(id, &tab_title, url));
        self.active_index = self.tabs.len() - 1;
        id
    }

    pub fn close_tab(&mut self, index: usize) {
        if self.tabs.len() > 1 && index < self.tabs.len() {
            self.tabs.remove(index);
            if self.active_index >= self.tabs.len() {
                self.active_index = self.tabs.len() - 1;
            }
        }
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_index]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_index]
    }

    pub fn switch_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_index = index;
        }
    }

    pub fn render_tab_strip_html(&self) -> String {
        let mut html = String::from(r#"<div class="tab-strip" style="background-color: #0b0f19; padding: 6px 16px 0 16px; border-bottom: 1px solid #1f2937;">"#);
        for (i, tab) in self.tabs.iter().enumerate() {
            let is_active = i == self.active_index;
            let bg_color = if is_active { "#111827" } else { "#0b0f19" };
            let text_color = if is_active { "#38bdf8" } else { "#9ca3af" };
            let border = if is_active { "1px solid #374151" } else { "1px solid transparent" };
            
            html.push_str(&format!(
                r#"<span style="background-color: {}; color: {}; border: {}; border-radius: 6px 6px 0 0; padding: 6px 14px; margin-right: 4px; font-weight: bold; font-size: 13px;">[{}] {}  [x]</span>"#,
                bg_color, text_color, border, i + 1, tab.title
            ));
        }
        html.push_str(r#"<span style="background-color: #1f2937; color: #10b981; border: 1px solid #374151; border-radius: 4px; padding: 4px 10px; font-weight: bold; font-size: 13px;">[+]</span></div>"#);
        html
    }

    pub fn get_tab_at_click(&self, x: f32, y: f32) -> Option<usize> {
        if y <= 40.0 {
            let stride = 146.0;
            let tab_width = 140.0;
            for i in 0..self.tabs.len() {
                let start_x = i as f32 * stride;
                if x >= start_x && x < (start_x + tab_width - 25.0) {
                    return Some(i);
                }
            }
        }
        None
    }

    pub fn get_close_click(&self, x: f32, y: f32) -> Option<usize> {
        if y <= 40.0 {
            let stride = 146.0;
            let tab_width = 140.0;
            for i in 0..self.tabs.len() {
                let start_x = i as f32 * stride;
                if x >= (start_x + tab_width - 25.0) && x <= (start_x + tab_width) {
                    return Some(i);
                }
            }
        }
        None
    }

    pub fn is_new_tab_click(&self, x: f32, y: f32) -> bool {
        if y <= 40.0 {
            let stride = 146.0;
            let plus_x = self.tabs.len() as f32 * stride;
            x >= plus_x && x <= (plus_x + 50.0)
        } else {
            false
        }
    }
}
