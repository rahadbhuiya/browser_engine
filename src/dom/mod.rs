pub mod builder;
pub mod node;

pub use builder::build_dom;
pub use node::{Dom, DomNode, ElementData, NodeType};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::Tokenizer;

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
}
