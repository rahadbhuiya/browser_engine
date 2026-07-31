use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub host: String,
    pub headers: HashMap<String, String>,
}

impl HttpRequest {
    pub fn new_get(host: &str, path: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Host".to_string(), host.to_string());
        headers.insert("User-Agent".to_string(), "SecureBrowser/1.0".to_string());
        headers.insert("Accept".to_string(), "text/html,text/css,*/*".to_string());
        headers.insert("Connection".to_string(), "close".to_string());

        HttpRequest {
            method: "GET".to_string(),
            path: path.to_string(),
            host: host.to_string(),
            headers,
        }
    }

    pub fn serialize(&self) -> String {
        let mut req = format!("{} {} HTTP/1.1\r\n", self.method, self.path);
        for (k, v) in &self.headers {
            req.push_str(&format!("{}: {}\r\n", k, v));
        }
        req.push_str("\r\n");
        req
    }
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn parse(raw_data: &[u8]) -> Result<Self, String> {
        let header_end = raw_data
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .ok_or_else(|| "Malformed HTTP response: missing header end".to_string())?;

        let header_bytes = &raw_data[..header_end];
        let body_bytes = &raw_data[header_end + 4..];

        let header_str = String::from_utf8_lossy(header_bytes);
        let mut lines = header_str.lines();

        let status_line = lines
            .next()
            .ok_or_else(|| "Empty status line".to_string())?;
        let parts: Vec<&str> = status_line.splitn(3, ' ').collect();
        if parts.len() < 2 {
            return Err("Invalid status line format".to_string());
        }

        let status_code = parts[1]
            .parse::<u16>()
            .map_err(|e| format!("Invalid status code: {}", e))?;
        let status_text = if parts.len() > 2 { parts[2] } else { "" }.to_string();

        let mut headers = HashMap::new();
        for line in lines {
            if let Some((k, v)) = line.split_once(':') {
                headers.insert(k.trim().to_lowercase(), v.trim().to_string());
            }
        }

        Ok(HttpResponse {
            status_code,
            status_text,
            headers,
            body: body_bytes.to_vec(),
        })
    }

    pub fn body_as_string(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
}
