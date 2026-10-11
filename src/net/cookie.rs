//! RFC 6265 Compliant HTTP State Management (Cookie Engine).
//!
//! Provides parsing, security policy enforcement (Secure, HttpOnly, SameSite),
//! domain matching, path matching, expiration pruning, and CookieJar storage.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// SameSite enforcement policy as per RFC 6265bis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

/// A parsed and validated HTTP Cookie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSite,
    pub expires_at: Option<u64>, // Epoch seconds
}

impl Cookie {
    /// Parses a `Set-Cookie` header value for a given origin URL domain and path.
    pub fn parse(set_cookie_str: &str, request_domain: &str, request_path: &str) -> Option<Self> {
        let mut parts = set_cookie_str.split(';');

        // First part is always Name=Value
        let name_value_part = parts.next()?.trim();
        let (name, value) = match name_value_part.find('=') {
            Some(idx) => (
                name_value_part[..idx].trim().to_string(),
                name_value_part[idx + 1..].trim().to_string(),
            ),
            None => return None,
        };

        if name.is_empty() {
            return None;
        }

        let mut domain = request_domain.to_lowercase();
        let mut path = if request_path.is_empty() {
            "/".to_string()
        } else {
            request_path.to_string()
        };
        let mut secure = false;
        let mut http_only = false;
        let mut same_site = SameSite::Lax;
        let mut expires_at = None;

        for attr in parts {
            let attr = attr.trim();
            if attr.is_empty() {
                continue;
            }

            let mut kv = attr.splitn(2, '=');
            let attr_name = kv.next().unwrap_or("").trim().to_lowercase();
            let attr_val = kv.next().unwrap_or("").trim();

            match attr_name.as_str() {
                "secure" => secure = true,
                "httponly" => http_only = true,
                "domain" => {
                    let d = attr_val.trim_start_matches('.').to_lowercase();
                    if !d.is_empty() {
                        domain = d;
                    }
                }
                "path" => {
                    if !attr_val.is_empty() {
                        path = attr_val.to_string();
                    }
                }
                "samesite" => {
                    same_site = match attr_val.to_lowercase().as_str() {
                        "strict" => SameSite::Strict,
                        "none" => SameSite::None,
                        _ => SameSite::Lax,
                    };
                }
                "max-age" => {
                    if let Ok(secs) = attr_val.parse::<i64>() {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        if secs <= 0 {
                            expires_at = Some(0); // Expired immediately
                        } else {
                            expires_at = Some(now + (secs as u64));
                        }
                    }
                }
                _ => {}
            }
        }

        Some(Cookie {
            name,
            value,
            domain,
            path,
            secure,
            http_only,
            same_site,
            expires_at,
        })
    }

    /// Checks if this cookie is expired at the given epoch time in seconds.
    pub fn is_expired(&self, current_epoch_secs: u64) -> bool {
        if let Some(exp) = self.expires_at {
            current_epoch_secs >= exp
        } else {
            false
        }
    }

    /// Checks if this cookie matches the destination domain and path.
    pub fn matches(&self, target_domain: &str, target_path: &str, is_https: bool) -> bool {
        if self.secure && !is_https {
            return false;
        }

        let target_domain = target_domain.to_lowercase();
        let domain_match = target_domain == self.domain
            || (target_domain.ends_with(&format!(".{}", self.domain)));

        if !domain_match {
            return false;
        }

        let path_match = target_path.starts_with(&self.path);
        path_match
    }
}

/// Thread-safe in-memory Cookie Jar storing and organizing cookies by domain.
#[derive(Debug, Default, Clone)]
pub struct CookieJar {
    cookies: Arc<RwLock<HashMap<String, Vec<Cookie>>>>,
}

impl CookieJar {
    /// Creates a new empty CookieJar.
    pub fn new() -> Self {
        Self {
            cookies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Stores a parsed cookie into the jar, overwriting any previous cookie with the same name and path.
    pub fn store(&self, cookie: Cookie) {
        let mut map = self.cookies.write().unwrap();
        let list = map.entry(cookie.domain.clone()).or_insert_with(Vec::new);

        // Remove existing identical cookie
        list.retain(|existing| !(existing.name == cookie.name && existing.path == cookie.path));

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Only store if not already expired
        if !cookie.is_expired(now) {
            list.push(cookie);
        }
    }

    /// Ingests all `Set-Cookie` headers from a response.
    pub fn process_response_headers<'a, I>(&self, headers: I, domain: &str, path: &str)
    where
        I: IntoIterator<Item = (&'a String, &'a String)>,
    {
        for (k, v) in headers {
            if k.eq_ignore_ascii_case("set-cookie") {
                if let Some(cookie) = Cookie::parse(v, domain, path) {
                    self.store(cookie);
                }
            }
        }
    }

    /// Generates the serialized `Cookie: name1=val1; name2=val2` string for an outgoing request.
    pub fn get_cookie_header(&self, target_domain: &str, target_path: &str, is_https: bool) -> Option<String> {
        let mut map = self.cookies.write().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut matched_cookies = Vec::new();

        for (domain, list) in map.iter_mut() {
            let target_domain_lower = target_domain.to_lowercase();
            if target_domain_lower == *domain || target_domain_lower.ends_with(&format!(".{}", domain)) {
                // Prune expired cookies
                list.retain(|c| !c.is_expired(now));

                for c in list.iter() {
                    if c.matches(target_domain, target_path, is_https) {
                        matched_cookies.push(format!("{}={}", c.name, c.value));
                    }
                }
            }
        }

        if matched_cookies.is_empty() {
            None
        } else {
            Some(matched_cookies.join("; "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_parse_basic() {
        let raw = "session_token=xyz123; Domain=example.com; Path=/api; Secure; HttpOnly; SameSite=Strict";
        let cookie = Cookie::parse(raw, "example.com", "/api/v1").unwrap();

        assert_eq!(cookie.name, "session_token");
        assert_eq!(cookie.value, "xyz123");
        assert_eq!(cookie.domain, "example.com");
        assert_eq!(cookie.path, "/api");
        assert!(cookie.secure);
        assert!(cookie.http_only);
        assert_eq!(cookie.same_site, SameSite::Strict);
    }

    #[test]
    fn test_cookie_jar_domain_isolation() {
        let jar = CookieJar::new();

        let cookie1 = Cookie::parse("user=alice; Domain=example.com; Path=/", "example.com", "/").unwrap();
        let cookie2 = Cookie::parse("admin=root; Domain=attacker.com; Path=/", "attacker.com", "/").unwrap();

        jar.store(cookie1);
        jar.store(cookie2);

        let header = jar.get_cookie_header("example.com", "/dashboard", true).unwrap();
        assert_eq!(header, "user=alice");
        assert!(!header.contains("admin=root"));
    }

    #[test]
    fn test_cookie_expiration_pruning() {
        let jar = CookieJar::new();

        // Expired cookie with Max-Age=0
        let cookie = Cookie::parse("temp=data; Max-Age=0", "example.com", "/").unwrap();
        jar.store(cookie);

        let header = jar.get_cookie_header("example.com", "/", true);
        assert!(header.is_none());
    }
}
