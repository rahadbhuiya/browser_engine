// Batch 3 — Layout geometry pass
//
// Implements simplified CSS 2.1 block-formatting-context layout: block boxes
// stack vertically, each taking the full width of its containing block
// unless an explicit width is set. This follows the same overall structure
// as the well-known "robinson" toy browser design (calculate width ->
// position -> lay out children -> calculate height), adapted to this
// engine's arena-based DOM/layout trees.
//
// APPROXIMATION (documented, not hidden): there is no real text shaping yet
// (no font metrics, no HarfBuzz) — text run height/width is estimated with a
// fixed average-character-width heuristic. Real glyph-accurate text layout
// is planned for Batch 4 once painting/fonts are in place.

use std::collections::HashMap;

use crate::style::ComputedStyle;

use super::box_model::Dimensions;
use super::tree::{BoxType, LayoutBox};

/// Rough text metrics stand-in until real font shaping exists.
const AVG_CHAR_WIDTH_PX: f32 = 8.0;
const LINE_HEIGHT_PX: f32 = 18.0;

pub fn layout_tree(
    root: &mut LayoutBox,
    viewport_width: f32,
    styles: &HashMap<usize, ComputedStyle>,
) {
    let containing_block = Dimensions {
        content: super::box_model::Rect {
            x: 0.0,
            y: 0.0,
            width: viewport_width,
            height: 0.0,
        },
        ..Default::default()
    };
    layout_box(root, containing_block, styles);
}

fn layout_box(
    layout_box_ref: &mut LayoutBox,
    containing_block: Dimensions,
    styles: &HashMap<usize, ComputedStyle>,
) {
    match layout_box_ref.box_type {
        BoxType::Block(node_idx) => layout_block(layout_box_ref, containing_block, styles, node_idx),
        BoxType::Anonymous => layout_anonymous_text(layout_box_ref, containing_block),
    }
}

fn style_for<'a>(
    styles: &'a HashMap<usize, ComputedStyle>,
    node_idx: usize,
) -> Option<&'a ComputedStyle> {
    styles.get(&node_idx)
}

/// Parses a CSS length value ("10px", "50%", "0") into pixels relative to
/// `containing`. Unknown units or garbage values fall back to 0.0 rather
/// than panicking — consistent with the rest of this engine's error handling.
fn parse_length(value: &str, containing: f32) -> f32 {
    let v = value.trim();
    if let Some(stripped) = v.strip_suffix("px") {
        stripped.trim().parse().unwrap_or(0.0)
    } else if let Some(stripped) = v.strip_suffix('%') {
        stripped
            .trim()
            .parse::<f32>()
            .map(|p| p / 100.0 * containing)
            .unwrap_or(0.0)
    } else {
        v.parse().unwrap_or(0.0)
    }
}

/// Reads a box-edge property (margin/padding/border-width) honoring both the
/// shorthand ("margin: 10px") and longhands ("margin-left: 5px"), with the
/// longhand overriding the shorthand when both are present.
fn edge_sizes(
    style: Option<&ComputedStyle>,
    prefix: &str,
    containing_width: f32,
) -> super::box_model::EdgeSizes {
    let mut edges = super::box_model::EdgeSizes::default();
    let Some(style) = style else { return edges };

    if let Some(shorthand) = style.get(prefix) {
        let v = parse_length(shorthand, containing_width);
        edges = super::box_model::EdgeSizes {
            top: v,
            right: v,
            bottom: v,
            left: v,
        };
    }
    if let Some(v) = style.get(&format!("{prefix}-top")) {
        edges.top = parse_length(v, containing_width);
    }
    if let Some(v) = style.get(&format!("{prefix}-right")) {
        edges.right = parse_length(v, containing_width);
    }
    if let Some(v) = style.get(&format!("{prefix}-bottom")) {
        edges.bottom = parse_length(v, containing_width);
    }
    if let Some(v) = style.get(&format!("{prefix}-left")) {
        edges.left = parse_length(v, containing_width);
    }
    edges
}

fn layout_block(
    lb: &mut LayoutBox,
    containing_block: Dimensions,
    styles: &HashMap<usize, ComputedStyle>,
    node_idx: usize,
) {
    let style = style_for(styles, node_idx);
    let containing_width = containing_block.content.width;

    lb.dimensions.margin = edge_sizes(style, "margin", containing_width);
    lb.dimensions.border = edge_sizes(style, "border-width", containing_width);
    lb.dimensions.padding = edge_sizes(style, "padding", containing_width);

    let explicit_width = style
        .and_then(|s| s.get("width"))
        .filter(|w| w.as_str() != "auto")
        .map(|w| parse_length(w, containing_width));

    let used_width = explicit_width.unwrap_or_else(|| {
        (containing_width
            - lb.dimensions.margin.left
            - lb.dimensions.margin.right
            - lb.dimensions.border.left
            - lb.dimensions.border.right
            - lb.dimensions.padding.left
            - lb.dimensions.padding.right)
            .max(0.0)
    });
    lb.dimensions.content.width = used_width;

    // position: x is fixed relative to containing block's content box;
    // y stacks below whatever has already been placed in the containing
    // block (tracked via containing_block.content.height as a running
    // cursor, following the classic block-layout technique).
    lb.dimensions.content.x = containing_block.content.x
        + lb.dimensions.margin.left
        + lb.dimensions.border.left
        + lb.dimensions.padding.left;
    lb.dimensions.content.y = containing_block.content.y
        + containing_block.content.height
        + lb.dimensions.margin.top
        + lb.dimensions.border.top
        + lb.dimensions.padding.top;

    // lay out children, stacking them vertically inside this box
    let mut cursor = lb.dimensions;
    cursor.content.height = 0.0;
    for child in &mut lb.children {
        layout_box(child, cursor, styles);
        cursor.content.height += child.dimensions.margin_box().height;
    }

    let explicit_height = style
        .and_then(|s| s.get("height"))
        .filter(|h| h.as_str() != "auto")
        .map(|h| parse_length(h, containing_width));

    lb.dimensions.content.height = explicit_height.unwrap_or(cursor.content.height);
}

fn layout_anonymous_text(lb: &mut LayoutBox, containing_block: Dimensions) {
    lb.dimensions.content.x = containing_block.content.x;
    lb.dimensions.content.y = containing_block.content.y + containing_block.content.height;
    lb.dimensions.content.width = containing_block.content.width;

    let char_count = lb.text_content.as_deref().unwrap_or("").chars().count() as f32;
    let chars_per_line = (containing_block.content.width / AVG_CHAR_WIDTH_PX).max(1.0);
    let lines = (char_count / chars_per_line).ceil().max(1.0);
    lb.dimensions.content.height = lines * LINE_HEIGHT_PX;
}