//! Layout geometry pass: Block formatting, CSS Grid, Flexbox, and text flow.

use std::collections::HashMap;

use crate::style::ComputedStyle;

use super::box_model::Dimensions;
use super::tree::{BoxType, LayoutBox};

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
        BoxType::Block(node_idx) => {
            layout_block(layout_box_ref, containing_block, styles, node_idx)
        }
        BoxType::Anonymous => layout_anonymous_text(layout_box_ref, containing_block, None),
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
        let mut current_len = 0;

        for word in words {
            let word_len = word.chars().count();
            if current_line.is_empty() {
                current_line.push_str(word);
                current_len = word_len;
            } else if current_len + 1 + word_len <= max_chars {
                current_line.push(' ');
                current_line.push_str(word);
                current_len += 1 + word_len;
            } else {
                lines.push(current_line);
                current_line = word.to_string();
                current_len = word_len;
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
        .map(|w| parse_length(w, containing_width))
        .or_else(|| lb.image_data.as_ref().and_then(|img| img.width));

    let max_width = style
        .and_then(|s| s.get("max-width"))
        .filter(|w| w.as_str() != "none")
        .map(|w| parse_length(w, containing_width));

    let mut used_width = explicit_width.unwrap_or_else(|| {
        if lb.image_data.is_some() {
            300.0f32.min(containing_width).max(20.0)
        } else {
            (containing_width
                - lb.dimensions.margin.left
                - lb.dimensions.margin.right
                - lb.dimensions.border.left
                - lb.dimensions.border.right
                - lb.dimensions.padding.left
                - lb.dimensions.padding.right)
                .max(0.0)
        }
    });

    if let Some(mw) = max_width {
        if mw > 0.0 && used_width > mw {
            used_width = mw;
        }
    }
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

    if margin_left_auto && margin_right_auto && (explicit_width.is_some() || max_width.is_some()) {
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

    // Normal flow placement relative to containing block
    lb.dimensions.content.x = containing_block.content.x
        + lb.dimensions.margin.left
        + lb.dimensions.border.left
        + lb.dimensions.padding.left;
    lb.dimensions.content.y = containing_block.content.y
        + containing_block.content.height
        + lb.dimensions.margin.top
        + lb.dimensions.border.top
        + lb.dimensions.padding.top;

    let position_mode = style
        .and_then(|s| s.get("position"))
        .map(|p| p.trim().to_ascii_lowercase())
        .unwrap_or_else(|| "static".to_string());

    let top_offset = style
        .and_then(|s| s.get("top"))
        .map(|v| parse_length(v, containing_block.content.height));
    let left_offset = style
        .and_then(|s| s.get("left"))
        .map(|v| parse_length(v, containing_width));
    let right_offset = style
        .and_then(|s| s.get("right"))
        .map(|v| parse_length(v, containing_width));

    match position_mode.as_str() {
        "fixed" => {
            let x = left_offset.unwrap_or_else(|| {
                if let Some(r) = right_offset {
                    (containing_width - used_width - r).max(0.0)
                } else {
                    0.0
                }
            });
            let y = top_offset.unwrap_or(0.0);
            lb.dimensions.content.x = x + lb.dimensions.margin.left;
            lb.dimensions.content.y = y + lb.dimensions.margin.top;
        }
        "absolute" => {
            let x = left_offset.unwrap_or_else(|| {
                if let Some(r) = right_offset {
                    (containing_width - used_width - r).max(0.0)
                } else {
                    0.0
                }
            });
            let y = top_offset.unwrap_or(0.0);
            lb.dimensions.content.x = containing_block.content.x + x + lb.dimensions.margin.left;
            lb.dimensions.content.y = containing_block.content.y + y + lb.dimensions.margin.top;
        }
        "relative" => {
            lb.dimensions.content.x += left_offset.unwrap_or(0.0);
            lb.dimensions.content.y += top_offset.unwrap_or(0.0);
        }
        _ => {}
    }

    let display = style
        .and_then(|s| s.get("display"))
        .map(|d| d.trim().to_ascii_lowercase());
    let is_flex = display.as_deref() == Some("flex") || display.as_deref() == Some("inline-flex");
    let is_grid = display.as_deref() == Some("grid") || display.as_deref() == Some("inline-grid");

    let children_height = if is_grid {
        layout_grid_children(lb, style, styles)
    } else if is_flex {
        layout_flex_children(lb, style, styles)
    } else {
        layout_block_children(lb, style, styles)
    };

    let explicit_height = style
        .and_then(|s| s.get("height"))
        .filter(|h| h.as_str() != "auto")
        .map(|h| parse_length(h, containing_width))
        .or_else(|| lb.image_data.as_ref().and_then(|img| img.height));

    let fallback_height = if lb.image_data.is_some() {
        (lb.dimensions.content.width * 0.75).min(400.0).max(30.0)
    } else {
        children_height
    };

    lb.dimensions.content.height = explicit_height.unwrap_or(fallback_height);
}

fn layout_block_children(
    lb: &mut LayoutBox,
    parent_style: Option<&ComputedStyle>,
    styles: &HashMap<usize, ComputedStyle>,
) -> f32 {
    let mut cursor = lb.dimensions;
    cursor.content.height = 0.0;
    for child in &mut lb.children {
        match child.box_type {
            BoxType::Block(idx) => layout_block(child, cursor, styles, idx),
            BoxType::Anonymous => layout_anonymous_text(child, cursor, parent_style),
        }
        let is_out_of_flow = match child.box_type {
            BoxType::Block(idx) => styles
                .get(&idx)
                .and_then(|s| s.get("position"))
                .map(|p| {
                    let s = p.trim().to_ascii_lowercase();
                    s == "absolute" || s == "fixed"
                })
                .unwrap_or(false),
            _ => false,
        };
        if !is_out_of_flow {
            cursor.content.height += child.dimensions.margin_box().height;
        }
    }
    cursor.content.height
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum GridTrack {
    Px(f32),
    Percent(f32),
    Fr(f32),
    Auto,
}

fn expand_repeat_tracks(input: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;
    while let Some(start) = remaining.find("repeat(") {
        result.push_str(&remaining[..start]);
        let after = &remaining[start + 7..];
        if let Some(end) = after.find(')') {
            let inner = &after[..end];
            if let Some((count_str, track_str)) = inner.split_once(',') {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    let track = track_str.trim();
                    for i in 0..count {
                        if i > 0 {
                            result.push(' ');
                        }
                        result.push_str(track);
                    }
                } else {
                    result.push_str(inner);
                }
            }
            remaining = &after[end + 1..];
        } else {
            result.push_str("repeat(");
            remaining = after;
        }
    }
    result.push_str(remaining);
    result
}

fn parse_grid_track_list(value: &str, _containing: f32) -> Vec<GridTrack> {
    let expanded = expand_repeat_tracks(value);
    let mut tracks = Vec::new();
    for token in expanded.split_whitespace() {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(stripped) = t.strip_suffix("fr") {
            let fr = stripped.trim().parse::<f32>().unwrap_or(1.0).max(0.0);
            tracks.push(GridTrack::Fr(fr));
        } else if let Some(stripped) = t.strip_suffix("px") {
            let px = stripped.trim().parse::<f32>().unwrap_or(0.0).max(0.0);
            tracks.push(GridTrack::Px(px));
        } else if let Some(stripped) = t.strip_suffix('%') {
            let pct = stripped.trim().parse::<f32>().unwrap_or(0.0).max(0.0);
            tracks.push(GridTrack::Percent(pct));
        } else if t.eq_ignore_ascii_case("auto") {
            tracks.push(GridTrack::Auto);
        } else if let Ok(val) = t.parse::<f32>() {
            tracks.push(GridTrack::Px(val));
        }
    }
    tracks
}

fn parse_gaps(style: Option<&ComputedStyle>, containing: f32) -> (f32, f32) {
    let mut row_gap = 0.0;
    let mut col_gap = 0.0;
    if let Some(style) = style {
        if let Some(gap) = style.get("gap") {
            let parts: Vec<&str> = gap.split_whitespace().collect();
            if parts.len() == 1 {
                let v = parse_length(parts[0], containing);
                row_gap = v;
                col_gap = v;
            } else if parts.len() >= 2 {
                row_gap = parse_length(parts[0], containing);
                col_gap = parse_length(parts[1], containing);
            }
        }
        if let Some(rg) = style.get("row-gap") {
            row_gap = parse_length(rg, containing);
        }
        if let Some(cg) = style.get("column-gap") {
            col_gap = parse_length(cg, containing);
        }
    }
    (row_gap, col_gap)
}

fn layout_grid_children(
    lb: &mut LayoutBox,
    style: Option<&ComputedStyle>,
    styles: &HashMap<usize, ComputedStyle>,
) -> f32 {
    if lb.children.is_empty() {
        return 0.0;
    }

    let container_width = lb.dimensions.content.width;
    let (row_gap, col_gap) = parse_gaps(style, container_width);

    let col_tracks_spec = style
        .and_then(|s| s.get("grid-template-columns"))
        .map(|s| parse_grid_track_list(s, container_width))
        .unwrap_or_default();

    let col_tracks = if col_tracks_spec.is_empty() {
        vec![GridTrack::Fr(1.0)]
    } else {
        col_tracks_spec
    };

    let num_cols = col_tracks.len().max(1);
    let total_col_gaps = if num_cols > 1 {
        (num_cols - 1) as f32 * col_gap
    } else {
        0.0
    };

    let mut fixed_w = 0.0;
    let mut total_fr = 0.0;
    for track in &col_tracks {
        match track {
            GridTrack::Px(px) => fixed_w += px,
            GridTrack::Percent(pct) => fixed_w += pct / 100.0 * container_width,
            GridTrack::Fr(fr) => total_fr += fr,
            GridTrack::Auto => fixed_w += 100.0f32.min(container_width / num_cols as f32),
        }
    }

    let remaining_w = (container_width - total_col_gaps - fixed_w).max(0.0);
    let fr_unit_w = if total_fr > 0.0 {
        remaining_w / total_fr
    } else {
        0.0
    };

    let col_widths: Vec<f32> = col_tracks
        .iter()
        .map(|track| match track {
            GridTrack::Px(px) => *px,
            GridTrack::Percent(pct) => pct / 100.0 * container_width,
            GridTrack::Fr(fr) => fr * fr_unit_w,
            GridTrack::Auto => 100.0f32.min(container_width / num_cols as f32),
        })
        .collect();

    let num_children = lb.children.len();
    let num_rows = (num_children + num_cols - 1) / num_cols;
    let mut row_heights = vec![0.0f32; num_rows];

    for (idx, child) in lb.children.iter_mut().enumerate() {
        let col = idx % num_cols;
        let row = idx / num_cols;

        let col_w = col_widths[col];
        let probe = Dimensions {
            content: super::box_model::Rect {
                x: 0.0,
                y: 0.0,
                width: col_w,
                height: 0.0,
            },
            ..Default::default()
        };
        layout_box(child, probe, styles);
        let h = child.dimensions.margin_box().height;
        if h > row_heights[row] {
            row_heights[row] = h;
        }
    }

    for (idx, child) in lb.children.iter_mut().enumerate() {
        let col = idx % num_cols;
        let row = idx / num_cols;

        let mut cell_x = lb.dimensions.content.x;
        for c in 0..col {
            cell_x += col_widths[c] + col_gap;
        }

        let mut cell_y = lb.dimensions.content.y;
        for r in 0..row {
            cell_y += row_heights[r] + row_gap;
        }

        child.dimensions.content.x = cell_x + child.dimensions.margin.left;
        child.dimensions.content.y = cell_y + child.dimensions.margin.top;
    }

    let total_row_gaps = if num_rows > 1 {
        (num_rows - 1) as f32 * row_gap
    } else {
        0.0
    };
    row_heights.iter().sum::<f32>() + total_row_gaps
}

fn child_prop(
    child: &LayoutBox,
    styles: &HashMap<usize, ComputedStyle>,
    prop: &str,
) -> Option<String> {
    match child.box_type {
        BoxType::Block(idx) => styles.get(&idx).and_then(|s| s.get(prop)).cloned(),
        BoxType::Anonymous => None,
    }
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

    let flex_wrap = style
        .and_then(|s| s.get("flex-wrap"))
        .map(|w| w.trim().to_ascii_lowercase())
        .unwrap_or_else(|| "nowrap".to_string());
    let is_wrap = flex_wrap == "wrap" || flex_wrap == "wrap-reverse";

    let justify_content = style
        .and_then(|s| s.get("justify-content"))
        .map(|d| d.trim().to_ascii_lowercase())
        .unwrap_or_else(|| "flex-start".to_string());

    let align_items = style
        .and_then(|s| s.get("align-items"))
        .map(|d| d.trim().to_ascii_lowercase())
        .unwrap_or_else(|| "stretch".to_string());

    let (row_gap, col_gap) = parse_gaps(style, lb.dimensions.content.width);

    if lb.children.is_empty() {
        return 0.0;
    }

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
        let mut lines: Vec<Vec<usize>> = Vec::new();
        let mut current_line = Vec::new();
        let mut current_w = 0.0;

        for (i, child) in lb.children.iter().enumerate() {
            let child_w = child.dimensions.margin_box().width;
            let needed = if current_line.is_empty() {
                child_w
            } else {
                current_w + col_gap + child_w
            };

            if is_wrap && !current_line.is_empty() && needed > lb.dimensions.content.width {
                lines.push(std::mem::take(&mut current_line));
                current_line.push(i);
                current_w = child_w;
            } else {
                current_line.push(i);
                current_w = needed;
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if flex_wrap == "wrap-reverse" {
            lines.reverse();
        }

        let container_w = lb.dimensions.content.width;
        let mut current_y = lb.dimensions.content.y;
        let mut total_height = 0.0;

        for (line_idx, line_indices) in lines.iter().enumerate() {
            if line_idx > 0 {
                current_y += row_gap;
            }

            let line_items_w: f32 = line_indices
                .iter()
                .map(|&idx| lb.children[idx].dimensions.margin_box().width)
                .sum();
            let line_gaps = if line_indices.len() > 1 {
                (line_indices.len() - 1) as f32 * col_gap
            } else {
                0.0
            };
            let mut free_space = (container_w - line_items_w - line_gaps).max(0.0);

            let mut total_grow = 0.0f32;
            for &idx in line_indices {
                let grow = child_prop(&lb.children[idx], styles, "flex-grow")
                    .and_then(|g| g.trim().parse::<f32>().ok())
                    .unwrap_or(0.0)
                    .max(0.0);
                total_grow += grow;
            }

            if total_grow > 0.0 && free_space > 0.0 {
                for &idx in line_indices {
                    let grow = child_prop(&lb.children[idx], styles, "flex-grow")
                        .and_then(|g| g.trim().parse::<f32>().ok())
                        .unwrap_or(0.0)
                        .max(0.0);
                    if grow > 0.0 {
                        let extra = free_space * (grow / total_grow);
                        lb.children[idx].dimensions.content.width += extra;
                    }
                }
                free_space = 0.0;
            }

            let (start_x, extra_gap) = match justify_content.as_str() {
                "center" => (free_space / 2.0, 0.0),
                "flex-end" => (free_space, 0.0),
                "space-between" => {
                    if line_indices.len() > 1 {
                        (0.0, free_space / (line_indices.len() - 1) as f32)
                    } else {
                        (0.0, 0.0)
                    }
                }
                "space-around" => {
                    let sp = free_space / line_indices.len() as f32;
                    (sp / 2.0, sp)
                }
                _ => (0.0, 0.0),
            };

            let mut max_line_h: f32 = 0.0;
            for &idx in line_indices {
                let h = lb.children[idx].dimensions.margin_box().height;
                if h > max_line_h {
                    max_line_h = h;
                }
            }

            let mut current_x = lb.dimensions.content.x + start_x;
            for &idx in line_indices {
                let child = &mut lb.children[idx];
                child.dimensions.content.x = current_x + child.dimensions.margin.left;

                let child_h = child.dimensions.margin_box().height;
                let align =
                    child_prop(child, styles, "align-self").unwrap_or_else(|| align_items.clone());
                let align_offset = match align.as_str() {
                    "center" => (max_line_h - child_h) / 2.0,
                    "flex-end" => max_line_h - child_h,
                    _ => 0.0,
                };

                child.dimensions.content.y = current_y + align_offset + child.dimensions.margin.top;
                current_x += child.dimensions.margin_box().width + col_gap + extra_gap;
            }

            current_y += max_line_h;
            total_height = current_y - lb.dimensions.content.y;
        }

        total_height
    } else {
        let mut cursor_y = lb.dimensions.content.y;
        for child in &mut lb.children {
            let align =
                child_prop(child, styles, "align-self").unwrap_or_else(|| align_items.clone());
            let align_offset = match align.as_str() {
                "center" => {
                    (lb.dimensions.content.width - child.dimensions.margin_box().width) / 2.0
                }
                "flex-end" => lb.dimensions.content.width - child.dimensions.margin_box().width,
                _ => 0.0,
            };

            child.dimensions.content.x =
                lb.dimensions.content.x + align_offset + child.dimensions.margin.left;
            child.dimensions.content.y = cursor_y + child.dimensions.margin.top;
            cursor_y += child.dimensions.margin_box().height + row_gap;
        }
        if !lb.children.is_empty() {
            cursor_y -= row_gap;
        }
        cursor_y - lb.dimensions.content.y
    }
}

fn layout_anonymous_text(
    lb: &mut LayoutBox,
    containing_block: Dimensions,
    parent_style: Option<&ComputedStyle>,
) {
    lb.dimensions.content.x = containing_block.content.x;
    lb.dimensions.content.y = containing_block.content.y + containing_block.content.height;
    lb.dimensions.content.width = containing_block.content.width;

    let font_size = parent_style
        .and_then(|s| s.get("font-size"))
        .and_then(|fs| {
            let fs_trimmed = fs.trim();
            if let Some(stripped) = fs_trimmed.strip_suffix("px") {
                stripped.trim().parse::<f32>().ok()
            } else {
                fs_trimmed.parse::<f32>().ok()
            }
        })
        .unwrap_or(16.0);

    let char_width = 8.0 * (font_size / 16.0);
    let line_height = 18.0 * (font_size / 16.0);

    let text = lb.text_content.as_deref().unwrap_or("");
    let lines = wrap_text(text, containing_block.content.width, char_width);
    lb.dimensions.content.height = (lines.len() as f32 * line_height).max(line_height);
}
