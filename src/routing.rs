use std::path::Path;

use crate::config::{Config, WebSite};
use crate::error::{Error, Result};

/// HTTP status codes
pub struct HttpStatus;

impl HttpStatus {
    /// 200 OK
    pub const OK: &'static str = "HTTP/1.1 200 OK";
    /// 400 Bad Request
    pub const BAD_REQUEST: &'static str = "HTTP/1.1 400 BAD REQUEST";
    /// 404 Not Found
    pub const NOT_FOUND: &'static str = "HTTP/1.1 404 NOT FOUND";
}

/// HTTP request method
#[derive(Debug, PartialEq)]
pub enum Method {
    /// GET method
    GET,
    /// Unsupported method
    Unsupported(String),
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s {
            "GET" => Method::GET,
            other => Method::Unsupported(other.to_string()),
        }
    }
}

/// HTTP request
#[derive(Debug)]
pub struct Request {
    /// HTTP method
    pub method: Method,
    /// Request a path
    pub path: String,
    /// HTTP version
    pub version: String,
    /// Host header
    pub host: String,
}

impl Request {
    /// Parse an HTTP request from a buffer
    pub fn parse(buffer: &[u8]) -> Result<Self> {
        let request_str = std::str::from_utf8(buffer)
            .map_err(|_| Error::Http("Invalid UTF-8 sequence".to_string()))?;

        let lines: Vec<&str> = request_str.lines().collect();
        if lines.is_empty() {
            return Err(Error::Http("Empty request".to_string()));
        }

        let request_line: Vec<&str> = lines[0].split_whitespace().collect();
        if request_line.len() < 3 {
            return Err(Error::Http("Invalid request line".to_string()));
        }

        let method = Method::from(request_line[0]);
        let path = request_line[1].to_string();
        let version = request_line[2].to_string();

        // Find Host header
        let mut host = String::new();
        for line in &lines[1..] {
            if line.starts_with("Host:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    host = parts[1].to_string();
                    break;
                }
            }
        }

        if host.is_empty() {
            return Err(Error::Http("Missing Host header".to_string()));
        }

        Ok(Request {
            method,
            path,
            version,
            host,
        })
    }

    /// Check if the request is valid
    pub fn is_valid(&self) -> bool {
        self.version == "HTTP/1.1" && self.method == Method::GET && !self.host.is_empty()
    }
}

/// Routing result
#[derive(Debug)]
pub struct RoutingResult {
    /// HTTP status line
    pub status_line: &'static str,
    /// File path to serve
    pub file_path: String,
}

/// Router for HTTP requests
#[derive(Clone)]
pub struct Router {
    config: Config,
}

impl Router {
    /// Create a new router with the given configuration
    pub fn new(config: Config) -> Self {
        Router { config }
    }

    /// Route an HTTP request
    pub fn route(&self, request: &Request) -> RoutingResult {
        // Check if the request is valid
        if !request.is_valid() {
            return RoutingResult {
                status_line: HttpStatus::BAD_REQUEST,
                file_path: "static/400.html".to_string(),
            };
        }

        // Find a matching website
        let website = self.find_website(&request.host);

        match website {
            Some(site) => {
                // Build a file path
                let file_path = self.build_file_path(&site, &request.path);

                // Check if a file exists
                if Path::new(&file_path).is_file() {
                    RoutingResult {
                        status_line: HttpStatus::OK,
                        file_path,
                    }
                } else {
                    RoutingResult {
                        status_line: HttpStatus::NOT_FOUND,
                        file_path: "static/404.html".to_string(),
                    }
                }
            }
            None => RoutingResult {
                status_line: HttpStatus::NOT_FOUND,
                file_path: "static/404.html".to_string(),
            },
        }
    }

    // Find website configuration by host name
    fn find_website(&self, host: &str) -> Option<&WebSite> {
        self.config.websites.iter().find(|site| site.host_name == host)
    }

    // Build a file path from website configuration and request path
    fn build_file_path(&self, website: &WebSite, path: &str) -> String {
        let has_end_slash = path.ends_with('/');

        if has_end_slash {
            format!("{}{}index.html", website.server_root_path, path)
        } else {
            format!("{}{}/index.html", website.server_root_path, path)
        }
    }
}
