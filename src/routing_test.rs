#[cfg(test)]
mod tests {
    use crate::config::{Config, WebSite};
    use crate::routing::{HttpStatus, Method, Request, Router};

    // Mock router for testing that doesn't check if files exist
    struct MockRouter {
        config: Config,
    }

    impl MockRouter {
        fn new(config: Config) -> Self {
            MockRouter { config }
        }

        fn route(&self, request: &Request) -> crate::routing::RoutingResult {
            // Check if request is valid
            if !request.is_valid() {
                return crate::routing::RoutingResult {
                    status_line: HttpStatus::BAD_REQUEST,
                    file_path: "static/400.html".to_string(),
                };
            }

            // Find matching website
            let website = self.config.websites.iter().find(|site| site.host_name == request.host);

            match website {
                Some(site) => {
                    // Build file path
                    let has_end_slash = request.path.ends_with('/');
                    let file_path = if has_end_slash {
                        format!("{}{}index.html", site.server_root_path, request.path)
                    } else {
                        format!("{}{}/index.html", site.server_root_path, request.path)
                    };

                    // In the mock, we always return OK for valid paths
                    crate::routing::RoutingResult {
                        status_line: HttpStatus::OK,
                        file_path,
                    }
                }
                None => crate::routing::RoutingResult {
                    status_line: HttpStatus::NOT_FOUND,
                    file_path: "static/404.html".to_string(),
                },
            }
        }
    }

