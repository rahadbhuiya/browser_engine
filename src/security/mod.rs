#![allow(dead_code)]

pub mod csp;
pub mod indexeddb;
pub mod ipc;
pub mod sop;
pub mod storage;

pub use csp::CspPolicy;
pub use indexeddb::{IndexedDbDatabase, IndexedDbManager, ObjectStore};
pub use ipc::{IpcChannel, IpcMessage};
pub use sop::Origin;
pub use storage::{Cookie, StorageManager};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_communication() {
        let (ep1, ep2) = IpcChannel::new();
        ep1.send(IpcMessage::FetchUrl { url: "http://example.com".to_string() }).unwrap();
        let msg = ep2.try_recv().unwrap();
        assert_eq!(msg, IpcMessage::FetchUrl { url: "http://example.com".to_string() });
    }

    #[test]
    fn test_same_origin_policy() {
        let o1 = Origin::parse("https://example.com:443/page1").unwrap();
        let o2 = Origin::parse("https://example.com:443/page2").unwrap();
        let o3 = Origin::parse("http://example.com:80/page1").unwrap();
        let o4 = Origin::parse("https://api.example.com:443/page1").unwrap();

        assert!(o1.is_same_origin(&o2));
        assert!(!o1.is_same_origin(&o3)); // different scheme
        assert!(!o1.is_same_origin(&o4)); // different host
    }

    #[test]
    fn test_csp_script_policy() {
        let policy = CspPolicy::parse("script-src 'self' https://trusted.com; style-src 'unsafe-inline'");
        assert!(policy.allows_script("https://example.com/app.js"));
        assert!(policy.allows_script("https://trusted.com/library.js"));
        assert!(!policy.allows_script("http://untrusted-site.com/malicious.js"));
    }

    #[test]
    fn test_storage_manager() {
        let mut sm = StorageManager::new();
        sm.set_item("example.com", "theme", "dark");
        assert_eq!(sm.get_item("example.com", "theme"), Some(&"dark".to_string()));

        sm.set_cookie("example.com", "session_id", "xyz123");
        assert_eq!(sm.get_cookies("example.com"), "session_id=xyz123");
    }

    #[test]
    fn test_indexeddb_transactional_store() {
        let mut db_mgr = IndexedDbManager::new();
        db_mgr.open_db("example.com", "MyDatabase", 1);
        db_mgr.create_object_store("example.com", "MyDatabase", "users");
        db_mgr.put("example.com", "MyDatabase", "users", "user_101", "{\"name\": \"Diaz\"}");

        let record = db_mgr.get("example.com", "MyDatabase", "users", "user_101");
        assert_eq!(record, Some(&"{\"name\": \"Diaz\"}".to_string()));
    }
}
