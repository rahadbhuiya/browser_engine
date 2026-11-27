//! CSS Cascade and computed style calculation.

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

struct MatchedDecl<'a> {
    specificity: (u32, u32, u32),
    order: usize,
    important: bool,
    property: &'a str,
    value: &'a str,
}

fn parse_inline_style(style_str: &str) -> Vec<(String, String)> {
    let mut decls = Vec::new();
    for part in style_str.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((prop, val)) = part.split_once(':') {
            let p = prop.trim().to_ascii_lowercase();
            let v = val.trim().to_string();
            if !p.is_empty() && !v.is_empty() {
                decls.push((p, v));
            }
        }
    }
    decls
}

pub fn compute_styles(dom: &Dom, stylesheet: &Stylesheet) -> HashMap<usize, ComputedStyle> {
    compute_styles_with_viewport(dom, stylesheet, 1024.0, false)
}

pub fn compute_styles_with_viewport(
    dom: &Dom,
    stylesheet: &Stylesheet,
    viewport_width: f32,
    is_dark: bool,
) -> HashMap<usize, ComputedStyle> {
    let mut result = HashMap::new();
    compute_recursive(
        dom,
        dom.root,
        stylesheet,
        None,
        &mut result,
        viewport_width,
        is_dark,
    );
    result
}

pub fn compute_styles_with_ua(dom: &Dom, stylesheet: &Stylesheet) -> HashMap<usize, ComputedStyle> {
    let mut merged = crate::css::default_user_agent_stylesheet();
    merged.rules.extend(stylesheet.rules.clone());
    compute_styles(dom, &merged)
}

fn compute_recursive(
    dom: &Dom,
    node_idx: usize,
    stylesheet: &Stylesheet,
    parent_style: Option<&ComputedStyle>,
    result: &mut HashMap<usize, ComputedStyle>,
    viewport_width: f32,
    is_dark: bool,
) {
    if let NodeType::Element(el) = &dom.nodes[node_idx].node_type {
        let own = resolve_own_declarations(dom, node_idx, stylesheet, viewport_width, is_dark);
        let mut style = ComputedStyle::new();

        if let Some(parent) = parent_style {
            for prop in INHERITED_PROPERTIES {
                if let Some(v) = parent.get(*prop) {
                    style.insert(prop.to_string(), v.clone());
                }
            }
            for (k, v) in parent {
                if k.starts_with("--") {
                    style.insert(k.clone(), v.clone());
                }
            }
        }
        for (k, v) in own {
            style.insert(k, v);
        }

        if let Some(inline_str) = el.get_attribute("style") {
            for (k, v) in parse_inline_style(inline_str) {
                style.insert(k, v);
            }
        }

        resolve_all_variables(&mut style);

        result.insert(node_idx, style);
    }

    let child_style = result.get(&node_idx).cloned();
    for &child in &dom.nodes[node_idx].children.clone() {
        compute_recursive(
            dom,
            child,
            stylesheet,
            child_style.as_ref(),
            result,
            viewport_width,
            is_dark,
        );
    }
}

fn resolve_var_expression(val: &str, style: &ComputedStyle, depth: usize) -> String {
    if depth > 16 || !val.contains("var(") {
        return val.to_string();
    }
    let mut result = String::with_capacity(val.len());
    let mut remaining = val;
    while let Some(start_idx) = remaining.find("var(") {
        result.push_str(&remaining[..start_idx]);
        let after_var = &remaining[start_idx + 4..];
        let mut depth_paren = 1;
        let mut end_idx = None;
        for (i, c) in after_var.char_indices() {
            if c == '(' {
                depth_paren += 1;
            } else if c == ')' {
                depth_paren -= 1;
                if depth_paren == 0 {
                    end_idx = Some(i);
                    break;
                }
            }
        }
        if let Some(end_paren) = end_idx {
            let inner = after_var[..end_paren].trim();
            let (var_name, fallback) = if let Some((name, fb)) = inner.split_once(',') {
                (name.trim(), Some(fb.trim()))
            } else {
                (inner, None)
            };

            let resolved = if let Some(v) = style.get(var_name) {
                resolve_var_expression(v, style, depth + 1)
            } else if let Some(fb) = fallback {
                resolve_var_expression(fb, style, depth + 1)
            } else {
                String::new()
            };
            result.push_str(&resolved);
            remaining = &after_var[end_paren + 1..];
        } else {
            result.push_str("var(");
            remaining = after_var;
        }
    }
    result.push_str(remaining);
    result
}

fn resolve_all_variables(style: &mut ComputedStyle) {
    let snapshot = style.clone();
    for val in style.values_mut() {
        if val.contains("var(") {
            *val = resolve_var_expression(val, &snapshot, 0);
        }
    }
}

fn resolve_own_declarations(
    dom: &Dom,
    node_idx: usize,
    stylesheet: &Stylesheet,
    viewport_width: f32,
    is_dark: bool,
) -> ComputedStyle {
    let mut matched: Vec<MatchedDecl> = Vec::new();

    for (order, rule) in stylesheet.rules.iter().enumerate() {
        if let Some(mq) = &rule.media_query {
            if !mq.matches(viewport_width, is_dark) {
                continue;
            }
        }

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
