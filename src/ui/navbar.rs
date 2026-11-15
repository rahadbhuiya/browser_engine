#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavAction {
    Back,
    Forward,
    Reload,
    BookmarkToggle,
    PrivacyShieldToggle,
}

#[derive(Debug, Clone, Default)]
pub struct NavBarHistory {
    pub back_stack: Vec<String>,
    pub forward_stack: Vec<String>,
    pub current_url: Option<String>,
}

impl NavBarHistory {
    pub fn navigate_to(&mut self, url: &str) {
        if let Some(curr) = self.current_url.take() {
            self.back_stack.push(curr);
        }
        self.forward_stack.clear();
        self.current_url = Some(url.to_string());
    }

    pub fn can_go_back(&self) -> bool {
        !self.back_stack.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.forward_stack.is_empty()
    }

    pub fn go_back(&mut self) -> Option<String> {
        if let Some(prev) = self.back_stack.pop() {
            if let Some(curr) = self.current_url.take() {
                self.forward_stack.push(curr);
            }
            self.current_url = Some(prev.clone());
            Some(prev)
        } else {
            None
        }
    }

    pub fn go_forward(&mut self) -> Option<String> {
        if let Some(next) = self.forward_stack.pop() {
            if let Some(curr) = self.current_url.take() {
                self.back_stack.push(curr);
            }
            self.current_url = Some(next.clone());
            Some(next)
        } else {
            None
        }
    }
}

/// Navigation Bar component providing visual Back, Forward, Reload, Bookmark,
/// and Privacy Shield controls alongside the URL omnibox and hit-test resolution.
#[derive(Debug, Clone, Default)]
pub struct NavBar;

impl NavBar {
    pub fn new() -> Self {
        NavBar
    }

    /// Renders visual Back, Forward, Reload, Bookmark, and Privacy Shield controls alongside the URL omnibox into HTML.
    pub fn render_html(
        &self,
        url_text: &str,
        can_go_back: bool,
        can_go_forward: bool,
        is_bookmarked: bool,
        blocked_trackers: usize,
        suggestions_html: &str,
    ) -> String {
        let (back_bg, back_fg) = if can_go_back {
            ("#334155", "#38bdf8")
        } else {
            ("#1e293b", "#64748b")
        };

        let (fwd_bg, fwd_fg) = if can_go_forward {
            ("#334155", "#38bdf8")
        } else {
            ("#1e293b", "#64748b")
        };

        let (star_icon, star_color, star_text) = if is_bookmarked {
            ("★", "#facc15", "Bookmarked")
        } else {
            ("☆", "#94a3b8", "Bookmark")
        };

        let shield_color = if blocked_trackers > 0 { "#f43f5e" } else { "#10b981" };

        format!(
            r#"
            <div class="navbar">
                <div class="nav-controls">
                    <span class="nav-btn nav-back" style="background-color: {}; color: {};">[ &lt; Back ]</span>
                    <span class="nav-btn nav-forward" style="background-color: {}; color: {};">[ Forward &gt; ]</span>
                    <span class="nav-btn nav-reload" style="background-color: #334155; color: #38bdf8;">[ &#x21bb; Reload ]</span>
                    <span class="nav-btn nav-star" style="background-color: #1e293b; color: {}; border: 1px solid #475569;">[ {} {} ]</span>
                    <span class="nav-btn nav-shield" style="background-color: #0f172a; color: {}; border: 1px solid {}; font-weight: bold;">[ 🛡️ Shield: {} blocked ]</span>
                    <span class="nav-shortcuts" style="color: #64748b; font-size: 11px;">(Ctrl+T, Ctrl+W, Ctrl+Tab, Ctrl+P)</span>
                </div>
                <p class="address-input">[ {} | ]</p>
                {}
            </div>
            "#,
            back_bg,
            back_fg,
            fwd_bg,
            fwd_fg,
            star_color,
            star_icon,
            star_text,
            shield_color,
            shield_color,
            blocked_trackers,
            url_text,
            suggestions_html
        )
    }

    /// Hit-tests a mouse click against the navigation controls (Back, Forward, Reload, Bookmark, Shield).
    ///
    /// Row sits directly below the tab strip (y: 42.0 to 76.0 px):
    /// - Back button:        x: 12.0 .. 98.0
    /// - Forward button:     x: 104.0 .. 212.0
    /// - Reload button:      x: 218.0 .. 316.0
    /// - Bookmark toggle:    x: 322.0 .. 442.0
    /// - Privacy Shield:     x: 448.0 .. 600.0
    pub fn get_button_at_click(&self, x: f32, y: f32) -> Option<NavAction> {
        if y >= 42.0 && y <= 76.0 {
            if x >= 12.0 && x <= 98.0 {
                return Some(NavAction::Back);
            } else if x >= 104.0 && x <= 212.0 {
                return Some(NavAction::Forward);
            } else if x >= 218.0 && x <= 316.0 {
                return Some(NavAction::Reload);
            } else if x >= 322.0 && x <= 442.0 {
                return Some(NavAction::BookmarkToggle);
            } else if x >= 448.0 && x <= 600.0 {
                return Some(NavAction::PrivacyShieldToggle);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navbar_history_stacks() {
        let mut history = NavBarHistory::default();
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());

        history.navigate_to("https://first.com");
        assert!(!history.can_go_back());
        assert_eq!(history.current_url.as_deref(), Some("https://first.com"));

        history.navigate_to("https://second.com");
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());
        assert_eq!(history.current_url.as_deref(), Some("https://second.com"));

        let prev = history.go_back();
        assert_eq!(prev.as_deref(), Some("https://first.com"));
        assert!(!history.can_go_back());
        assert!(history.can_go_forward());

        let next = history.go_forward();
        assert_eq!(next.as_deref(), Some("https://second.com"));
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());
    }

    #[test]
    fn test_navbar_button_hit_testing() {
        let navbar = NavBar::new();
        // Click Back button
        assert_eq!(
            navbar.get_button_at_click(50.0, 55.0),
            Some(NavAction::Back)
        );
        // Click Forward button
        assert_eq!(
            navbar.get_button_at_click(150.0, 55.0),
            Some(NavAction::Forward)
        );
        // Click Reload button
        assert_eq!(
            navbar.get_button_at_click(260.0, 55.0),
            Some(NavAction::Reload)
        );
        // Click Bookmark button
        assert_eq!(
            navbar.get_button_at_click(360.0, 55.0),
            Some(NavAction::BookmarkToggle)
        );
        // Click Privacy Shield button
        assert_eq!(
            navbar.get_button_at_click(500.0, 55.0),
            Some(NavAction::PrivacyShieldToggle)
        );
        // Click outside button Y range
        assert_eq!(navbar.get_button_at_click(50.0, 20.0), None);
        assert_eq!(navbar.get_button_at_click(50.0, 95.0), None);
    }

    #[test]
    fn test_navbar_render_html_contains_buttons_and_shortcuts() {
        let navbar = NavBar::new();
        let html_enabled =
            navbar.render_html("https://example.com", true, false, true, 2, "");
        assert!(html_enabled.contains("[ &lt; Back ]"));
        assert!(html_enabled.contains("[ Forward &gt; ]"));
        assert!(html_enabled.contains("[ &#x21bb; Reload ]"));
        assert!(html_enabled.contains("Bookmarked"));
        assert!(html_enabled.contains("Shield: 2 blocked"));
        assert!(html_enabled.contains("Ctrl+T, Ctrl+W, Ctrl+Tab"));
    }
}
