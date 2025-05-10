#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::error::Error;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.websites.is_empty());
        assert_eq!(config.server_address, "127.0.0.1:7878");
        assert!(config.num_threads > 0);
    }

    #[test]
    fn test_from_yaml_str_empty() {
        let yaml_str = "";
        let config = Config::from_yaml_str(yaml_str).unwrap();
        assert!(config.websites.is_empty());
        assert_eq!(config.server_address, "127.0.0.1:7878");
    }

    #[test]
    fn test_from_yaml_str_valid() {
        let yaml_str = r#"
        web_site:
          - host_name: example.com
            server_root_path: /path/to/site1
          - host_name: another-example.com
            server_root_path: /path/to/site2
        num_threads: 8
        "#;

        let config = Config::from_yaml_str(yaml_str).unwrap();

        assert_eq!(config.websites.len(), 2);
        assert_eq!(config.websites[0].host_name, "example.com");
        assert_eq!(config.websites[0].server_root_path, "/path/to/site1");
        assert_eq!(config.websites[1].host_name, "another-example.com");
        assert_eq!(config.websites[1].server_root_path, "/path/to/site2");
        assert_eq!(config.num_threads, 8);
    }

    #[test]
    fn test_from_yaml_str_missing_host_name() {
        let yaml_str = r#"
        web_site:
          - server_root_path: /path/to/site1
        "#;

        let result = Config::from_yaml_str(yaml_str);
        assert!(result.is_err());

        match result {
            Err(Error::Config(msg)) => {
                assert!(msg.contains("Missing host_name"));
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_from_yaml_str_missing_server_root_path() {
        let yaml_str = r#"
        web_site:
          - host_name: example.com
        "#;

        let result = Config::from_yaml_str(yaml_str);
        assert!(result.is_err());

        match result {
            Err(Error::Config(msg)) => {
                assert!(msg.contains("Missing server_root_path"));
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_from_yaml_str_invalid_num_threads() {
        let yaml_str = r#"
        web_site:
          - host_name: example.com
            server_root_path: /path/to/site1
        num_threads: 0
        "#;

        let result = Config::from_yaml_str(yaml_str);
        assert!(result.is_err());

        match result {
            Err(Error::Config(msg)) => {
                assert!(msg.contains("num_threads must be greater than 0"));
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_from_yaml_str_invalid_yaml() {
        let yaml_str = "invalid: yaml: format:";

        let result = Config::from_yaml_str(yaml_str);
        assert!(result.is_err());

        match result {
            Err(Error::YamlParse(_)) => {
                // Expected error type
            }
            _ => panic!("Expected YamlParse error"),
        }
    }
}
