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
                <div class="card" style="background-color: #1e293b; padding: 20px; border-width: 2px; border-color: #6366f1;">
                    <h2 style="color: #818cf8; font-size: 24px;">Welcome to Diaz's Browser Engine</h2>
                    <p style="color: #94a3b8;">GPU-Accelerated • Native Rust HTML5/CSS3 Engine • Clickable Form Controls</p>
                    <br/>
                    <div style="background-color: #0f172a; padding: 14px; border-width: 1px; border-color: #38bdf8; margin-bottom: 12px;">
                        <p style="color: #38bdf8; font-weight: bold;">🔍 INTERACTIVE FORM & SEARCH HUB</p>
                        <p style="color: #cbd5e1;">Type any keywords or web address in the top Omnibox or click quick links below!</p>
                    </div>
                    <div style="background-color: #0f172a; padding: 10px; border-width: 1px; border-color: #6366f1;">
                        <p style="color: #818cf8; font-weight: bold;">★ QUICK LINKS & ACTIONS:</p>
                        <p style="color: #38bdf8; font-weight: bold;">[ Google Search ]  [ Rust Documentation ]  [ GitHub Repo ]</p>
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
        let mut html = String::from(r#"<div class="tab-strip" style="background-color: #0f172a; padding: 6px; border-bottom: 2px solid #334155;">"#);
        for (i, tab) in self.tabs.iter().enumerate() {
            let is_active = i == self.active_index;
            let bg_color = if is_active { "#1e293b" } else { "#0f172a" };
            let text_color = if is_active { "#38bdf8" } else { "#64748b" };
            let border = if is_active { "2px solid #6366f1" } else { "1px solid #334155" };
            
            html.push_str(&format!(
                r#"<span style="background-color: {}; color: {}; border: {}; padding: 6px 14px; margin-right: 6px; font-weight: bold; font-size: 13px;">[{}] {}</span>"#,
                bg_color, text_color, border, i + 1, tab.title
            ));
        }
        html.push_str(r#"<span style="color: #64748b; font-size: 12px; margin-left: 10px;">(ESC: Clear | Left/Right: Back/Forward)</span></div>"#);
        html
    }

    pub fn get_tab_at_click(&self, x: f32, y: f32) -> Option<usize> {
        if y <= 35.0 {
            if x >= 0.0 && x <= 140.0 && !self.tabs.is_empty() {
                return Some(0);
            } else if x >= 145.0 && x <= 285.0 && self.tabs.len() > 1 {
                return Some(1);
            } else if x >= 290.0 && x <= 430.0 && self.tabs.len() > 2 {
                return Some(2);
            }
        }
        None
    }
}
