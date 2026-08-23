use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub url: String,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct CacheStorageManager {
    pub caches: HashMap<String, HashMap<String, HashMap<String, CacheEntry>>>,
}

impl CacheStorageManager {
    pub fn new() -> Self {
        CacheStorageManager {
            caches: HashMap::new(),
        }
    }

    pub fn put(&mut self, domain: &str, cache_name: &str, url: &str, body: &[u8]) {
        let domain_caches = self.caches.entry(domain.to_string()).or_insert_with(HashMap::new);
        let cache = domain_caches.entry(cache_name.to_string()).or_insert_with(HashMap::new);
        cache.insert(url.to_string(), CacheEntry {
            url: url.to_string(),
            body: body.to_vec(),
        });
    }

    pub fn match_url(&self, domain: &str, cache_name: &str, url: &str) -> Option<&CacheEntry> {
        self.caches.get(domain)?.get(cache_name)?.get(url)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServiceWorkerRegistration {
    pub scope: String,
    pub script_url: String,
    pub active: bool,
}

pub struct ServiceWorkerManager {
    pub registrations: Vec<ServiceWorkerRegistration>,
}

impl ServiceWorkerManager {
    pub fn new() -> Self {
        ServiceWorkerManager { registrations: Vec::new() }
    }

    pub fn register(&mut self, scope: &str, script_url: &str) -> ServiceWorkerRegistration {
        let reg = ServiceWorkerRegistration {
            scope: scope.to_string(),
            script_url: script_url.to_string(),
            active: true,
        };
        println!("ServiceWorker registered: [{}] -> {}", scope, script_url);
        self.registrations.push(reg.clone());
        reg
    }
}
