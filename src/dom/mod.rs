pub mod builder;
pub mod events;
pub mod form;
pub mod node;

pub use builder::build_dom;
pub use events::{DomEvent, EventListener, EventPhase, EventRegistry};
pub use form::{
    build_form_submission, collect_form_fields, encode_multipart, encode_url_encoded,
    FormControlState, FormEncoding, FormManager, FormMethod, FormSubmissionPayload, InputType,
};
pub use node::{Dom, DomNode, ElementData, NodeType};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::Tokenizer;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_input_and_button_parsing() {
        let html = r#"<form><input type="text" value="search"><button>Submit</button></form>"#;
        let tokens = Tokenizer::new(html).tokenize();
        let dom = build_dom(tokens);
        assert!(dom
            .nodes
            .iter()
            .any(|n| matches!(&n.node_type, NodeType::Element(e) if e.tag == "input")));
        assert!(dom
            .nodes
            .iter()
            .any(|n| matches!(&n.node_type, NodeType::Element(e) if e.tag == "button")));
    }

    #[test]
    fn test_dom_query_by_id_tag_class() {
        let html = r#"<main id="app"><div class="container active"><p class="text">Hello</p><span class="text">World</span></div></main>"#;
        let tokens = Tokenizer::new(html).tokenize();
        let dom = build_dom(tokens);

        let app_idx = dom.get_element_by_id("app").expect("app element exists");
        let app_elem = dom.element_at(app_idx).expect("is element");
        assert_eq!(app_elem.tag, "main");
        assert!(app_elem.has_attribute("id"));
        assert_eq!(app_elem.get_attribute("id"), Some("app"));

        let p_nodes = dom.get_elements_by_tag_name("p");
        assert_eq!(p_nodes.len(), 1);
        assert_eq!(dom.text_content(p_nodes[0]), "Hello");

        let text_nodes = dom.get_elements_by_class_name("text");
        assert_eq!(text_nodes.len(), 2);
    }

    #[test]
    fn test_dom_traversal_and_text_content() {
        let html = r#"<section id="s1"><h1 class="title">Header</h1><p>Paragraph <span>bold</span> text</p></section>"#;
        let tokens = Tokenizer::new(html).tokenize();
        let dom = build_dom(tokens);

        let s1 = dom.get_element_by_id("s1").unwrap();
        let children = dom.children(s1);
        assert_eq!(children.len(), 2);

        let h1_idx = children[0];
        let p_idx = children[1];

        assert_eq!(dom.first_child(s1), Some(h1_idx));
        assert_eq!(dom.last_child(s1), Some(p_idx));
        assert_eq!(dom.next_sibling(h1_idx), Some(p_idx));
        assert_eq!(dom.preceding_sibling(p_idx), Some(h1_idx));
        assert_eq!(dom.parent(h1_idx), Some(s1));

        let text = dom.text_content(s1);
        assert!(text.contains("Header"));
        assert!(text.contains("Paragraph bold text"));
    }

    #[test]
    fn test_dom_query_selector() {
        let html = r#"<div id="container"><div class="card"><p class="desc">First Card</p></div><div class="card active"><p class="desc">Second Card</p></div></div>"#;
        let tokens = Tokenizer::new(html).tokenize();
        let dom = build_dom(tokens);

        let first_card = dom.query_selector("div.card").expect("matches first card");
        assert_eq!(dom.element_at(first_card).unwrap().tag, "div");

        let active_card = dom
            .query_selector("div.card.active")
            .expect("matches active card");
        assert!(dom.element_at(active_card).unwrap().has_class("active"));

        let all_cards = dom.query_selector_all("div.card");
        assert_eq!(all_cards.len(), 2);

        let desc_all = dom.query_selector_all("p.desc");
        assert_eq!(desc_all.len(), 2);
        assert_eq!(dom.text_content(desc_all[0]), "First Card");
        assert_eq!(dom.text_content(desc_all[1]), "Second Card");
    }

    #[test]
    fn test_dom_query_selector_pseudo_classes() {
        let html = r#"<ul id="list"><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul><button disabled>Off</button><button id="btn" data-hover>Hover Me</button>"#;
        let tokens = Tokenizer::new(html).tokenize();
        let mut dom = build_dom(tokens);

        let first = dom.query_selector("li:first-child").expect("first-child matches");
        assert_eq!(dom.text_content(first), "Item 1");

        let last = dom.query_selector("li:last-child").expect("last-child matches");
        assert_eq!(dom.text_content(last), "Item 3");

        let disabled = dom.query_selector("button:disabled").expect("disabled matches");
        assert_eq!(dom.text_content(disabled), "Off");

        let btn_idx = dom.get_element_by_id("btn").unwrap();
        dom.set_hovered(btn_idx, true);
        let hovered = dom.query_selector("button:hover").expect("hover matches");
        assert_eq!(hovered, btn_idx);
    }

    #[test]
    fn test_w3c_event_flow_capture_target_bubble() {
        let html = r#"<div id="parent"><button id="child">Click Me</button></div>"#;
        let tokens = Tokenizer::new(html).tokenize();
        let mut dom = build_dom(tokens);

        let parent_idx = dom.get_element_by_id("parent").unwrap();
        let child_idx = dom.get_element_by_id("child").unwrap();

        let mut registry = EventRegistry::new();
        let sequence = Arc::new(std::sync::Mutex::new(Vec::new()));

        // Capture listener on parent
        let seq1 = Arc::clone(&sequence);
        registry.add_event_listener(parent_idx, "click", true, move |_ev, _dom| {
            seq1.lock().unwrap().push("parent_capture");
        });

        // Bubble listener on parent
        let seq2 = Arc::clone(&sequence);
        registry.add_event_listener(parent_idx, "click", false, move |_ev, _dom| {
            seq2.lock().unwrap().push("parent_bubble");
        });

        // Target listener on child
        let seq3 = Arc::clone(&sequence);
        registry.add_event_listener(child_idx, "click", false, move |_ev, _dom| {
            seq3.lock().unwrap().push("child_target");
        });

        let mut click_event = DomEvent::new("click", child_idx, true, true);
        let ok = registry.dispatch_event(&mut dom, &mut click_event);
        assert!(ok);

        let logged = sequence.lock().unwrap().clone();
        assert_eq!(logged, vec!["parent_capture", "child_target", "parent_bubble"]);
    }

    #[test]
    fn test_event_prevent_default_and_stop_propagation() {
        let html = r#"<div id="p"><button id="c">Action</button></div>"#;
        let tokens = Tokenizer::new(html).tokenize();
        let mut dom = build_dom(tokens);

        let parent_idx = dom.get_element_by_id("p").unwrap();
        let child_idx = dom.get_element_by_id("c").unwrap();

        let mut registry = EventRegistry::new();
        let parent_called = Arc::new(AtomicUsize::new(0));

        let pc = Arc::clone(&parent_called);
        registry.add_event_listener(parent_idx, "submit", false, move |_ev, _dom| {
            pc.fetch_add(1, Ordering::SeqCst);
        });

        registry.add_event_listener(child_idx, "submit", false, move |ev, _dom| {
            ev.prevent_default();
            ev.stop_propagation();
        });

        let mut submit_event = DomEvent::new("submit", child_idx, true, true);
        let not_cancelled = registry.dispatch_event(&mut dom, &mut submit_event);

        assert!(!not_cancelled, "Default prevented");
        assert!(submit_event.is_default_prevented());
        assert_eq!(parent_called.load(Ordering::SeqCst), 0, "Propagation stopped before parent");
    }

    #[test]
    fn test_form_controls_keyboard_input_and_submission() {
        let html = r#"<form id="login" action="/api/login" method="POST"><input type="text" name="user" value="admin"><input type="password" name="pass" value="1234"><input type="checkbox" name="remember" checked><textarea name="bio">developer</textarea></form>"#;
        let tokens = Tokenizer::new(html).tokenize();
        let dom = build_dom(tokens);

        let mut form_mgr = FormManager::new();
        form_mgr.scan_dom(&dom);

        let user_idx = dom.nodes.iter().position(|n| {
            matches!(&n.node_type, NodeType::Element(e) if e.get_attribute("name") == Some("user"))
        }).unwrap();

        // Focus and type into input
        form_mgr.set_focus(Some(user_idx));
        assert_eq!(form_mgr.focused_control(), Some(user_idx));
        form_mgr.handle_char_input('_');
        form_mgr.handle_char_input('1');

        let c = form_mgr.get_control(user_idx).unwrap();
        assert_eq!(c.value, "admin_1");

        // Backspace
        form_mgr.handle_backspace();
        let c = form_mgr.get_control(user_idx).unwrap();
        assert_eq!(c.value, "admin_");

        // Submit form (UrlEncoded)
        let form_idx = dom.get_element_by_id("login").unwrap();
        let payload = build_form_submission(&dom, form_idx, &form_mgr, "https://example.com/").unwrap();

        assert_eq!(payload.target_url, "https://example.com/api/login");
        assert_eq!(payload.method, FormMethod::Post);
        assert_eq!(payload.content_type.as_deref(), Some("application/x-www-form-urlencoded"));

        let body_str = String::from_utf8(payload.body).unwrap();
        assert!(body_str.contains("user=admin_"));
        assert!(body_str.contains("pass=1234"));
        assert!(body_str.contains("remember=on"));
        assert!(body_str.contains("bio=developer"));
    }

    #[test]
    fn test_form_get_submission_url_encoding() {
        let html = r#"<form id="search" action="search.php" method="GET"><input type="text" name="q" value="rust browser engine"><input type="number" name="page" value="2"></form>"#;
        let tokens = Tokenizer::new(html).tokenize();
        let dom = build_dom(tokens);

        let mut form_mgr = FormManager::new();
        form_mgr.scan_dom(&dom);

        let form_idx = dom.get_element_by_id("search").unwrap();
        let payload = build_form_submission(&dom, form_idx, &form_mgr, "https://example.com/app/").unwrap();

        assert_eq!(payload.method, FormMethod::Get);
        assert_eq!(payload.target_url, "https://example.com/app/search.php?q=rust+browser+engine&page=2");
        assert!(payload.body.is_empty());
    }

    #[test]
    fn test_form_multipart_encoding() {
        let html = r#"<form id="upload" action="/upload" method="POST" enctype="multipart/form-data"><input type="text" name="title" value="report"></form>"#;
        let tokens = Tokenizer::new(html).tokenize();
        let dom = build_dom(tokens);

        let mut form_mgr = FormManager::new();
        form_mgr.scan_dom(&dom);

        let form_idx = dom.get_element_by_id("upload").unwrap();
        let payload = build_form_submission(&dom, form_idx, &form_mgr, "https://example.com/").unwrap();

        assert_eq!(payload.method, FormMethod::Post);
        let ct = payload.content_type.unwrap();
        assert!(ct.starts_with("multipart/form-data; boundary="));

        let body_str = String::from_utf8(payload.body).unwrap();
        assert!(body_str.contains("Content-Disposition: form-data; name=\"title\""));
        assert!(body_str.contains("report"));
    }
}
