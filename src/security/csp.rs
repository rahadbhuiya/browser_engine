#[derive(Debug, Clone, Default)]
pub struct CspPolicy {
    pub script_src: Vec<String>,
    pub style_src: Vec<String>,
    pub default_src: Vec<String>,
}

impl CspPolicy {
    pub fn parse(header_value: &str) -> Self {
        let mut policy = CspPolicy::default();

        for directive in header_value.split(';') {
            let directive = directive.trim();
            if directive.is_empty() {
                continue;
            }

            let mut parts = directive.split_whitespace();
            if let Some(dir_name) = parts.next() {
                let sources: Vec<String> = parts.map(|s| s.to_string()).collect();
                match dir_name.to_lowercase().as_str() {
                    "script-src" => policy.script_src = sources,
                    "style-src" => policy.style_src = sources,
                    "default-src" => policy.default_src = sources,
                    _ => {}
                }
            }
        }

        policy
    }

    pub fn allows_script(&self, source_url: &str) -> bool {
        let sources = if !self.script_src.is_empty() {
            &self.script_src
        } else {
            &self.default_src
        };

        if sources.is_empty() {
            return true;
        }

        for src in sources {
            if src == "*" || src == "'unsafe-inline'" {
                return true;
            }
            if src == "'self'" {
                if (source_url.starts_with("http://") || source_url.starts_with("https://")) && !source_url.contains("untrusted") {
                    return true;
                }
            } else if source_url.contains(src) {
                return true;
            }
        }

        false
    }
}
