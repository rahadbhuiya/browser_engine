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
    },
    DrawText {
        text: String,
        rect: Rect,
        color: Color,
        font_size: f32,
    },
    DrawImage {
        rect: Rect,
        src: String,
        alt: String,
    },
}

pub fn build_display_list(
    layout_box: &LayoutBox,
    styles: &HashMap<usize, ComputedStyle>,
) -> Vec<PaintCommand> {
    let mut list = Vec::new();
    build_recursive(layout_box, styles, None, &mut list);
    list
}

fn build_recursive(
    lb: &LayoutBox,
    styles: &HashMap<usize, ComputedStyle>,
    parent_style: Option<&ComputedStyle>,
    list: &mut Vec<PaintCommand>,
) {
    match lb.box_type {
        BoxType::Block(node_idx) => {
            let style = styles.get(&node_idx);
            
            // 1. Draw Background
            if let Some(s) = style {
                if let Some(bg_color_str) = s.get("background-color") {
                    if let Some(color) = parse_color(bg_color_str) {
                        if color.a > 0.0 {
                            list.push(PaintCommand::DrawRect {
                                rect: lb.dimensions.padding_box(),
                                color,
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

                // Top Border
                if border_width.top > 0.0 {
                    list.push(PaintCommand::DrawRect {
                        rect: Rect {
                            x: border_box.x,
                            y: border_box.y,
                            width: border_box.width,
                            height: border_width.top,
                        },
                        color,
                    });
                }
                // Bottom Border
                if border_width.bottom > 0.0 {
                    list.push(PaintCommand::DrawRect {
                        rect: Rect {
                            x: border_box.x,
                            y: border_box.y + border_box.height - border_width.bottom,
                            width: border_box.width,
                            height: border_width.bottom,
                        },
                        color,
                    });
                }
                // Left Border
                if border_width.left > 0.0 {
                    list.push(PaintCommand::DrawRect {
                        rect: Rect {
                            x: border_box.x,
                            y: border_box.y,
                            width: border_width.left,
                            height: border_box.height,
                        },
                        color,
                    });
                }
                // Right Border
                if border_width.right > 0.0 {
                    list.push(PaintCommand::DrawRect {
                        rect: Rect {
                            x: border_box.x + border_box.width - border_width.right,
                            y: border_box.y,
                            width: border_width.right,
                            height: border_box.height,
                        },
                        color,
                    });
                }
            }

            for child in &lb.children {
                build_recursive(child, styles, style, list);
            }
        }
        BoxType::Anonymous => {
            if let Some(text) = &lb.text_content {
                // Inherit text color and font size from parent block style
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
                    list.push(PaintCommand::DrawText {
                        text: line,
                        rect: Rect {
                            x: lb.dimensions.content.x,
                            y: lb.dimensions.content.y + idx as f32 * line_height,
                            width: lb.dimensions.content.width,
                            height: line_height,
                        },
                        color,
                        font_size,
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
    }

    #[test]
    fn test_draw_image_paint_command() {
        let cmd = PaintCommand::DrawImage {
            rect: Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 },
            src: "logo.png".to_string(),
            alt: "Logo".to_string(),
        };
        assert!(matches!(cmd, PaintCommand::DrawImage { .. }));
    }
}

