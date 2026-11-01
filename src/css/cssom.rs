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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PseudoClass {
    Hover,
    Active,
    Focus,
    FirstChild,
    LastChild,
    OnlyChild,
    Root,
    Empty,
    Checked,
    Disabled,
    Enabled,
    Other(String),
}

impl PseudoClass {
    pub fn from_name(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "hover" => PseudoClass::Hover,
            "active" => PseudoClass::Active,
            "focus" => PseudoClass::Focus,
            "first-child" => PseudoClass::FirstChild,
            "last-child" => PseudoClass::LastChild,
            "only-child" => PseudoClass::OnlyChild,
            "root" => PseudoClass::Root,
            "empty" => PseudoClass::Empty,
            "checked" => PseudoClass::Checked,
            "disabled" => PseudoClass::Disabled,
            "enabled" => PseudoClass::Enabled,
            other => PseudoClass::Other(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            PseudoClass::Hover => "hover",
            PseudoClass::Active => "active",
            PseudoClass::Focus => "focus",
            PseudoClass::FirstChild => "first-child",
            PseudoClass::LastChild => "last-child",
            PseudoClass::OnlyChild => "only-child",
            PseudoClass::Root => "root",
            PseudoClass::Empty => "empty",
            PseudoClass::Checked => "checked",
            PseudoClass::Disabled => "disabled",
            PseudoClass::Enabled => "enabled",
            PseudoClass::Other(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SimpleSelector {
    pub universal: bool,
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub pseudo_classes: Vec<PseudoClass>,
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
            spec.1 += (simple.classes.len() + simple.pseudo_classes.len()) as u32;
            if simple.tag.is_some() {
                spec.2 += 1;
            }
        }
        spec
    }
}