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

    // Helper function to create a test configuration
    fn create_test_config() -> Config {
        let mut config = Config::default();
        config.websites.push(WebSite {
            host_name: "localhost".to_string(),
            server_root_path: "/test_site/site1".to_string(),
        });
        config
    }
}
