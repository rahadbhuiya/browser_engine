// Batch 3 — Layout tree construction
//
// Walks the styled DOM and produces a tree of LayoutBoxes: one Block box per
// block-level element, and an "anonymous" box wrapping each run of inline
// content (text, inline elements) found between block-level children —
// the same technique real engines use so inline content always has a box
// to be laid out in, even when it has no element of its own.
//
// SIMPLIFICATION: inline elements (e.g. <span>, <a>) are not yet given their
// own inline boxes — their text is flattened into the surrounding run. Proper
// inline-level boxes (needed for e.g. styling just the <a> text) are future
// work once real text shaping (Batch 4+) is in place.

use std::collections::HashMap;

use crate::dom::{Dom, NodeType};
use crate::style::ComputedStyle;

use super::box_model::Dimensions;

const INLINE_TAGS: &[&str] = &[
    "span", "a", "b", "i", "em", "strong", "code", "small", "label", "abbr",
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxType {
    Block(usize), // index into Dom::nodes
    Anonymous,
}

#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub dimensions: Dimensions,
    pub box_type: BoxType,
    pub children: Vec<LayoutBox>,
    /// Only set on Anonymous boxes: the flattened inline text they contain.
    pub text_content: Option<String>,
}

impl LayoutBox {
    fn new_block(node_idx: usize) -> Self {
        LayoutBox {
            dimensions: Dimensions::default(),
            box_type: BoxType::Block(node_idx),
            children: vec![],
            text_content: None,
        }
    }
    fn new_anonymous_text(text: String) -> Self {
        LayoutBox {
            dimensions: Dimensions::default(),
            box_type: BoxType::Anonymous,
            children: vec![],
            text_content: Some(text),
        }
    }
}

fn is_block_level(dom: &Dom, node_idx: usize, styles: &HashMap<usize, ComputedStyle>) -> bool {
    if let Some(style) = styles.get(&node_idx) {
        if let Some(display) = style.get("display") {
            return display != "inline" && display != "none";
        }
    }
    match dom.element_at(node_idx) {
        Some(el) => !INLINE_TAGS.contains(&el.tag.as_str()),
        None => true,
    }
}

fn is_display_none(node_idx: usize, styles: &HashMap<usize, ComputedStyle>) -> bool {
    styles
        .get(&node_idx)
        .and_then(|s| s.get("display"))
        .map(|d| d == "none")
        .unwrap_or(false)
}

/// Recursively appends this node's rendered text (and its inline
/// descendants' text) onto `buf`, used to flatten inline content into a
/// single anonymous run.
fn collect_inline_text(dom: &Dom, node_idx: usize, buf: &mut String) {
    match &dom.nodes[node_idx].node_type {
        NodeType::Text(t) => buf.push_str(t),
        NodeType::Element(_) => {
            for &child in &dom.nodes[node_idx].children {
                collect_inline_text(dom, child, buf);
            }
        }
        _ => {}
    }
}

pub fn build_layout_tree(
    dom: &Dom,
    styles: &HashMap<usize, ComputedStyle>,
    node_idx: usize,
) -> Option<LayoutBox> {
    let is_container = matches!(
        dom.nodes[node_idx].node_type,
        NodeType::Element(_) | NodeType::Document
    );
    if !is_container {
        return None;
    }
    if is_display_none(node_idx, styles) {
        return None;
    }

    let mut layout_box = LayoutBox::new_block(node_idx);
    let mut pending_inline = String::new();

    for &child_idx in &dom.nodes[node_idx].children {
        match &dom.nodes[child_idx].node_type {
            NodeType::Text(t) => {
                if !t.trim().is_empty() {
                    pending_inline.push_str(t);
                }
            }
            NodeType::Element(_) => {
                if is_display_none(child_idx, styles) {
                    continue;
                }
                if is_block_level(dom, child_idx, styles) {
                    flush_inline_run(&mut pending_inline, &mut layout_box);
                    if let Some(child_box) = build_layout_tree(dom, styles, child_idx) {
                        layout_box.children.push(child_box);
                    }
                } else {
                    collect_inline_text(dom, child_idx, &mut pending_inline);
                }
            }
            _ => {}
        }
    }
    flush_inline_run(&mut pending_inline, &mut layout_box);

    Some(layout_box)
}

fn flush_inline_run(pending_inline: &mut String, layout_box: &mut LayoutBox) {
    if !pending_inline.trim().is_empty() {
        layout_box
            .children
            .push(LayoutBox::new_anonymous_text(std::mem::take(pending_inline)));
    } else {
        pending_inline.clear();
    }
}