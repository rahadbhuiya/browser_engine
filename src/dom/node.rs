// Batch 1 (completed here as a prerequisite for Batch 3) — DOM tree
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
        match &self.nodes[idx].node_type {
            NodeType::Element(e) => Some(e),
            _ => None,
        }
    }

    /// The index of the node immediately before `idx` among its siblings, if any.
    pub fn preceding_sibling(&self, idx: usize) -> Option<usize> {
        let parent = self.nodes[idx].parent?;
        let siblings = &self.nodes[parent].children;
        let pos = siblings.iter().position(|&n| n == idx)?;
        if pos == 0 {
            None
        } else {
            Some(siblings[pos - 1])
        }
    }

    /// All siblings before `idx`, in document order (nearest-first is NOT
    /// guaranteed here — callers needing "any preceding sibling" for the `~`
    /// combinator should just iterate; order doesn't affect match correctness).
    pub fn preceding_siblings(&self, idx: usize) -> Vec<usize> {
        match self.nodes[idx].parent {
            Some(parent) => {
                let siblings = &self.nodes[parent].children;
                match siblings.iter().position(|&n| n == idx) {
                    Some(pos) => siblings[..pos].to_vec(),
                    None => vec![],
                }
            }
            None => vec![],
        }
    }
}