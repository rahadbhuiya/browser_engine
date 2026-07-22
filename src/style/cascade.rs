// Batch 3 — Cascade + computed style
//
// For every element in the DOM, gathers all CSS declarations whose selector
// matches it, resolves the cascade (specificity + !important + source order,
// per https://www.w3.org/TR/css-cascade-3/#cascade-sort), and produces a
// computed style map. A small fixed list of inherited properties (color,
// font-family, etc.) is propagated from parent to child when not explicitly
// set, matching real browsers' default inheritance behavior.

use std::collections::HashMap;

use crate::css::Stylesheet;
use crate::dom::{Dom, NodeType};

use super::matching::matches_selector;

pub type ComputedStyle = HashMap<String, String>;

const INHERITED_PROPERTIES: &[&str] = &[
    "color",
    "font-family",
    "font-size",
    "font-weight",
    "line-height",
    "text-align",
    "visibility",
];

/// One matched declaration, tagged with the specificity/order/importance
/// needed to resolve the cascade if multiple rules set the same property.
struct MatchedDecl<'a> {
    specificity: (u32, u32, u32),
    order: usize,
    important: bool,
    property: &'a str,
    value: &'a str,
}

pub fn compute_styles(dom: &Dom, stylesheet: &Stylesheet) -> HashMap<usize, ComputedStyle> {
    let mut result = HashMap::new();
    compute_recursive(dom, dom.root, stylesheet, None, &mut result);
    result
}

fn compute_recursive(
    dom: &Dom,
    node_idx: usize,
    stylesheet: &Stylesheet,
    parent_style: Option<&ComputedStyle>,
    result: &mut HashMap<usize, ComputedStyle>,
) {
    if let NodeType::Element(_) = &dom.nodes[node_idx].node_type {
        let own = resolve_own_declarations(dom, node_idx, stylesheet);
        let mut style = ComputedStyle::new();

        // 1. inherit from parent first (so explicit declarations can override)
        if let Some(parent) = parent_style {
            for prop in INHERITED_PROPERTIES {
                if let Some(v) = parent.get(*prop) {
                    style.insert(prop.to_string(), v.clone());
                }
            }
        }
        // 2. apply this element's own cascaded declarations
        for (k, v) in own {
            style.insert(k, v);
        }

        result.insert(node_idx, style);
    }

    let child_style = result.get(&node_idx).cloned();
    for &child in &dom.nodes[node_idx].children.clone() {
        compute_recursive(dom, child, stylesheet, child_style.as_ref(), result);
    }
}

fn resolve_own_declarations(
    dom: &Dom,
    node_idx: usize,
    stylesheet: &Stylesheet,
) -> ComputedStyle {
    let mut matched: Vec<MatchedDecl> = Vec::new();

    for (order, rule) in stylesheet.rules.iter().enumerate() {
        let best_specificity = rule
            .selectors
            .iter()
            .filter(|sel| matches_selector(dom, node_idx, sel))
            .map(|sel| sel.specificity())
            .max();

        let Some(specificity) = best_specificity else {
            continue;
        };

        for decl in &rule.declarations {
            matched.push(MatchedDecl {
                specificity,
                order,
                important: decl.important,
                property: &decl.property,
                value: &decl.value,
            });
        }
    }

    // Group by property, then pick the cascade winner for each:
    // !important beats normal regardless of specificity; within the same
    // importance tier, higher specificity wins; ties broken by source order
    // (later rule wins), matching the CSS cascade-sort algorithm.
    let mut by_property: HashMap<&str, Vec<&MatchedDecl>> = HashMap::new();
    for decl in &matched {
        by_property.entry(decl.property).or_default().push(decl);
    }

    let mut style = ComputedStyle::new();
    for (property, decls) in by_property {
        let winner = decls.iter().max_by(|a, b| {
            (a.important, a.specificity, a.order).cmp(&(b.important, b.specificity, b.order))
        });
        if let Some(winner) = winner {
            style.insert(property.to_string(), winner.value.to_string());
        }
    }
    style
}