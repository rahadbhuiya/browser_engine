use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Cookie {
    pub domain: String,
    pub name: String,
    pub value: String,
}

pub struct StorageManager {
    pub local_storage: HashMap<String, HashMap<String, String>>,
    pub cookies: Vec<Cookie>,
}

impl StorageManager {
    pub fn new() -> Self {
        StorageManager {
            local_storage: HashMap::new(),
            cookies: Vec::new(),
        }
    }

    pub fn set_item(&mut self, domain: &str, key: &str, value: &str) {
        self.local_storage
            .entry(domain.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value.to_string());
    }

    pub fn get_item(&self, domain: &str, key: &str) -> Option<&String> {
        self.local_storage.get(domain)?.get(key)
    }

    pub fn set_cookie(&mut self, domain: &str, name: &str, value: &str) {
        if let Some(cookie) = self.cookies.iter_mut().find(|c| c.domain == domain && c.name == name) {
            cookie.value = value.to_string();
        } else {
            self.cookies.push(Cookie {
                domain: domain.to_string(),
                name: name.to_string(),
                value: value.to_string(),
            });
        }
    }

    pub fn get_cookies(&self, domain: &str) -> String {
        let matching: Vec<String> = self.cookies
            .iter()
            .filter(|c| c.domain == domain)
            .map(|c| format!("{}={}", c.name, c.value))
            .collect();
        matching.join("; ")
    }
}
