// Batch 3 — Box model geometry
// Mirrors CSS 2.1's box model: content box, surrounded by padding, border,
// then margin. All units are CSS pixels (f32) for simplicity in this batch.

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn expanded_by(&self, edge: EdgeSizes) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Dimensions {
    pub content: Rect,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

impl Dimensions {
    pub fn padding_box(&self) -> Rect {
        self.content.expanded_by(self.padding)
    }
    pub fn border_box(&self) -> Rect {
        self.padding_box().expanded_by(self.border)
    }
    pub fn margin_box(&self) -> Rect {
        self.border_box().expanded_by(self.margin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains_point() {
        let rect = Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        };
        assert!(rect.contains(10.0, 20.0));
        assert!(rect.contains(110.0, 70.0));
        assert!(rect.contains(50.0, 40.0));
        assert!(!rect.contains(9.9, 20.0));
        assert!(!rect.contains(10.0, 19.9));
        assert!(!rect.contains(110.1, 50.0));
        assert!(!rect.contains(50.0, 70.1));
    }
}
