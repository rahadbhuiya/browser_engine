// Batch 3 & Production Pillar 1 — Layout geometry pass
//
// Implements modern CSS layout: Block formatting context, Flexbox (row,
// column, justify-content, align-items), margin: auto horizontal centering,
// and real line-wrapping text flow.

use std::collections::HashMap;

use crate::style::ComputedStyle;

use super::box_model::Dimensions;
use super::tree::{BoxType, LayoutBox};

/// Rough text metrics stand-in until real font shaping exists.
pub const AVG_CHAR_WIDTH_PX: f32 = 8.0;
pub const LINE_HEIGHT_PX: f32 = 18.0;

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
/// `containing`. Unknown units or garbage values fall back to 0.0.
pub fn parse_length(value: &str, containing: f32) -> f32 {
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

/// Reads a box-edge property (margin/padding/border-width) honoring both shorthand and longhands.
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

/// Breaks text by whitespace into lines wrapped to `max_width`.
pub fn wrap_text(text: &str, max_width: f32, char_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let max_chars = ((max_width / char_width).floor() as usize).max(1);

    for raw_line in text.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let words = trimmed.split_whitespace();
        let mut current_line = String::new();

        for word in words {
            if current_line.is_empty() {
                current_line.push_str(word);
            } else if current_line.chars().count() + 1 + word.chars().count() <= max_chars {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
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

    // Check for margin: auto centering
    let margin_left_auto = style
        .and_then(|s| s.get("margin-left"))
        .map(|m| m.as_str() == "auto")
        .unwrap_or(false)
        || style
            .and_then(|s| s.get("margin"))
            .map(|m| m.contains("auto"))
            .unwrap_or(false);
    let margin_right_auto = style
        .and_then(|s| s.get("margin-right"))
        .map(|m| m.as_str() == "auto")
        .unwrap_or(false)
        || style
            .and_then(|s| s.get("margin"))
            .map(|m| m.contains("auto"))
            .unwrap_or(false);

    if margin_left_auto && margin_right_auto && explicit_width.is_some() {
        let free_space = (containing_width
            - used_width
            - lb.dimensions.border.left
            - lb.dimensions.border.right
            - lb.dimensions.padding.left
            - lb.dimensions.padding.right)
            .max(0.0);
        let half = free_space / 2.0;
        lb.dimensions.margin.left = half;
        lb.dimensions.margin.right = half;
    }

    // position: x is fixed relative to containing block's content box;
    // y stacks below whatever has already been placed in containing block
    lb.dimensions.content.x = containing_block.content.x
        + lb.dimensions.margin.left
        + lb.dimensions.border.left
        + lb.dimensions.padding.left;
    lb.dimensions.content.y = containing_block.content.y
        + containing_block.content.height
        + lb.dimensions.margin.top
        + lb.dimensions.border.top
        + lb.dimensions.padding.top;

    let display = style
        .and_then(|s| s.get("display"))
        .map(|d| d.trim().to_ascii_lowercase());
    let is_flex = display.as_deref() == Some("flex") || display.as_deref() == Some("inline-flex");

    let children_height = if is_flex {
        layout_flex_children(lb, style, styles)
    } else {
        layout_block_children(lb, styles)
    };

    let explicit_height = style
        .and_then(|s| s.get("height"))
        .filter(|h| h.as_str() != "auto")
        .map(|h| parse_length(h, containing_width));

    lb.dimensions.content.height = explicit_height.unwrap_or(children_height);
}

fn layout_block_children(lb: &mut LayoutBox, styles: &HashMap<usize, ComputedStyle>) -> f32 {
    let mut cursor = lb.dimensions;
    cursor.content.height = 0.0;
    for child in &mut lb.children {
        layout_box(child, cursor, styles);
        cursor.content.height += child.dimensions.margin_box().height;
    }
    cursor.content.height
}

fn layout_flex_children(
    lb: &mut LayoutBox,
    style: Option<&ComputedStyle>,
    styles: &HashMap<usize, ComputedStyle>,
) -> f32 {
    let flex_direction = style
        .and_then(|s| s.get("flex-direction"))
        .map(|d| d.trim().to_ascii_lowercase())
        .unwrap_or_else(|| "row".to_string());
    let is_row = flex_direction != "column";

    let justify_content = style
        .and_then(|s| s.get("justify-content"))
        .map(|d| d.trim().to_ascii_lowercase())
        .unwrap_or_else(|| "flex-start".to_string());

    let align_items = style
        .and_then(|s| s.get("align-items"))
        .map(|d| d.trim().to_ascii_lowercase())
        .unwrap_or_else(|| "stretch".to_string());

    if lb.children.is_empty() {
        return 0.0;
    }

    // First layout pass to measure children natural sizes
    let probe_cursor = Dimensions {
        content: super::box_model::Rect {
            x: lb.dimensions.content.x,
            y: lb.dimensions.content.y,
            width: lb.dimensions.content.width,
            height: 0.0,
        },
        ..Default::default()
    };

    for child in &mut lb.children {
        layout_box(child, probe_cursor, styles);
    }

    if is_row {
        let total_child_width: f32 = lb.children.iter().map(|c| c.dimensions.margin_box().width).sum();
        let free_space = (lb.dimensions.content.width - total_child_width).max(0.0);

        let (start_x, gap) = match justify_content.as_str() {
            "center" => (free_space / 2.0, 0.0),
            "flex-end" => (free_space, 0.0),
            "space-between" => {
                if lb.children.len() > 1 {
                    (0.0, free_space / (lb.children.len() - 1) as f32)
                } else {
                    (0.0, 0.0)
                }
            }
            "space-around" => {
                let space = free_space / lb.children.len() as f32;
                (space / 2.0, space)
            }
            _ => (0.0, 0.0), // "flex-start" default
        };

        let mut max_height: f32 = 0.0;
        for child in &lb.children {
            max_height = max_height.max(child.dimensions.margin_box().height);
        }

        let mut current_x = lb.dimensions.content.x + start_x;
        for child in &mut lb.children {
            child.dimensions.content.x = current_x + child.dimensions.margin.left;

            // Cross-axis alignment (align-items)
            let child_mb_height = child.dimensions.margin_box().height;
            let align_y_offset = match align_items.as_str() {
                "center" => (max_height - child_mb_height) / 2.0,
                "flex-end" => max_height - child_mb_height,
                _ => 0.0, // "flex-start", "stretch"
            };

            child.dimensions.content.y = lb.dimensions.content.y
                + align_y_offset
                + child.dimensions.margin.top;

            current_x += child.dimensions.margin_box().width + gap;
        }

        max_height
    } else {
        // flex-direction: column
        let mut cursor_y = lb.dimensions.content.y;
        for child in &mut lb.children {
            child.dimensions.content.y = cursor_y + child.dimensions.margin.top;
            cursor_y += child.dimensions.margin_box().height;
        }
        cursor_y - lb.dimensions.content.y
    }
}

fn layout_anonymous_text(lb: &mut LayoutBox, containing_block: Dimensions) {
    lb.dimensions.content.x = containing_block.content.x;
    lb.dimensions.content.y = containing_block.content.y + containing_block.content.height;
    lb.dimensions.content.width = containing_block.content.width;

    let text = lb.text_content.as_deref().unwrap_or("");
    let lines = wrap_text(text, containing_block.content.width, AVG_CHAR_WIDTH_PX);
    lb.dimensions.content.height = (lines.len() as f32 * LINE_HEIGHT_PX).max(LINE_HEIGHT_PX);
}