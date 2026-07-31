use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub enum Scheme {
    Http,
    Https,
}

#[derive(Debug, Clone)]
pub struct ParsedUrl {
    pub scheme: Scheme,
    pub host: String,
    pub port: u16,
    pub path_and_query: String,
}

impl ParsedUrl {
    pub fn parse(input: &str) -> Result<Self, String> {
        let parsed = Url::parse(input).map_err(|e| format!("Invalid URL: {}", e))?;
        let scheme = match parsed.scheme() {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
            s => return Err(format!("Unsupported scheme: {}", s)),
        };

        let host = parsed
            .host_str()
            .ok_or_else(|| "URL missing host".to_string())?
            .to_string();

        let port = parsed.port_or_known_default().unwrap_or(match scheme {
            Scheme::Http => 80,
            Scheme::Https => 443,
        });

        let mut path_and_query = parsed.path().to_string();
        if path_and_query.is_empty() {
            path_and_query = "/".to_string();
        }
        if let Some(query) = parsed.query() {
            path_and_query.push('?');
            path_and_query.push_str(query);
        }

        Ok(ParsedUrl {
            scheme,
            host,
            port,
            path_and_query,
        })
    }
}
