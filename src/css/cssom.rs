// Batch 2 — CSSOM data structures

#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: String,
    pub value: String,
    pub important: bool,
}

/// A selector is a chain of simple selectors joined by combinators, e.g.
/// `div.card > p.title` = [ (None, div.card), (Child, p.title) ]
#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub steps: Vec<(Option<Combinator>, SimpleSelector)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Combinator {
    Descendant, // "a b"
    Child,      // "a > b"
    NextSibling, // "a + b"
    SubsequentSibling, // "a ~ b"
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SimpleSelector {
    pub universal: bool,
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
}

/// (id_count, class_count, type_count) — CSS specificity, per
/// https://www.w3.org/TR/selectors-3/#specificity
pub type Specificity = (u32, u32, u32);

impl Selector {
    pub fn specificity(&self) -> Specificity {
        let mut spec = (0, 0, 0);
        for (_, simple) in &self.steps {
            if simple.id.is_some() {
                spec.0 += 1;
            }
            spec.1 += simple.classes.len() as u32;
            if simple.tag.is_some() {
                spec.2 += 1;
            }
        }
        spec
    }
}