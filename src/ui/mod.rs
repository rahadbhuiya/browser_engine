#![allow(dead_code)]

pub mod address_bar;
pub mod bookmarks;
pub mod context_menu;
pub mod devtools;
pub mod navbar;
pub mod tab;

pub use address_bar::AddressBar;
pub use bookmarks::{Bookmark, BookmarkManager};
pub use context_menu::ContextMenu;
pub use devtools::{DevTools, DevToolsTab};
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
        assert!(html.contains("New Tab"));
        assert!(html.contains("Rust Documentation"));
    }

    #[test]
    fn test_per_tab_history() {
        let mut tab = Tab::new(1, "Google", "https://google.com");
        tab.navigate_to("Example", "https://example.com", "body");
        assert!(tab.history.can_go_back());
        assert_eq!(tab.history.go_back(), Some("https://google.com".to_string()));
    }

    #[test]
    fn test_bookmark_manager() {
        let mut b_mgr = BookmarkManager::new();
        assert_eq!(b_mgr.items.len(), 3);
        b_mgr.add("StackOverflow", "https://stackoverflow.com");
        assert_eq!(b_mgr.items.len(), 4);
        let html = b_mgr.render_bookmarks_bar_html();
        assert!(html.contains("StackOverflow"));
    }

    #[test]
    fn test_hit_testing_mouse_clicks() {
        let mgr = TabManager::new();
        assert_eq!(mgr.get_tab_at_click(50.0, 20.0), Some(0));

        let b_mgr = BookmarkManager::new();
        assert_eq!(b_mgr.get_url_at_click(100.0, 90.0), Some("https://google.com".to_string()));
    }

    #[test]
    fn test_devtools_dom_inspector() {
        let html = "<html><body><h1>Test Page</h1></body></html>";
        let dom = crate::dom::build_dom(crate::html::Tokenizer::new(html).tokenize());
        let dump = DevTools::inspect_dom(&dom);
        assert!(dump.contains("<h1 >"));
        assert!(dump.contains("Test Page"));
    }

    #[test]
    fn test_find_in_page_matching() {
        let tab = Tab::new(1, "Test", "https://example.com");
        let matches = tab.find_in_page("Engine");
        assert!(matches >= 1);
    }

    #[test]
    fn test_tab_page_zoom_controls() {
        let mut tab = Tab::new(1, "Test", "https://example.com");
        assert_eq!(tab.zoom_level, 1.0);
        tab.zoom_in();
        assert!((tab.zoom_level - 1.10).abs() < 0.001);
        tab.zoom_out();
        assert!((tab.zoom_level - 1.00).abs() < 0.001);
        tab.reset_zoom();
        assert_eq!(tab.zoom_level, 1.0);
    }

    #[test]
    fn test_tab_closing_and_new_tab() {
        let mut mgr = TabManager::new();
        mgr.new_tab("Second Tab", "https://github.com");
        assert_eq!(mgr.tabs.len(), 2);

        // Click close on tab 1
        assert_eq!(mgr.get_close_click(140.0 * 2.0 - 5.0, 20.0), Some(1));
        mgr.close_tab(1);
        assert_eq!(mgr.tabs.len(), 1);

        // Click new tab [+]
        assert!(mgr.is_new_tab_click(146.0 * 1.0 + 10.0, 20.0));
    }

    #[test]
    fn test_address_bar_search_suggestions() {
        let bar = AddressBar::new("you");
        let suggestions = bar.generate_suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("youtube.com")));

        let html = bar.render_suggestions_html();
        assert!(html.contains("youtube.com"));
        assert!(html.contains("suggestions"));
    }

    #[test]
    fn test_context_menu_interaction() {
        let mut menu = ContextMenu::new();
        assert!(!menu.is_visible);
        menu.show(100.0, 100.0);
        assert!(menu.is_visible);

        let item = menu.get_item_at_click(120.0, 110.0);
        assert_eq!(item, Some("Back"));

        let item2 = menu.get_item_at_click(120.0, 138.0);
        assert_eq!(item2, Some("Forward"));

        let html = menu.render_html();
        assert!(html.contains("context-menu"));
        assert!(html.contains("Reload"));

        menu.hide();
        assert!(!menu.is_visible);
        assert_eq!(menu.get_item_at_click(120.0, 110.0), None);
    }

    #[test]
    fn test_devtools_panel_state() {
        let mut dt = DevTools::new();
        assert!(!dt.is_open);
        dt.toggle();
        assert!(dt.is_open);
        dt.select_tab(DevToolsTab::Console);
        assert_eq!(dt.active_tab, DevToolsTab::Console);
        dt.log("Test log entry");
        assert!(dt.console_logs.contains(&"Test log entry".to_string()));
        dt.record_network_request("POST", "https://api.example.com", 201);
        assert!(dt.network_requests[0].contains("Status: 201"));
        dt.close();
        assert!(!dt.is_open);
    }

    #[test]
    fn test_devtools_html_rendering() {
        let mut dt = DevTools::new();
        let html_input = "<html><body><main><p>Hello DevTools</p></main></body></html>";
        let dom = crate::dom::build_dom(crate::html::Tokenizer::new(html_input).tokenize());

        // Closed devtools renders empty string
        assert_eq!(dt.render_html(&dom, "https://example.com"), "");

        // Opened devtools renders panel
        dt.open();
        let panel_html = dt.render_html(&dom, "https://example.com");
        assert!(panel_html.contains("devtools-panel"));
        assert!(panel_html.contains("DEVTOOLS DOCK PANEL"));
        assert!(panel_html.contains("Elements"));
        assert!(panel_html.contains("Console"));
        assert!(panel_html.contains("Hello DevTools"));

        // Console tab rendering
        dt.select_tab(DevToolsTab::Console);
        dt.log("Kernel graphics context online");
        let console_html = dt.render_html(&dom, "https://example.com");
        assert!(console_html.contains("Kernel graphics context online"));

        // Network tab rendering
        dt.select_tab(DevToolsTab::Network);
        dt.record_network_request("GET", "https://example.com/bundle.js", 200);
        let net_html = dt.render_html(&dom, "https://example.com");
        assert!(net_html.contains("bundle.js"));
        assert!(net_html.contains("200 OK"));
    }
}




