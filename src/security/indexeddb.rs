use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ObjectStore {
    pub name: String,
    pub records: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct IndexedDbDatabase {
    pub name: String,
    pub version: u32,
    pub stores: HashMap<String, ObjectStore>,
}

pub struct IndexedDbManager {
    pub databases: HashMap<String, HashMap<String, IndexedDbDatabase>>,
}

impl IndexedDbManager {
    pub fn new() -> Self {
        IndexedDbManager {
            databases: HashMap::new(),
        }
    }

    pub fn open_db(&mut self, domain: &str, db_name: &str, version: u32) {
        let domain_dbs = self.databases.entry(domain.to_string()).or_insert_with(HashMap::new);
        domain_dbs.entry(db_name.to_string()).or_insert_with(|| IndexedDbDatabase {
            name: db_name.to_string(),
            version,
            stores: HashMap::new(),
        });
    }

    pub fn create_object_store(&mut self, domain: &str, db_name: &str, store_name: &str) {
        if let Some(domain_dbs) = self.databases.get_mut(domain) {
            if let Some(db) = domain_dbs.get_mut(db_name) {
                db.stores.entry(store_name.to_string()).or_insert_with(|| ObjectStore {
                    name: store_name.to_string(),
                    records: HashMap::new(),
                });
            }
        }
    }

    pub fn put(&mut self, domain: &str, db_name: &str, store_name: &str, key: &str, val: &str) {
        if let Some(domain_dbs) = self.databases.get_mut(domain) {
            if let Some(db) = domain_dbs.get_mut(db_name) {
                if let Some(store) = db.stores.get_mut(store_name) {
                    store.records.insert(key.to_string(), val.to_string());
                }
            }
        }
    }

    pub fn get(&self, domain: &str, db_name: &str, store_name: &str, key: &str) -> Option<&String> {
        self.databases.get(domain)?.get(db_name)?.stores.get(store_name)?.records.get(key)
    }
}