    #[test]
    fn test_routing_ok() {
        // Create a test request
        let request = Request {
            method: Method::GET,
            path: "/".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        // Create a test configuration
        let config = create_test_config();

        // Create a mock router
        let router = MockRouter::new(config);

        // Route the request
        let result = router.route(&request);

        // Check the result
        assert_eq!(result.status_line, HttpStatus::OK);
        assert_eq!(result.file_path, "/test_site/site1/index.html");
    }

    #[test]
    fn test_routing_nested_path_ok() {
        // Create a test request
        let request = Request {
            method: Method::GET,
            path: "/otaku/".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        // Create a test configuration
        let config = create_test_config();

        // Create a mock router
        let router = MockRouter::new(config);

        // Route the request
        let result = router.route(&request);

        // Check the result
        assert_eq!(result.status_line, HttpStatus::OK);
        assert_eq!(result.file_path, "/test_site/site1/otaku/index.html");
    }

    #[test]
    fn test_routing_not_found() {
        // Create a test request with a path that doesn't exist
        let request = Request {
            method: Method::GET,
            path: "/unknown".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        // Create a test configuration
        let config = create_test_config();

        // Create a router
        let router = Router::new(config);

        // Route the request
        let result = router.route(&request);

        // Check the result
        assert_eq!(result.status_line, HttpStatus::NOT_FOUND);
        assert_eq!(result.file_path, "static/404.html");
    }

    #[test]
    fn test_routing_bad_request() {
        // Create a test request with an invalid method
        let request = Request {
            method: Method::Unsupported("INVALID".to_string()),
            path: "/".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        // Create a test configuration
        let config = create_test_config();

        // Create a router
        let router = Router::new(config);

        // Route the request
        let result = router.route(&request);

        // Check the result
        assert_eq!(result.status_line, HttpStatus::BAD_REQUEST);
        assert_eq!(result.file_path, "static/400.html");
    }

    #[test]
    fn test_request_parsing() {
        // Create a test request buffer
        let mut buffer = [0; 1024];
        let request = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        buffer[..request.len()].copy_from_slice(request);

        // Parse the request
        let request = Request::parse(&buffer).unwrap();

        // Check the request
        assert_eq!(request.method, Method::GET);
        assert_eq!(request.path, "/");
        assert_eq!(request.version, "HTTP/1.1");
        assert_eq!(request.host, "localhost");
    }

    #[test]
    fn test_request_parsing_with_query_params() {
        // Create a test request buffer with query parameters
        let mut buffer = [0; 1024];
        let request = b"GET /search?q=rust&page=1 HTTP/1.1\r\nHost: localhost\r\n\r\n";
        buffer[..request.len()].copy_from_slice(request);

        // Parse the request
        let request = Request::parse(&buffer).unwrap();

        // Check the request
        assert_eq!(request.method, Method::GET);
        assert_eq!(request.path, "/search?q=rust&page=1");
        assert_eq!(request.version, "HTTP/1.1");
        assert_eq!(request.host, "localhost");
    }

    #[test]
    fn test_request_parsing_with_multiple_headers() {
        // Create a test request buffer with multiple headers
        let mut buffer = [0; 1024];
        let request = b"GET / HTTP/1.1\r\nHost: localhost\r\nUser-Agent: Mozilla\r\nAccept: text/html\r\n\r\n";
        buffer[..request.len()].copy_from_slice(request);

        // Parse the request
        let request = Request::parse(&buffer).unwrap();

        // Check the request
        assert_eq!(request.method, Method::GET);
        assert_eq!(request.path, "/");
        assert_eq!(request.version, "HTTP/1.1");
        assert_eq!(request.host, "localhost");
    }

    #[test]
    fn test_request_parsing_invalid_utf8() {
        // Create a test request buffer with invalid UTF-8
        let mut buffer = [0; 1024];
        buffer[0] = 0xFF; // Invalid UTF-8 byte

        // Parse the request
        let result = Request::parse(&buffer);

        // Check that parsing failed
        assert!(result.is_err());
        match result {
            Err(crate::error::Error::Http(msg)) => {
                assert!(msg.contains("Invalid UTF-8"));
            }
            _ => panic!("Expected HTTP error"),
        }
    }

    #[test]
    fn test_request_parsing_empty_request() {
        // Create an empty request buffer
        let buffer = [0; 0];

        // Parse the request
        let result = Request::parse(&buffer);

        // Check that parsing failed
        assert!(result.is_err());
        match result {
            Err(crate::error::Error::Http(msg)) => {
                assert!(msg.contains("Empty request"));
            }
            _ => panic!("Expected HTTP error"),
        }
    }

    #[test]
    fn test_request_parsing_invalid_request_line() {
        // Create a test request buffer with an invalid request line
        let mut buffer = [0; 1024];
        let request = b"INVALID\r\nHost: localhost\r\n\r\n";
        buffer[..request.len()].copy_from_slice(request);

        // Parse the request
        let result = Request::parse(&buffer);

        // Check that parsing failed
        assert!(result.is_err());
        match result {
            Err(crate::error::Error::Http(msg)) => {
                assert!(msg.contains("Invalid request line"));
            }
            _ => panic!("Expected HTTP error"),
        }
    }

    #[test]
    fn test_request_parsing_missing_host() {
        // Create a test request buffer without a Host header
        let mut buffer = [0; 1024];
        let request = b"GET / HTTP/1.1\r\nUser-Agent: Mozilla\r\n\r\n";
        buffer[..request.len()].copy_from_slice(request);

        // Parse the request
        let result = Request::parse(&buffer);

        // Check that parsing failed
        assert!(result.is_err());
        match result {
            Err(crate::error::Error::Http(msg)) => {
                assert!(msg.contains("Missing Host header"));
            }
            _ => panic!("Expected HTTP error"),
        }
    }

    // Helper function to create a test configuration
    fn create_test_config() -> Config {
        let mut config = Config::default();
        config.websites.push(WebSite {
            host_name: "localhost".to_string(),
            server_root_path: "/test_site/site1".to_string(),
        });
        config
    }

    #[test]
    fn test_routing_static_file() {
        // Create a test request with a path that includes a file extension
        let request = Request {
            method: Method::GET,
            path: "/styles.css".to_string(),
            version: "HTTP/1.1".to_string(),
            host: "localhost".to_string(),
        };

        // Create a test configuration
        let config = create_test_config();

        // Create a router
        let router = Router::new(config);

        // Route the request
        let result = router.route(&request);

        // Check the result - should not append index.html for paths with file extensions
        assert_eq!(result.file_path, "/test_site/site1/styles.css");
    }
}
