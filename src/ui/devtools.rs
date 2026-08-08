use crate::dom::{Dom, NodeType};
use crate::layout::LayoutBox;

pub struct DevTools;

impl DevTools {
    pub fn inspect_dom(dom: &Dom) -> String {
        let mut out = String::from("=== DEVTOOLS: DOM TREE INSPECTOR ===\n");
        Self::format_node(dom, dom.root, 0, &mut out);
        out
    }

    fn format_node(dom: &Dom, idx: usize, depth: usize, out: &mut String) {
        if idx >= dom.nodes.len() {
            return;
        }
        let indent = "  ".repeat(depth);
        let node = &dom.nodes[idx];
        match &node.node_type {
            NodeType::Document => {
                out.push_str(&format!("{indent}#document (id={})\n", idx));
            }
            NodeType::Element(data) => {
                let attrs_str = data.attributes
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect::<Vec<_>>()
                    .join(" ");
                out.push_str(&format!("{indent}<{} {}> (id={})\n", data.tag, attrs_str, idx));
            }
            NodeType::Text(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    out.push_str(&format!("{indent}\"{}\"\n", trimmed));
                }
            }
            NodeType::Comment(cmt) => {
                out.push_str(&format!("{indent}<!-- {} -->\n", cmt));
            }
        }

        for &child in &node.children {
            Self::format_node(dom, child, depth + 1, out);
        }
    }

    pub fn inspect_layout(lb: &LayoutBox) -> String {
        let mut out = String::from("=== DEVTOOLS: LAYOUT BOX INSPECTOR ===\n");
        Self::format_layout_box(lb, 0, &mut out);
        out
    }

    fn format_layout_box(lb: &LayoutBox, depth: usize, out: &mut String) {
        let indent = "  ".repeat(depth);
        let d = lb.dimensions.content;
        out.push_str(&format!(
            "{indent}Box x={:.0} y={:.0} w={:.0} h={:.0}\n",
            d.x, d.y, d.width, d.height
        ));
        for child in &lb.children {
            Self::format_layout_box(child, depth + 1, out);
        }
    }
}
