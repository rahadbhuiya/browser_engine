pub mod box_model;
pub mod engine;
pub mod tree;

pub use box_model::{Dimensions, EdgeSizes, Rect};
pub use engine::layout_tree;
pub use tree::{build_layout_tree, BoxType, LayoutBox};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_wrap_text_algorithm() {
        let sentence = "The quick brown fox jumps over the lazy dog";
        let lines = engine::wrap_text(sentence, 100.0, 10.0);
        assert!(lines.len() >= 4);
        assert_eq!(lines[0], "The quick");
    }

    #[test]
    fn test_flexbox_row_layout() {
        let mut parent = LayoutBox::new(BoxType::Block(0));
        let child1 = LayoutBox::new(BoxType::Block(1));
        let child2 = LayoutBox::new(BoxType::Block(2));
        parent.children.push(child1);
        parent.children.push(child2);

        let mut styles = HashMap::new();
        let mut parent_style = HashMap::new();
        parent_style.insert("display".to_string(), "flex".to_string());
        parent_style.insert("flex-direction".to_string(), "row".to_string());
        parent_style.insert("width".to_string(), "300px".to_string());
        styles.insert(0, parent_style);

        let mut child1_style = HashMap::new();
        child1_style.insert("width".to_string(), "100px".to_string());
        child1_style.insert("height".to_string(), "50px".to_string());
        styles.insert(1, child1_style);

        let mut child2_style = HashMap::new();
        child2_style.insert("width".to_string(), "100px".to_string());
        child2_style.insert("height".to_string(), "50px".to_string());
        styles.insert(2, child2_style);

        layout_tree(&mut parent, 800.0, &styles);

        let c1_x = parent.children[0].dimensions.content.x;
        let c2_x = parent.children[1].dimensions.content.x;
        assert!(c2_x > c1_x);
        assert_eq!(c2_x - c1_x, 100.0);
    }

    #[test]
    fn test_margin_auto_horizontal_centering() {
        let mut box_node = LayoutBox::new(BoxType::Block(0));
        let mut styles = HashMap::new();
        let mut style = HashMap::new();
        style.insert("width".to_string(), "400px".to_string());
        style.insert("margin".to_string(), "0 auto".to_string());
        styles.insert(0, style);

        layout_tree(&mut box_node, 1000.0, &styles);

        assert_eq!(box_node.dimensions.margin.left, 300.0);
        assert_eq!(box_node.dimensions.margin.right, 300.0);
        assert_eq!(box_node.dimensions.content.x, 300.0);
    }

    #[test]
    fn test_position_absolute_and_fixed() {
        let mut parent = LayoutBox::new(BoxType::Block(0));
        let child_abs = LayoutBox::new(BoxType::Block(1));
        parent.children.push(child_abs);

        let mut styles = HashMap::new();
        let mut parent_style = HashMap::new();
        parent_style.insert("width".to_string(), "500px".to_string());
        parent_style.insert("height".to_string(), "500px".to_string());
        styles.insert(0, parent_style);

        let mut abs_style = HashMap::new();
        abs_style.insert("position".to_string(), "absolute".to_string());
        abs_style.insert("left".to_string(), "50px".to_string());
        abs_style.insert("top".to_string(), "30px".to_string());
        abs_style.insert("width".to_string(), "100px".to_string());
        styles.insert(1, abs_style);

        layout_tree(&mut parent, 800.0, &styles);

        let abs_box = &parent.children[0];
        assert_eq!(abs_box.dimensions.content.x, 50.0);
        assert_eq!(abs_box.dimensions.content.y, 30.0);
    }

    #[test]
    fn test_layout_box_hit_test() {
        let mut root = LayoutBox::new(BoxType::Block(0));
        root.dimensions.content = Rect {
            x: 0.0,
            y: 0.0,
            width: 500.0,
            height: 400.0,
        };

        let mut child = LayoutBox::new(BoxType::Block(1));
        child.dimensions.content = Rect {
            x: 50.0,
            y: 50.0,
            width: 200.0,
            height: 100.0,
        };

        let mut link_box = LayoutBox::new_inline_link(
            "Click here".to_string(),
            "https://rust-lang.org".to_string(),
        );
        link_box.dimensions.content = Rect {
            x: 60.0,
            y: 60.0,
            width: 80.0,
            height: 20.0,
        };

        child.children.push(link_box);
        root.children.push(child);

        // Hit link box
        assert_eq!(
            root.hit_test_link(70.0, 65.0),
            Some("https://rust-lang.org")
        );

        // Inside child but outside link box
        assert_eq!(root.hit_test_link(180.0, 100.0), None);

        // Outside root
        assert_eq!(root.hit_test_link(600.0, 500.0), None);
    }

    #[test]
    fn test_build_layout_tree_with_inline_hyperlinks() {
        use crate::dom::build_dom;
        use crate::html::Tokenizer;

        let html = r#"<div>Click <a href="https://example.com/about">About Us</a> now!</div>"#;
        let dom = build_dom(Tokenizer::new(html).tokenize());
        let styles = HashMap::new();

        let layout_tree =
            build_layout_tree(&dom, &styles, dom.root).expect("Layout tree must build");

        let div_box = &layout_tree.children[0];
        assert_eq!(div_box.children.len(), 3);
        assert_eq!(
            div_box.children[1].text_content.as_deref(),
            Some("About Us")
        );
        assert_eq!(
            div_box.children[1].link_url.as_deref(),
            Some("https://example.com/about")
        );
        assert_eq!(div_box.children[0].link_url, None);
        assert_eq!(div_box.children[2].link_url, None);
    }

    #[test]
    fn test_image_layout_dimensional_scaling() {
        use crate::dom::build_dom;
        use crate::html::Tokenizer;

        let html = r#"<div><img src="https://example.com/logo.png" alt="Logo" width="240" height="120"></div>"#;
        let dom = build_dom(Tokenizer::new(html).tokenize());
        let styles = HashMap::new();

        let mut layout_root =
            build_layout_tree(&dom, &styles, dom.root).expect("Layout tree must build");
        layout_tree(&mut layout_root, 800.0, &styles);

        let div_box = &layout_root.children[0];
        let img_box = &div_box.children[0];

        assert!(img_box.image_data.is_some());
        let img_data = img_box.image_data.as_ref().unwrap();
        assert_eq!(img_data.src, "https://example.com/logo.png");
        assert_eq!(img_data.alt, "Logo");
        assert_eq!(img_data.width, Some(240.0));
        assert_eq!(img_data.height, Some(120.0));

        // Dimensional scaling check
        assert_eq!(img_box.dimensions.content.width, 240.0);
        assert_eq!(img_box.dimensions.content.height, 120.0);
    }
}
