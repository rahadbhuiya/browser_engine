#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub is_visible: bool,
    pub x: f32,
    pub y: f32,
    pub items: Vec<String>,
}

impl ContextMenu {
    pub fn new() -> Self {
        ContextMenu {
            is_visible: false,
            x: 0.0,
            y: 0.0,
            items: vec![
                "⬅ Back".to_string(),
                "➡ Forward".to_string(),
                "🔄 Reload".to_string(),
                "📋 Copy URL".to_string(),
                "🔍 Inspect Element".to_string(),
            ],
        }
    }

    pub fn show(&mut self, x: f32, y: f32) {
        self.is_visible = true;
        self.x = x;
        self.y = y;
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
    }

    pub fn get_item_at_click(&self, click_x: f32, click_y: f32) -> Option<&str> {
        if !self.is_visible {
            return None;
        }

        let width = 160.0;
        let item_height = 28.0;
        let total_height = self.items.len() as f32 * item_height;

        if click_x >= self.x && click_x <= (self.x + width) && click_y >= self.y && click_y <= (self.y + total_height) {
            let index = ((click_y - self.y) / item_height) as usize;
            if index < self.items.len() {
                return Some(&self.items[index]);
            }
        }
        None
    }

    pub fn render_html(&self) -> String {
        if !self.is_visible {
            return String::new();
        }

        let mut html = format!(
            r#"<div class="context-menu" style="position: absolute; left: {}px; top: {}px; width: 160px; background-color: #1e293b; border: 1px solid #475569; border-radius: 8px; box-shadow: 0 8px 24px rgba(0,0,0,0.6); z-index: 1000; padding: 4px 0;">"#,
            self.x, self.y
        );

        for item in &self.items {
            html.push_str(&format!(
                r#"<div style="padding: 6px 14px; color: #cbd5e1; font-size: 13px; font-weight: bold; border-bottom: 1px solid #334155;">{}</div>"#,
                item
            ));
        }

        html.push_str("</div>");
        html
    }
}
