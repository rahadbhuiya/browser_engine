#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavAction {
    Back,
    Forward,
    Reload,
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

/// Navigation Bar component providing visual Back, Forward, and Reload controls
/// alongside the URL omnibox and hit-test boundary resolution.
#[derive(Debug, Clone, Default)]
pub struct NavBar;

impl NavBar {
    pub fn new() -> Self {
        NavBar
    }

    /// Renders visual Back, Forward, and Reload controls alongside the URL omnibox into HTML.
    pub fn render_html(
        &self,
        url_text: &str,
        can_go_back: bool,
        can_go_forward: bool,
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

        format!(
            r#"
            <div class="navbar">
                <div class="nav-controls">
                    <span class="nav-btn nav-back" style="background-color: {}; color: {};">[ &lt; Back ]</span>
                    <span class="nav-btn nav-forward" style="background-color: {}; color: {};">[ Forward &gt; ]</span>
                    <span class="nav-btn nav-reload" style="background-color: #334155; color: #38bdf8;">[ &#x21bb; Reload ]</span>
                    <span class="nav-shortcuts" style="color: #64748b; font-size: 11px;">(Alt+Left / Alt+Right / F5)</span>
                </div>
                <p class="address-input">[ {} | ]</p>
                {}
            </div>
            "#,
            back_bg,
            back_fg,
            fwd_bg,
            fwd_fg,
            url_text,
            suggestions_html
        )
    }

    /// Hit-tests a mouse click against the navigation controls (Back, Forward, Reload).
    ///
    /// The navbar buttons row sits directly below the tab strip (y: 42.0 to 76.0 px):
    /// - Back button:    x: 12.0 .. 98.0
    /// - Forward button: x: 104.0 .. 212.0
    /// - Reload button:  x: 218.0 .. 316.0
    pub fn get_button_at_click(&self, x: f32, y: f32) -> Option<NavAction> {
        if y >= 42.0 && y <= 76.0 {
            if x >= 12.0 && x <= 98.0 {
                return Some(NavAction::Back);
            } else if x >= 104.0 && x <= 212.0 {
                return Some(NavAction::Forward);
            } else if x >= 218.0 && x <= 316.0 {
                return Some(NavAction::Reload);
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
        assert_eq!(navbar.get_button_at_click(50.0, 55.0), Some(NavAction::Back));
        // Click Forward button
        assert_eq!(navbar.get_button_at_click(150.0, 55.0), Some(NavAction::Forward));
        // Click Reload button
        assert_eq!(navbar.get_button_at_click(260.0, 55.0), Some(NavAction::Reload));
        // Click outside button Y range (e.g. inside tab strip or address bar)
        assert_eq!(navbar.get_button_at_click(50.0, 20.0), None);
        assert_eq!(navbar.get_button_at_click(50.0, 95.0), None);
        // Click in gap between buttons
        assert_eq!(navbar.get_button_at_click(100.0, 55.0), None);
    }

    #[test]
    fn test_navbar_render_html_contains_buttons_and_shortcuts() {
        let navbar = NavBar::new();
        let html_enabled = navbar.render_html("https://example.com", true, false, "");
        assert!(html_enabled.contains("[ &lt; Back ]"));
        assert!(html_enabled.contains("[ Forward &gt; ]"));
        assert!(html_enabled.contains("[ &#x21bb; Reload ]"));
        assert!(html_enabled.contains("Alt+Left"));
        assert!(html_enabled.contains("https://example.com"));
    }
}
