use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::dom::{Dom, NodeType};
use crate::net::fetch::fetch;
use crate::net::url_parser::ParsedUrl;
use crate::render::{DecodedImage, ImageCache};

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

/// Thread-safe asynchronous pipeline for fetching and caching remote/data image resources.
#[derive(Clone, Default)]
pub struct AsyncImagePipeline {
    pub cache: Arc<Mutex<ImageCache>>,
    pub in_flight: Arc<Mutex<HashSet<String>>>,
}

impl AsyncImagePipeline {
    pub fn new() -> Self {
        AsyncImagePipeline {
            cache: Arc::new(Mutex::new(ImageCache::new())),
            in_flight: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn with_cache(cache: Arc<Mutex<ImageCache>>) -> Self {
        AsyncImagePipeline {
            cache,
            in_flight: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn get_image(&self, url: &str) -> Option<DecodedImage> {
        self.cache.lock().unwrap().get(url).cloned()
    }

    pub fn is_cached(&self, url: &str) -> bool {
        self.cache.lock().unwrap().get(url).is_some()
    }

    pub fn is_in_flight(&self, url: &str) -> bool {
        self.in_flight.lock().unwrap().contains(url)
    }

    pub fn insert_decoded(&self, url: &str, decoded: DecodedImage) {
        self.cache.lock().unwrap().insert(url, decoded);
    }

    pub fn queue_fetch<F>(&self, url: &str, on_complete: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let url = url.trim();
        if url.is_empty() || self.is_cached(url) {
            return;
        }

        let mut in_flight = self.in_flight.lock().unwrap();
        if in_flight.contains(url) {
            return;
        }
        in_flight.insert(url.to_string());
        drop(in_flight);

        let url_owned = url.to_string();
        let cache_clone = self.cache.clone();
        let in_flight_clone = self.in_flight.clone();

        std::thread::spawn(move || {
            let bytes = if url_owned.starts_with("data:image/") {
                decode_data_uri(&url_owned)
            } else if url_owned.starts_with("http://") || url_owned.starts_with("https://") {
                match fetch(&url_owned) {
                    Ok(resp) if resp.status_code == 200 => Some(resp.body),
                    _ => None,
                }
            } else {
                None
            };

            if let Some(bytes) = bytes {
                let mut cache = cache_clone.lock().unwrap();
                let _ = cache.load_from_memory(&url_owned, &bytes);
                drop(cache);
                on_complete();
            }

            in_flight_clone.lock().unwrap().remove(&url_owned);
        });
    }
}

/// Decodes a base64 or raw data URI (`data:image/...;base64,...`).
pub fn decode_data_uri(data_uri: &str) -> Option<Vec<u8>> {
    let comma_idx = data_uri.find(',')?;
    let meta = &data_uri[..comma_idx];
    let payload = &data_uri[comma_idx + 1..];
    if meta.contains(";base64") {
        decode_base64(payload)
    } else {
        Some(payload.as_bytes().to_vec())
    }
}

pub fn decode_base64(input: &str) -> Option<Vec<u8>> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lookup = [255u8; 256];
    for (i, &b) in TABLE.iter().enumerate() {
        lookup[b as usize] = i as u8;
    }
    let clean: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace() && *b != b'=').collect();
    let mut out = Vec::with_capacity(clean.len() * 3 / 4);
    for chunk in clean.chunks(4) {
        let b0 = *lookup.get(*chunk.first()? as usize)?;
        let b1 = *lookup.get(*chunk.get(1)? as usize)?;
        if b0 == 255 || b1 == 255 {
            return None;
        }
        out.push((b0 << 2) | (b1 >> 4));
        if chunk.len() > 2 {
            let b2 = *lookup.get(*chunk.get(2)? as usize)?;
            if b2 == 255 {
                return None;
            }
            out.push(((b1 & 0xF) << 4) | (b2 >> 2));
            if chunk.len() > 3 {
                let b3 = *lookup.get(*chunk.get(3)? as usize)?;
                if b3 == 255 {
                    return None;
                }
                out.push(((b2 & 0x3) << 6) | b3);
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_base64_and_data_uri() {
        let raw = b"Hello Browser!";
        // "Hello Browser!" in base64 is "SGVsbG8gQnJvd3NlciE="
        let encoded = "SGVsbG8gQnJvd3NlciE=";
        let decoded = decode_base64(encoded).expect("base64 decoded");
        assert_eq!(decoded, raw);

        let data_uri = "data:image/png;base64,SGVsbG8gQnJvd3NlciE=";
        let uri_decoded = decode_data_uri(data_uri).expect("data uri decoded");
        assert_eq!(uri_decoded, raw);
    }

    #[test]
    fn test_async_image_pipeline_caching() {
        let pipeline = AsyncImagePipeline::new();
        assert!(!pipeline.is_cached("https://example.com/photo.png"));

        // 1x1 valid PNG
        let png_bytes: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
            0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        pipeline.cache.lock().unwrap().load_from_memory("https://example.com/photo.png", png_bytes).unwrap();
        assert!(pipeline.is_cached("https://example.com/photo.png"));

        let img = pipeline.get_image("https://example.com/photo.png").unwrap();
        assert_eq!(img.width, 1);
        assert_eq!(img.height, 1);
    }
}
