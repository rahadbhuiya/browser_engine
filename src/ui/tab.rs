use super::navbar::NavBarHistory;

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub page_body: String,
    /// Current rendered scroll offset (pixels from document top).
    pub scroll_y: f32,
    /// Target scroll position; smooth interpolation converges toward this each frame.
    pub target_scroll_y: f32,
    /// Inertia velocity (pixels/frame) decayed each frame for momentum scrolling.
    pub scroll_velocity: f32,
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
                    <h1 style="color: #60a5fa; font-size: 32px; margin-bottom: 8px;">Secure Rust Browser Engine</h1>
                    <p style="color: #9ca3af; font-size: 15px; margin-bottom: 24px;">Modular Web Browser Engine Built from Scratch in Pure Rust | GPU Accelerated | Custom JS Runtime</p>
                    
                    <div class="card" style="background-color: #111827; border-radius: 12px; padding: 20px; border: 1px solid #1f2937; margin-bottom: 16px;">
                        <p style="color: #38bdf8; font-weight: bold; font-size: 16px; margin-bottom: 10px;">SPEED DIAL &amp; QUICK LINKS</p>
                        <p style="color: #e2e8f0; font-size: 14px; margin: 8px 0;">
                            <a href="https://google.com" style="color: #38bdf8; margin: 0 6px;">[ Google ]</a>
                            <a href="https://github.com" style="color: #38bdf8; margin: 0 6px;">[ GitHub ]</a>
                            <a href="https://youtube.com" style="color: #38bdf8; margin: 0 6px;">[ YouTube ]</a>
                            <a href="https://doc.rust-lang.org" style="color: #38bdf8; margin: 0 6px;">[ Rust Docs ]</a>
                            <a href="https://wikipedia.org" style="color: #38bdf8; margin: 0 6px;">[ Wikipedia ]</a>
                        </p>
                        <p style="color: #6b7280; font-size: 13px; margin-top: 10px;">Click any link or type a URL in the Omnibox above to navigate.</p>
                    </div>
                </div>
                "#
            ),
            scroll_y: 0.0,
            target_scroll_y: 0.0,
            scroll_velocity: 0.0,
            zoom_level: 1.0,
            history,
        }
    }

    pub fn navigate_to(&mut self, title: &str, url: &str, body: &str) {
        self.title = title.to_string();
        self.url = url.to_string();
        self.page_body = body.to_string();
        self.scroll_y = 0.0;
        self.target_scroll_y = 0.0;
        self.scroll_velocity = 0.0;
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

    // ── Smooth scrolling ────────────────────────────────────────────────────────

    /// Called by the `MouseWheel` event handler.  `delta_y` is the raw
    /// wheel delta in pixels (positive = scroll down, negative = scroll up).
    /// `max_scroll` is the document height minus viewport height so we can
    /// clamp the target immediately.
    pub fn apply_wheel_delta(&mut self, delta_y: f32, max_scroll: f32) {
        // Accumulate velocity from wheel input (scaled for comfortable speed)
        self.scroll_velocity += delta_y * 0.8;
        // Advance the target by the new velocity and clamp
        self.target_scroll_y =
            (self.target_scroll_y + self.scroll_velocity).clamp(0.0, max_scroll.max(0.0));
    }

    /// Called once per frame (on `RedrawRequested`).
    /// Exponentially decays velocity and lerps `scroll_y` toward `target_scroll_y`.
    /// Returns `true` if the frame needs a follow-up redraw (animation still running).
    pub fn tick_scroll(&mut self) -> bool {
        // Decay inertia — friction coefficient ~0.85 gives comfortable momentum
        self.scroll_velocity *= 0.85;
        if self.scroll_velocity.abs() < 0.05 {
            self.scroll_velocity = 0.0;
        }

        // Lerp toward target (easing factor 0.18 → snappy but smooth)
        let diff = self.target_scroll_y - self.scroll_y;
        if diff.abs() < 0.5 {
            self.scroll_y = self.target_scroll_y;
            return false; // settled — no further redraws needed
        }
        self.scroll_y += diff * 0.18;
        true // still animating
    }

    /// Hard-clamp both `scroll_y` and `target_scroll_y` to `[0, max_scroll]`.
    /// Called whenever viewport size changes or a new page loads.
    pub fn clamp_scroll(&mut self, max_scroll: f32) {
        let upper = max_scroll.max(0.0);
        self.target_scroll_y = self.target_scroll_y.clamp(0.0, upper);
        self.scroll_y = self.scroll_y.clamp(0.0, upper);
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
        let tab_title = if title.is_empty() {
            format!("Tab {}", id)
        } else {
            title.to_string()
        };
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
        let mut html = String::from(
            r#"<div class="tab-strip" style="background-color: #0b0f19; padding: 6px 16px 0 16px; border-bottom: 1px solid #1f2937;">"#,
        );
        for (i, tab) in self.tabs.iter().enumerate() {
            let is_active = i == self.active_index;
            let bg_color = if is_active { "#111827" } else { "#0b0f19" };
            let text_color = if is_active { "#38bdf8" } else { "#9ca3af" };
            let border = if is_active {
                "1px solid #374151"
            } else {
                "1px solid transparent"
            };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_clamp_prevents_negative() {
        let mut tab = Tab::new(1, "Test", "https://example.com");
        tab.target_scroll_y = -50.0;
        tab.scroll_y = -50.0;
        tab.clamp_scroll(500.0);
        assert_eq!(tab.scroll_y, 0.0);
        assert_eq!(tab.target_scroll_y, 0.0);
    }

    #[test]
    fn test_scroll_clamp_prevents_overflow() {
        let mut tab = Tab::new(1, "Test", "https://example.com");
        tab.target_scroll_y = 9999.0;
        tab.scroll_y = 9999.0;
        tab.clamp_scroll(800.0);
        assert_eq!(tab.scroll_y, 800.0);
        assert_eq!(tab.target_scroll_y, 800.0);
    }

    #[test]
    fn test_tick_scroll_lerps_toward_target() {
        let mut tab = Tab::new(1, "Test", "https://example.com");
        tab.scroll_y = 0.0;
        tab.target_scroll_y = 100.0;
        let still_animating = tab.tick_scroll();
        assert!(still_animating);
        // scroll_y should have moved toward 100 but not reached it yet
        assert!(tab.scroll_y > 0.0 && tab.scroll_y < 100.0);
    }

    #[test]
    fn test_tick_scroll_settles_when_close() {
        let mut tab = Tab::new(1, "Test", "https://example.com");
        tab.scroll_y = 99.6;
        tab.target_scroll_y = 100.0;
        let still_animating = tab.tick_scroll();
        assert!(!still_animating);
        assert_eq!(tab.scroll_y, 100.0);
    }

    #[test]
    fn test_apply_wheel_delta_clamps_to_max() {
        let mut tab = Tab::new(1, "Test", "https://example.com");
        tab.apply_wheel_delta(10_000.0, 500.0);
        assert!(tab.target_scroll_y <= 500.0);
    }

    #[test]
    fn test_velocity_decays_to_zero() {
        let mut tab = Tab::new(1, "Test", "https://example.com");
        tab.scroll_velocity = 100.0;
        for _ in 0..60 {
            tab.tick_scroll();
        }
        assert!(tab.scroll_velocity.abs() < 0.1);
    }

    #[test]
    fn test_navigate_to_resets_scroll() {
        let mut tab = Tab::new(1, "Test", "https://example.com");
        tab.scroll_y = 300.0;
        tab.target_scroll_y = 300.0;
        tab.scroll_velocity = 12.0;
        tab.navigate_to("New Page", "https://new.com", "<p>hello</p>");
        assert_eq!(tab.scroll_y, 0.0);
        assert_eq!(tab.target_scroll_y, 0.0);
        assert_eq!(tab.scroll_velocity, 0.0);
    }
}
