use crate::dom::{Dom, NodeType};
use crate::net::fetch::fetch;
use crate::net::url_parser::ParsedUrl;

#[derive(Debug, Clone, PartialEq)]
pub struct DiscoveredResources {
    pub stylesheets: Vec<String>,
    pub images: Vec<String>,
}

/// Resolves a relative URL or path against a base page URL.
pub fn resolve_url(base_url: &str, relative_path: &str) -> String {
    let rel = relative_path.trim();
    if rel.starts_with("http://") || rel.starts_with("https://") {
        return rel.to_string();
    }
    if rel.starts_with("//") {
        let scheme = if base_url.starts_with("https:") { "https:" } else { "http:" };
        return format!("{}{}", scheme, rel);
    }

    let Ok(parsed_base) = ParsedUrl::parse(base_url) else {
        return rel.to_string();
    };

    let scheme_str = match parsed_base.scheme {
        crate::net::url_parser::Scheme::Http => "http",
        crate::net::url_parser::Scheme::Https => "https",
    };

    let port_str = match (parsed_base.scheme, parsed_base.port) {
        (crate::net::url_parser::Scheme::Http, 80) => String::new(),
        (crate::net::url_parser::Scheme::Https, 443) => String::new(),
        (_, p) => format!(":{}", p),
    };

    if rel.starts_with('/') {
        format!("{}://{}{}{}", scheme_str, parsed_base.host, port_str, rel)
    } else {
        let base_path = parsed_base.path_and_query;
        let dir = if let Some(last_slash) = base_path.rfind('/') {
            &base_path[..=last_slash]
        } else {
            "/"
        };
        format!("{}://{}{}{}{}", scheme_str, parsed_base.host, port_str, dir, rel)
    }
}

/// Scans DOM nodes to discover external stylesheets (<link rel="stylesheet">) and images (<img>).
pub fn discover_resources(dom: &Dom, base_url: &str) -> DiscoveredResources {
    let mut stylesheets = Vec::new();
    let mut images = Vec::new();

    for node in &dom.nodes {
        if let NodeType::Element(elem) = &node.node_type {
            let tag = elem.tag.to_ascii_lowercase();
            if tag == "link" {
                let is_rel_stylesheet = elem
                    .attributes
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("rel"))
                    .map(|(_, v)| v.to_ascii_lowercase().contains("stylesheet"))
                    .unwrap_or(false);
                if is_rel_stylesheet {
                    if let Some((_, href)) = elem
                        .attributes
                        .iter()
                        .find(|(k, _)| k.eq_ignore_ascii_case("href"))
                    {
                        if !href.trim().is_empty() {
                            stylesheets.push(resolve_url(base_url, href));
                        }
                    }
                }
            } else if tag == "img" {
                if let Some((_, src)) = elem
                    .attributes
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("src"))
                {
                    if !src.trim().is_empty() {
                        images.push(resolve_url(base_url, src));
                    }
                }
            }
        }
    }

    DiscoveredResources {
        stylesheets,
        images,
    }
}

pub struct ResourceLoader;

impl ResourceLoader {
    /// Fetches all discovered external stylesheets, returning the combined CSS string.
    pub fn fetch_all_stylesheets(urls: &[String]) -> String {
        let mut combined_css = String::new();
        for url in urls {
            println!("Fetching external stylesheet: {}", url);
            if let Ok(resp) = fetch(url) {
                if resp.status_code == 200 {
                    combined_css.push_str("\n/* External: ");
                    combined_css.push_str(url);
                    combined_css.push_str(" */\n");
                    combined_css.push_str(&resp.body_as_string());
                    combined_css.push('\n');
                }
            }
        }
        combined_css
    }
}
