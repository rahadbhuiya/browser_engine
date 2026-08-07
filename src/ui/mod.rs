#![allow(dead_code)]

pub mod address_bar;
pub mod navbar;
pub mod tab;

pub use address_bar::AddressBar;
pub use navbar::NavBarHistory;
pub use tab::{Tab, TabManager};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_bar_input() {
        let mut bar = AddressBar::new("http://example.com");
        bar.insert_char('a');
        assert_eq!(bar.url_text, "http://example.coma");
        bar.backspace();
        assert_eq!(bar.url_text, "http://example.com");
    }

    #[test]
    fn test_tab_manager() {
        let mut mgr = TabManager::new();
        assert_eq!(mgr.tabs.len(), 1);
        let id2 = mgr.new_tab("Google", "https://google.com");
        assert_eq!(mgr.tabs.len(), 2);
        assert_eq!(mgr.active_tab().id, id2);
        mgr.close_tab(1);
        assert_eq!(mgr.tabs.len(), 1);
    }

    #[test]
    fn test_navbar_history() {
        let mut history = NavBarHistory::default();
        history.navigate_to("http://example.com");
        history.navigate_to("http://example.com/page2");

        assert!(history.can_go_back());
        assert_eq!(history.go_back(), Some("http://example.com".to_string()));

        assert!(history.can_go_forward());
        assert_eq!(history.go_forward(), Some("http://example.com/page2".to_string()));
    }

    #[test]
    fn test_tab_strip_rendering() {
        let mut mgr = TabManager::new();
        mgr.new_tab("Rust Documentation", "https://doc.rust-lang.org");
        let html = mgr.render_tab_strip_html();
        assert!(html.contains("Google"));
        assert!(html.contains("Rust Documentation"));
    }
}

