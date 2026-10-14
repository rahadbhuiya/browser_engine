// Batch 1 (completed here as a prerequisite for Batch 3) -- DOM tree
//
// Uses an arena (flat Vec of nodes with index-based parent/child links)
// instead of Rc<RefCell<...>> pointer trees. This is deliberate: arena
// indices make ancestor/sibling walks (needed for CSS combinator matching in
// Batch 3) simple and panic-free, and avoid the borrow-checker fights that
// come with a pointer-based tree in Rust.

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Document,
    Element(ElementData),
    Text(String),
    Comment(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElementData {
    pub tag: String,
    pub attributes: Vec<(String, String)>,
    pub id: Option<String>,
    pub classes: Vec<String>,
}

impl ElementData {
    pub fn new(tag: String, attributes: Vec<(String, String)>) -> Self {
        let id = attributes
            .iter()
            .find(|(k, _)| k == "id")
            .map(|(_, v)| v.clone());
        let classes = attributes
            .iter()
            .find(|(k, _)| k == "class")
            .map(|(_, v)| {
                v.split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        ElementData {
            tag,
            attributes,
            id,
            classes,
        }
    }

    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case(name))
    }

    pub fn has_class(&self, class: &str) -> bool {
        self.classes.iter().any(|c| c == class)
    }
}

#[derive(Debug, Clone)]
pub struct DomNode {
    pub node_type: NodeType,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Dom {
    pub nodes: Vec<DomNode>,
    pub root: usize,
}

impl Dom {
    pub fn element_at(&self, idx: usize) -> Option<&ElementData> {
        match &self.nodes.get(idx)?.node_type {
            NodeType::Element(e) => Some(e),
            _ => None,
        }
    }

    /// Returns the parent index of `idx`, if any.
    pub fn parent(&self, idx: usize) -> Option<usize> {
        self.nodes.get(idx)?.parent
    }

    /// Returns the children indices of `idx`.
    pub fn children(&self, idx: usize) -> &[usize] {
        self.nodes
            .get(idx)
            .map(|n| n.children.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the first child index of `idx`, if any.
    pub fn first_child(&self, idx: usize) -> Option<usize> {
        self.nodes.get(idx)?.children.first().copied()
    }

    /// Returns the last child index of `idx`, if any.
    pub fn last_child(&self, idx: usize) -> Option<usize> {
        self.nodes.get(idx)?.children.last().copied()
    }

    /// The index of the node immediately before `idx` among its siblings, if any.
    pub fn preceding_sibling(&self, idx: usize) -> Option<usize> {
        let parent = self.nodes.get(idx)?.parent?;
        let siblings = &self.nodes.get(parent)?.children;
        let pos = siblings.iter().position(|&n| n == idx)?;
        if pos == 0 {
            None
        } else {
            Some(siblings[pos - 1])
        }
    }

    /// The index of the node immediately following `idx` among its siblings, if any.
    pub fn next_sibling(&self, idx: usize) -> Option<usize> {
        let parent = self.nodes.get(idx)?.parent?;
        let siblings = &self.nodes.get(parent)?.children;
        let pos = siblings.iter().position(|&n| n == idx)?;
        siblings.get(pos + 1).copied()
    }

    /// All siblings before `idx`, in document order (nearest-first is NOT
    /// guaranteed here; order doesn't affect match correctness).
    pub fn preceding_siblings(&self, idx: usize) -> Vec<usize> {
        match self.nodes.get(idx).and_then(|n| n.parent) {
            Some(parent) => {
                if let Some(parent_node) = self.nodes.get(parent) {
                    let siblings = &parent_node.children;
                    match siblings.iter().position(|&n| n == idx) {
                        Some(pos) => siblings[..pos].to_vec(),
                        None => vec![],
                    }
                } else {
                    vec![]
                }
            }
            None => vec![],
        }
    }

    /// Returns all ancestor indices from `idx` up to the root.
    pub fn ancestors(&self, idx: usize) -> Vec<usize> {
        let mut list = Vec::new();
        let mut curr = self.nodes.get(idx).and_then(|n| n.parent);
        while let Some(p) = curr {
            list.push(p);
            curr = self.nodes.get(p).and_then(|n| n.parent);
        }
        list
    }

    /// Returns all descendant node indices under `idx` in depth-first pre-order.
    pub fn descendants(&self, idx: usize) -> Vec<usize> {
        let mut result = Vec::new();
        self.collect_descendants(idx, &mut result);
        result
    }

    fn collect_descendants(&self, idx: usize, result: &mut Vec<usize>) {
        if let Some(node) = self.nodes.get(idx) {
            for &child in &node.children {
                result.push(child);
                self.collect_descendants(child, result);
            }
        }
    }

    /// Finds an element node by its ID attribute.
    pub fn get_element_by_id(&self, id: &str) -> Option<usize> {
        self.nodes.iter().enumerate().find_map(|(idx, node)| {
            if let NodeType::Element(ref elem) = node.node_type {
                if elem.id.as_deref() == Some(id) {
                    return Some(idx);
                }
            }
            None
        })
    }

    /// Finds all element nodes matching the specified tag name (case-insensitive, or "*" for all).
    pub fn get_elements_by_tag_name(&self, tag: &str) -> Vec<usize> {
        let is_wildcard = tag == "*";
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                if let NodeType::Element(ref elem) = node.node_type {
                    if is_wildcard || elem.tag.eq_ignore_ascii_case(tag) {
                        return Some(idx);
                    }
                }
                None
            })
            .collect()
    }

    /// Finds all element nodes containing the given class name.
    pub fn get_elements_by_class_name(&self, class: &str) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                if let NodeType::Element(ref elem) = node.node_type {
                    if elem.has_class(class) {
                        return Some(idx);
                    }
                }
                None
            })
            .collect()
    }

    /// Recursively collects all text content within the node and its subtree.
    pub fn text_content(&self, idx: usize) -> String {
        let mut text = String::new();
        self.collect_text(idx, &mut text);
        text
    }

    fn collect_text(&self, idx: usize, out: &mut String) {
        if let Some(node) = self.nodes.get(idx) {
            match &node.node_type {
                NodeType::Text(t) => {
                    out.push_str(t);
                }
                _ => {
                    for &child in &node.children {
                        self.collect_text(child, out);
                    }
                }
            }
        }
    }

    /// Finds the first element in the document matching the given CSS selector string.
    pub fn query_selector(&self, selector_str: &str) -> Option<usize> {
        let selector = crate::css::parser::parse_selector(selector_str)?;
        let mut candidate_order = vec![self.root];
        candidate_order.extend(self.descendants(self.root));
        candidate_order
            .into_iter()
            .find(|&idx| crate::style::matching::matches_selector(self, idx, &selector))
    }

    /// Finds all elements in the document matching the given CSS selector string in document order.
    pub fn query_selector_all(&self, selector_str: &str) -> Vec<usize> {
        let selector = match crate::css::parser::parse_selector(selector_str) {
            Some(s) => s,
            None => return Vec::new(),
        };
        let mut candidate_order = vec![self.root];
        candidate_order.extend(self.descendants(self.root));
        candidate_order
            .into_iter()
            .filter(|&idx| crate::style::matching::matches_selector(self, idx, &selector))
            .collect()
    }
}
