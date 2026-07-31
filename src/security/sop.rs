#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Origin {
    pub scheme: String,
    pub host: String,
    pub port: u16,
}

impl Origin {
    pub fn new(scheme: &str, host: &str, port: u16) -> Self {
        Origin {
            scheme: scheme.to_lowercase(),
            host: host.to_lowercase(),
            port,
        }
    }

    pub fn parse(url_str: &str) -> Option<Self> {
        if let Ok(parsed) = url::Url::parse(url_str) {
            let host = parsed.host_str()?.to_string();
            let port = parsed.port_or_known_default()?;
            Some(Origin::new(parsed.scheme(), &host, port))
        } else {
            None
        }
    }

    pub fn is_same_origin(&self, other: &Origin) -> bool {
        self.scheme == other.scheme && self.host == other.host && self.port == other.port
    }
}
