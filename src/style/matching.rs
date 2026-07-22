// Batch 3 — Selector matching
//
// Matches a CSS Selector (a combinator chain from Batch 2's CSSOM) against a
// specific node in the DOM tree (Batch 1). Standard engines match right-to-
// left (check the rightmost simple selector against the candidate element
// first, since it's cheapest to reject non-matches early) -- we do the same.

use crate::css::{Combinator, Selector, SimpleSelector};
use crate::dom::Dom;

pub fn matches_selector(dom: &Dom, node_idx: usize, selector: &Selector) -> bool {
    let steps = &selector.steps;
    if steps.is_empty() {
        return false;
    }
    let last = steps.len() - 1;
    if !matches_simple(dom, node_idx, &steps[last].1) {
        return false;
    }
    satisfies_combinators(dom, node_idx, steps, last)
}

/// Having already matched `steps[idx]` against `node_idx`, walk backward
/// through the remaining combinator chain (steps[0..idx]) to confirm the
/// rest of the selector is satisfied by this node's ancestors/siblings.
fn satisfies_combinators(
    dom: &Dom,
    node_idx: usize,
    steps: &[(Option<Combinator>, SimpleSelector)],
    idx: usize,
) -> bool {
    if idx == 0 {
        return true; // no earlier steps left to satisfy
    }
    let combinator = match steps[idx].0 {
        Some(c) => c,
        None => return true, // malformed chain; treat as satisfied rather than panic
    };
    let target = &steps[idx - 1].1;

    match combinator {
        Combinator::Descendant => {
            let mut current = dom.nodes[node_idx].parent;
            while let Some(p) = current {
                if matches_simple(dom, p, target) && satisfies_combinators(dom, p, steps, idx - 1)
                {
                    return true;
                }
                current = dom.nodes[p].parent;
            }
            false
        }
        Combinator::Child => match dom.nodes[node_idx].parent {
            Some(p) => matches_simple(dom, p, target) && satisfies_combinators(dom, p, steps, idx - 1),
            None => false,
        },
        Combinator::NextSibling => match dom.preceding_sibling(node_idx) {
            Some(p) => matches_simple(dom, p, target) && satisfies_combinators(dom, p, steps, idx - 1),
            None => false,
        },
        Combinator::SubsequentSibling => dom.preceding_siblings(node_idx).into_iter().any(|s| {
            matches_simple(dom, s, target) && satisfies_combinators(dom, s, steps, idx - 1)
        }),
    }
}

fn matches_simple(dom: &Dom, node_idx: usize, simple: &SimpleSelector) -> bool {
    let element = match dom.element_at(node_idx) {
        Some(e) => e,
        None => return false, // text/comment/document nodes never match
    };

    if let Some(tag) = &simple.tag {
        if &element.tag != tag {
            return false;
        }
    }
    if let Some(id) = &simple.id {
        if element.id.as_deref() != Some(id.as_str()) {
            return false;
        }
    }
    for class in &simple.classes {
        if !element.classes.iter().any(|c| c == class) {
            return false;
        }
    }
    // universal selector or bare tag/id/class combination: if we got here,
    // every constraint present on `simple` was satisfied
    true
}