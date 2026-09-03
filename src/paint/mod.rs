use std::collections::HashMap;

use crate::layout::{LayoutBox, BoxType, Rect};
use crate::style::ComputedStyle;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b, a: 1.0 }
    }

    pub fn black() -> Self {
        Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
    }

    pub fn white() -> Self {
        Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }
    }

    pub fn transparent() -> Self {
        Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
    }
}

pub fn parse_color(val: &str) -> Option<Color> {
    let s = val.trim().to_ascii_lowercase();
    if s.starts_with('#') {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() == 4 {
            let r = chars[1].to_digit(16)? as f32 / 15.0;
            let g = chars[2].to_digit(16)? as f32 / 15.0;
            let b = chars[3].to_digit(16)? as f32 / 15.0;
            return Some(Color::new(r, g, b, 1.0));
        } else if chars.len() == 7 {
            let r = (chars[1].to_digit(16)? * 16 + chars[2].to_digit(16)?) as f32 / 255.0;
            let g = (chars[3].to_digit(16)? * 16 + chars[4].to_digit(16)?) as f32 / 255.0;
            let b = (chars[5].to_digit(16)? * 16 + chars[6].to_digit(16)?) as f32 / 255.0;
            return Some(Color::new(r, g, b, 1.0));
        } else if chars.len() == 9 {
            let r = (chars[1].to_digit(16)? * 16 + chars[2].to_digit(16)?) as f32 / 255.0;
            let g = (chars[3].to_digit(16)? * 16 + chars[4].to_digit(16)?) as f32 / 255.0;
            let b = (chars[5].to_digit(16)? * 16 + chars[6].to_digit(16)?) as f32 / 255.0;
            let a = (chars[7].to_digit(16)? * 16 + chars[8].to_digit(16)?) as f32 / 255.0;
            return Some(Color::new(r, g, b, a));
        }
    } else if s.starts_with("rgb(") && s.ends_with(')') {
        let content = &s[4..s.len() - 1];
        let parts: Vec<&str> = content.split(',').collect();
        if parts.len() == 3 {
            let r = parts[0].trim().parse::<f32>().ok()? / 255.0;
            let g = parts[1].trim().parse::<f32>().ok()? / 255.0;
            let b = parts[2].trim().parse::<f32>().ok()? / 255.0;
            return Some(Color::new(r, g, b, 1.0));
        }
    } else if s.starts_with("rgba(") && s.ends_with(')') {
        let content = &s[5..s.len() - 1];
        let parts: Vec<&str> = content.split(',').collect();
        if parts.len() == 4 {
            let r = parts[0].trim().parse::<f32>().ok()? / 255.0;
            let g = parts[1].trim().parse::<f32>().ok()? / 255.0;
            let b = parts[2].trim().parse::<f32>().ok()? / 255.0;
            let a = parts[3].trim().parse::<f32>().ok()?;
            return Some(Color::new(r, g, b, a.clamp(0.0, 1.0)));
        }
    } else {
        match s.as_str() {
            "red" => return Some(Color::new(1.0, 0.0, 0.0, 1.0)),
            "green" => return Some(Color::new(0.0, 0.5, 0.0, 1.0)),
            "blue" => return Some(Color::new(0.0, 0.0, 1.0, 1.0)),
            "black" => return Some(Color::black()),
            "white" => return Some(Color::white()),
            "transparent" => return Some(Color::transparent()),
            _ => {}
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaintCommand {
    DrawRect {
        rect: Rect,
        color: Color,
        border_radius: f32,
        is_fixed: bool,
    },
    DrawText {
        text: String,
        rect: Rect,
        color: Color,
        font_size: f32,
        is_fixed: bool,
    },
    DrawImage {
        rect: Rect,
        src: String,
        alt: String,
        is_fixed: bool,
    },
}

#[derive(Clone)]
struct LayeredCommand {
    cmd: PaintCommand,
    z_index: i32,
    order: usize,
}

pub fn build_display_list(
    layout_box: &LayoutBox,
    styles: &HashMap<usize, ComputedStyle>,
) -> Vec<PaintCommand> {
    let mut items = Vec::new();
    let mut order = 0;
    build_recursive(layout_box, styles, None, false, 0, &mut items, &mut order);
    items.sort_by(|a, b| a.z_index.cmp(&b.z_index).then(a.order.cmp(&b.order)));
    items.into_iter().map(|item| item.cmd).collect()
}

fn build_recursive(
    lb: &LayoutBox,
    styles: &HashMap<usize, ComputedStyle>,
    parent_style: Option<&ComputedStyle>,
    parent_is_fixed: bool,
    parent_z_index: i32,
    items: &mut Vec<LayeredCommand>,
    order: &mut usize,
) {
    match lb.box_type {
        BoxType::Block(node_idx) => {
            let style = styles.get(&node_idx);

            let is_fixed = style
                .and_then(|s| s.get("position"))
                .map(|p| p.trim().eq_ignore_ascii_case("fixed"))
                .unwrap_or(parent_is_fixed);

            let z_index = style
                .and_then(|s| s.get("z-index"))
                .and_then(|z| z.trim().parse::<i32>().ok())
                .unwrap_or(parent_z_index);

            let border_radius = style
                .and_then(|s| s.get("border-radius"))
                .map(|br| crate::layout::engine::parse_length(br, lb.dimensions.content.width))
                .unwrap_or(0.0);

            // 0. Draw Box Shadow (Elevation)
            if let Some(s) = style {
                if let Some(_shadow_str) = s.get("box-shadow") {
                    let shadow_color = parse_color("rgba(0, 0, 0, 0.35)").unwrap_or(Color::new(0.0, 0.0, 0.0, 0.35));
                    *order += 1;
                    items.push(LayeredCommand {
                        cmd: PaintCommand::DrawRect {
                            rect: Rect {
                                x: lb.dimensions.padding_box().x + 2.0,
                                y: lb.dimensions.padding_box().y + 4.0,
                                width: lb.dimensions.padding_box().width,
                                height: lb.dimensions.padding_box().height,
                            },
                            color: shadow_color,
                            border_radius: border_radius + 2.0,
                            is_fixed,
                        },
                        z_index,
                        order: *order,
                    });
                }
            }

            // 1. Draw Background
            if let Some(s) = style {
                if let Some(bg_color_str) = s.get("background-color") {
                    if let Some(color) = parse_color(bg_color_str) {
                        if color.a > 0.0 {
                            *order += 1;
                            items.push(LayeredCommand {
                                cmd: PaintCommand::DrawRect {
                                    rect: lb.dimensions.padding_box(),
                                    color,
                                    border_radius,
                                    is_fixed,
                                },
                                z_index,
                                order: *order,
                            });
                        }
                    }
                }
            }

            // 2. Draw Borders
            let border_width = lb.dimensions.border;
            if border_width.top > 0.0 || border_width.bottom > 0.0 || border_width.left > 0.0 || border_width.right > 0.0 {
                let border_color_str = style.and_then(|s| s.get("border-color"));
                let color = border_color_str
                    .and_then(|c| parse_color(c))
                    .unwrap_or(Color::black());

                let border_box = lb.dimensions.border_box();

                if border_width.top > 0.0 {
                    *order += 1;
                    items.push(LayeredCommand {
                        cmd: PaintCommand::DrawRect {
                            rect: Rect {
                                x: border_box.x,
                                y: border_box.y,
                                width: border_box.width,
                                height: border_width.top,
                            },
                            color,
                            border_radius: 0.0,
                            is_fixed,
                        },
                        z_index,
                        order: *order,
                    });
                }
                if border_width.bottom > 0.0 {
                    *order += 1;
                    items.push(LayeredCommand {
                        cmd: PaintCommand::DrawRect {
                            rect: Rect {
                                x: border_box.x,
                                y: border_box.y + border_box.height - border_width.bottom,
                                width: border_box.width,
                                height: border_width.bottom,
                            },
                            color,
                            border_radius: 0.0,
                            is_fixed,
                        },
                        z_index,
                        order: *order,
                    });
                }
                if border_width.left > 0.0 {
                    *order += 1;
                    items.push(LayeredCommand {
                        cmd: PaintCommand::DrawRect {
                            rect: Rect {
                                x: border_box.x,
                                y: border_box.y,
                                width: border_width.left,
                                height: border_box.height,
                            },
                            color,
                            border_radius: 0.0,
                            is_fixed,
                        },
                        z_index,
                        order: *order,
                    });
                }
                if border_width.right > 0.0 {
                    *order += 1;
                    items.push(LayeredCommand {
                        cmd: PaintCommand::DrawRect {
                            rect: Rect {
                                x: border_box.x + border_box.width - border_width.right,
                                y: border_box.y,
                                width: border_width.right,
                                height: border_box.height,
                            },
                            color,
                            border_radius: 0.0,
                            is_fixed,
                        },
                        z_index,
                        order: *order,
                    });
                }
            }

            for child in &lb.children {
                build_recursive(child, styles, style, is_fixed, z_index, items, order);
            }
        }
        BoxType::Anonymous => {
            if let Some(text) = &lb.text_content {
                let color = parent_style
                    .and_then(|s| s.get("color"))
                    .and_then(|c| parse_color(c))
                    .unwrap_or(Color::black());

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
                let lines = crate::layout::engine::wrap_text(text, lb.dimensions.content.width, char_width);

                for (idx, line) in lines.into_iter().enumerate() {
                    *order += 1;
                    items.push(LayeredCommand {
                        cmd: PaintCommand::DrawText {
                            text: line,
                            rect: Rect {
                                x: lb.dimensions.content.x,
                                y: lb.dimensions.content.y + idx as f32 * line_height,
                                width: lb.dimensions.content.width,
                                height: line_height,
                            },
                            color,
                            font_size,
                            is_fixed: parent_is_fixed,
                        },
                        z_index: parent_z_index,
                        order: *order,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_and_named_colors() {
        assert_eq!(parse_color("#ff0000"), Some(Color::new(1.0, 0.0, 0.0, 1.0)));
        assert_eq!(parse_color("#00f"), Some(Color::new(0.0, 0.0, 1.0, 1.0)));
        assert_eq!(parse_color("black"), Some(Color::black()));
        assert_eq!(parse_color("transparent"), Some(Color::transparent()));
    }

    #[test]
    fn parses_rgb_color_function() {
        assert_eq!(parse_color("rgb(255, 0, 0)"), Some(Color::new(1.0, 0.0, 0.0, 1.0)));
        assert_eq!(parse_color("rgba(0, 128, 255, 0.5)"), Some(Color::new(0.0, 128.0 / 255.0, 1.0, 0.5)));
    }

    #[test]
    fn test_draw_image_paint_command() {
        let cmd = PaintCommand::DrawImage {
            rect: Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 },
            src: "logo.png".to_string(),
            alt: "Logo".to_string(),
            is_fixed: false,
        };
        assert!(matches!(cmd, PaintCommand::DrawImage { .. }));
    }

    #[test]
    fn test_border_radius_display_list() {
        let box_node = LayoutBox::new(BoxType::Block(0));
        let mut styles = HashMap::new();
        let mut style = HashMap::new();
        style.insert("background-color".to_string(), "#1e293b".to_string());
        style.insert("border-radius".to_string(), "12px".to_string());
        styles.insert(0, style);

        let list = build_display_list(&box_node, &styles);
        assert!(!list.is_empty());
        if let PaintCommand::DrawRect { border_radius, .. } = &list[0] {
            assert_eq!(*border_radius, 12.0);
        } else {
            panic!("Expected DrawRect with border_radius");
        }
    }

    #[test]
    fn test_z_index_display_ordering() {
        let mut root = LayoutBox::new(BoxType::Block(0));
        let child1 = LayoutBox::new(BoxType::Block(1));
        let child2 = LayoutBox::new(BoxType::Block(2));
        root.children.push(child1);
        root.children.push(child2);

        let mut styles = HashMap::new();
        let mut style1 = HashMap::new();
        style1.insert("background-color".to_string(), "red".to_string());
        style1.insert("z-index".to_string(), "10".to_string());
        styles.insert(1, style1);

        let mut style2 = HashMap::new();
        style2.insert("background-color".to_string(), "blue".to_string());
        style2.insert("z-index".to_string(), "1".to_string());
        styles.insert(2, style2);

        let list = build_display_list(&root, &styles);
        // child 2 (z-index 1) must be rendered before child 1 (z-index 10)
        assert_eq!(list.len(), 2);
        if let (PaintCommand::DrawRect { color: c1, .. }, PaintCommand::DrawRect { color: c2, .. }) = (&list[0], &list[1]) {
            assert_eq!(*c1, Color::new(0.0, 0.0, 1.0, 1.0)); // blue
            assert_eq!(*c2, Color::new(1.0, 0.0, 0.0, 1.0)); // red
        }
    }
}
