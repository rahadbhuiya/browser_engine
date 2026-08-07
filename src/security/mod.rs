#![allow(dead_code)]

pub mod csp;
pub mod ipc;
pub mod sop;

pub use csp::CspPolicy;
pub use ipc::{IpcChannel, IpcMessage};
pub use sop::Origin;

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
}
