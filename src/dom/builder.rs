// Batch 1 (DOM builder) — turns the Batch 1 tokenizer's Token stream into a
// Dom tree. This is a simplified tree builder: it does not implement the
// full WHATWG "insertion mode" state machine (no automatic </p> closing
// before block elements, no foster-parenting for misplaced <table> content,
// etc). Those refinements are noted as future work at the bottom of this
// file. What it does guarantee, matching the tokenizer's security posture:
// it never panics, even on deeply malformed/mismatched tag soup.

use crate::html::Token;
use super::node::{Dom, DomNode, ElementData, NodeType};

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "param", "source", "track", "wbr",
];

pub fn build_dom(tokens: Vec<Token>) -> Dom {
    let mut nodes = vec![DomNode {
        node_type: NodeType::Document,
        parent: None,
        children: vec![],
    }];
    let root = 0;
    let mut open_stack: Vec<usize> = vec![root];

    let push_node = |nodes: &mut Vec<DomNode>, parent: usize, node_type: NodeType| -> usize {
        let idx = nodes.len();
        nodes.push(DomNode {
            node_type,
            parent: Some(parent),
            children: vec![],
        });
        nodes[parent].children.push(idx);
        idx
    };

    for token in tokens {
        match token {
            Token::Doctype { .. } => {
                // doctype doesn't become a DOM node in this simplified model
            }
            Token::StartTag {
                name,
                attributes,
                self_closing,
            } => {
                let parent = *open_stack.last().unwrap_or(&root);
                let element = ElementData::new(name.clone(), attributes);
                let idx = push_node(&mut nodes, parent, NodeType::Element(element));
                let is_void = self_closing || VOID_ELEMENTS.contains(&name.as_str());
                if !is_void {
                    open_stack.push(idx);
                }
            }
            Token::EndTag { name } => {
                // Find the nearest matching open element on the stack and pop
                // back to (and including) it. If no match exists (mismatched
                // or garbage closing tag), ignore it rather than panicking or
                // corrupting the tree. Index 0 of open_stack is always the
                // Document root, which never matches an element tag, so a
                // found position is always >= 1 and safe to truncate to.
                if let Some(pos) = open_stack
                    .iter()
                    .rposition(|&idx| matches_tag(&nodes[idx], &name))
                {
                    open_stack.truncate(pos);
                }
            }
            Token::Text(text) => {
                let parent = *open_stack.last().unwrap_or(&root);
                push_node(&mut nodes, parent, NodeType::Text(text));
            }
            Token::Comment(text) => {
                let parent = *open_stack.last().unwrap_or(&root);
                push_node(&mut nodes, parent, NodeType::Comment(text));
            }
            Token::Eof => break,
        }
    }

    Dom { nodes, root }
}

fn matches_tag(node: &DomNode, name: &str) -> bool {
    matches!(&node.node_type, NodeType::Element(e) if e.tag == name)
}

// Future work (not yet implemented — see ROADMAP.md):
// - Full HTML5 insertion-mode state machine (implied end tags, e.g. an open
//   <p> auto-closes when a new block element starts)
// - Table foster-parenting rules
// - <template>/<script>/<style> raw-text content handling