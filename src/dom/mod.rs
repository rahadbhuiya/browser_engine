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
        assert!(dom.nodes.iter().any(|n| matches!(&n.node_type, NodeType::Element(e) if e.tag == "input")));
        assert!(dom.nodes.iter().any(|n| matches!(&n.node_type, NodeType::Element(e) if e.tag == "button")));
    }
}
