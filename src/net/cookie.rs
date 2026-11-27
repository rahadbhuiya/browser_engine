//! RFC 6265 Compliant HTTP State Management (Cookie Engine).
//!
//! Provides parsing, security policy enforcement (Secure, HttpOnly, SameSite),
//! domain matching, path matching, expiration pruning, and CookieJar storage.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
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
        let domain_match =
            target_domain == self.domain || (target_domain.ends_with(&format!(".{}", self.domain)));

        if !domain_match {
            return false;
        }

        let path_match = target_path.starts_with(&self.path);
        path_match
    }

    /// Serializes this cookie into a JSON string object.
    pub fn to_json(&self) -> String {
        let same_site_str = match self.same_site {
            SameSite::Strict => "Strict",
            SameSite::Lax => "Lax",
            SameSite::None => "None",
        };
        let exp_str = match self.expires_at {
            Some(exp) => exp.to_string(),
            None => "null".to_string(),
        };
        format!(
            "{{\"name\":\"{}\",\"value\":\"{}\",\"domain\":\"{}\",\"path\":\"{}\",\"secure\":{},\"http_only\":{},\"same_site\":\"{}\",\"expires_at\":{}}}",
            escape_json(&self.name),
            escape_json(&self.value),
            escape_json(&self.domain),
            escape_json(&self.path),
            self.secure,
            self.http_only,
            same_site_str,
            exp_str
        )
    }

    /// Deserializes a cookie from a key-value map.
    pub fn from_json_map(map: &HashMap<String, String>) -> Option<Self> {
        let name = map.get("name")?.clone();
        let value = map.get("value")?.clone();
        let domain = map.get("domain")?.clone();
        let path = map.get("path").cloned().unwrap_or_else(|| "/".to_string());
        let secure = map.get("secure").map(|s| s == "true").unwrap_or(false);
        let http_only = map.get("http_only").map(|s| s == "true").unwrap_or(false);
        let same_site = match map.get("same_site").map(|s| s.as_str()) {
            Some("Strict") => SameSite::Strict,
            Some("None") => SameSite::None,
            _ => SameSite::Lax,
        };
        let expires_at = map.get("expires_at").and_then(|s| {
            if s == "null" {
                None
            } else {
                s.parse::<u64>().ok()
            }
        });

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
    pub fn get_cookie_header(
        &self,
        target_domain: &str,
        target_path: &str,
        is_https: bool,
    ) -> Option<String> {
        let mut map = self.cookies.write().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut matched_cookies = Vec::new();

        for (domain, list) in map.iter_mut() {
            let target_domain_lower = target_domain.to_lowercase();
            if target_domain_lower == *domain
                || target_domain_lower.ends_with(&format!(".{}", domain))
            {
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

    /// Cleans up in-memory session cookies (cookies with no explicit `expires_at`).
    /// Typically invoked on browser shutdown.
    pub fn clear_session_cookies(&self) {
        let mut map = self.cookies.write().unwrap();
        for list in map.values_mut() {
            list.retain(|c| c.expires_at.is_some());
        }
    }

    /// Serializes all persistent non-expired cookies to disk at the specified path.
    pub fn save_to_disk(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let map = self.cookies.read().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut entries = Vec::new();
        for list in map.values() {
            for c in list {
                // Only persist persistent cookies (with expires_at) that are not expired
                if c.expires_at.is_some() && !c.is_expired(now) {
                    entries.push(c.to_json());
                }
            }
        }

        let json = format!("[\n  {}\n]\n", entries.join(",\n  "));
        fs::write(path, json)
    }

    /// Loads persisted cookies from disk, filtering out any that have expired.
    pub fn load_from_disk(path: &Path) -> Result<Self, std::io::Error> {
        let jar = Self::new();
        if !path.exists() {
            return Ok(jar);
        }

        let contents = fs::read_to_string(path)?;
        let objects = parse_json_object_list(&contents);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for obj in objects {
            if let Some(cookie) = Cookie::from_json_map(&obj) {
                if !cookie.is_expired(now) {
                    jar.store(cookie);
                }
            }
        }

        Ok(jar)
    }

    /// Returns the default persistent cookie storage file path (`~/.browser_engine/cookies.json`).
    pub fn default_storage_path() -> PathBuf {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".browser_engine")
            .join("cookies.json")
    }
}

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

pub fn parse_json_object_list(json_str: &str) -> Vec<HashMap<String, String>> {
    let mut result = Vec::new();
    let trimmed = json_str.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return result;
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    let mut in_string = false;
    let mut escape = false;
    let mut depth = 0;
    let mut current_obj = String::new();

    for ch in inner.chars() {
        if escape {
            if depth > 0 {
                current_obj.push(ch);
            }
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            if depth > 0 {
                current_obj.push(ch);
            }
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            if depth > 0 {
                current_obj.push(ch);
            }
            continue;
        }
        if !in_string {
            if ch == '{' {
                depth += 1;
                if depth == 1 {
                    current_obj.clear();
                    continue;
                }
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    if let Some(map) = parse_json_kv(&current_obj) {
                        result.push(map);
                    }
                    current_obj.clear();
                    continue;
                }
            }
        }
        if depth > 0 {
            current_obj.push(ch);
        }
    }
    result
}

fn parse_json_kv(obj_str: &str) -> Option<HashMap<String, String>> {
    let mut map = HashMap::new();
    let mut in_string = false;
    let mut escape = false;
    let mut current_key = String::new();
    let mut current_val = String::new();
    let mut reading_key = true;

    let chars: Vec<char> = obj_str.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if escape {
            if reading_key {
                current_key.push(ch);
            } else {
                current_val.push(ch);
            }
            escape = false;
            i += 1;
            continue;
        }
        if ch == '\\' {
            escape = true;
            i += 1;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            i += 1;
            continue;
        }
        if !in_string {
            if ch == ':' && reading_key {
                reading_key = false;
                i += 1;
                continue;
            } else if ch == ',' {
                let k = current_key.trim().trim_matches('"').to_string();
                let v = current_val.trim().trim_matches('"').to_string();
                if !k.is_empty() {
                    map.insert(k, v);
                }
                current_key.clear();
                current_val.clear();
                reading_key = true;
                i += 1;
                continue;
            }
        }
        if reading_key {
            current_key.push(ch);
        } else {
            current_val.push(ch);
        }
        i += 1;
    }
    let k = current_key.trim().trim_matches('"').to_string();
    let v = current_val.trim().trim_matches('"').to_string();
    if !k.is_empty() {
        map.insert(k, v);
    }
    Some(map)
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

        let cookie1 =
            Cookie::parse("user=alice; Domain=example.com; Path=/", "example.com", "/").unwrap();
        let cookie2 = Cookie::parse(
            "admin=root; Domain=attacker.com; Path=/",
            "attacker.com",
            "/",
        )
        .unwrap();

        jar.store(cookie1);
        jar.store(cookie2);

        let header = jar
            .get_cookie_header("example.com", "/dashboard", true)
            .unwrap();
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

    #[test]
    fn test_cookie_disk_persistence_and_session_cleanup() {
        let temp_dir = std::env::temp_dir().join(format!("test_cookie_jar_{}", std::process::id()));
        let cookie_file = temp_dir.join("cookies.json");

        let jar = CookieJar::new();
        // Persistent cookie with Max-Age 3600
        let persistent = Cookie::parse(
            "auth_token=secret123; Domain=example.com; Path=/; Max-Age=3600; Secure; HttpOnly",
            "example.com",
            "/",
        )
        .unwrap();
        // Session cookie (no Max-Age/Expires)
        let session = Cookie::parse(
            "session_id=sess456; Domain=example.com; Path=/",
            "example.com",
            "/",
        )
        .unwrap();

        jar.store(persistent);
        jar.store(session);

        // Verify both are present in memory
        let header = jar.get_cookie_header("example.com", "/", true).unwrap();
        assert!(header.contains("auth_token=secret123"));
        assert!(header.contains("session_id=sess456"));

        // Save to disk
        jar.save_to_disk(&cookie_file).unwrap();

        // Load into a new jar
        let loaded_jar = CookieJar::load_from_disk(&cookie_file).unwrap();
        let loaded_header = loaded_jar
            .get_cookie_header("example.com", "/", true)
            .unwrap();
        // Persistent cookie should be restored
        assert!(loaded_header.contains("auth_token=secret123"));
        // Session cookie must NOT be persisted to disk
        assert!(!loaded_header.contains("session_id=sess456"));

        // Test clear_session_cookies
        jar.clear_session_cookies();
        let post_cleanup_header = jar.get_cookie_header("example.com", "/", true).unwrap();
        assert!(post_cleanup_header.contains("auth_token=secret123"));
        assert!(!post_cleanup_header.contains("session_id=sess456"));

        let _ = fs::remove_file(cookie_file);
        let _ = fs::remove_dir(temp_dir);
    }
}
