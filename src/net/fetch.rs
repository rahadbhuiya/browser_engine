use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, StreamOwned};

use super::http::{HttpRequest, HttpResponse};
use super::url_parser::{ParsedUrl, Scheme};

pub fn fetch(url_str: &str) -> Result<HttpResponse, String> {
    let parsed_url = ParsedUrl::parse(url_str)?;
    let request = HttpRequest::new_get(&parsed_url.host, &parsed_url.path_and_query);
    let request_bytes = request.serialize().into_bytes();

    let addr = format!("{}:{}", parsed_url.host, parsed_url.port);
    let tcp_stream = TcpStream::connect(&addr).map_err(|e| format!("TCP connection to {} failed: {}", addr, e))?;

    let response_bytes = match parsed_url.scheme {
        Scheme::Http => {
            let mut stream = tcp_stream;
            stream.write_all(&request_bytes).map_err(|e| format!("Write failed: {}", e))?;
            let mut buf = Vec::new();
            stream.read_to_end(&mut buf).map_err(|e| format!("Read failed: {}", e))?;
            buf
        }
        Scheme::Https => {
            let root_store = rustls::RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.iter().cloned().collect(),
            };

            let config = ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();

            let server_name = ServerName::try_from(parsed_url.host.clone())
                .map_err(|_| format!("Invalid ServerName: {}", parsed_url.host))?;

            let conn = ClientConnection::new(Arc::new(config), server_name)
                .map_err(|e| format!("TLS Connection init failed: {}", e))?;

            let mut tls_stream = StreamOwned::new(conn, tcp_stream);
            tls_stream.write_all(&request_bytes).map_err(|e| format!("TLS Write failed: {}", e))?;
            let mut buf = Vec::new();
            tls_stream.read_to_end(&mut buf).map_err(|e| format!("TLS Read failed: {}", e))?;
            buf
        }
    };

    HttpResponse::parse(&response_bytes)
}
