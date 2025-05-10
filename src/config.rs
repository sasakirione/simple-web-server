use std::fs;
use yaml_rust2::YamlLoader;

use crate::error::{Error, Result};

/// Configuration for a website
#[derive(Debug, Clone)]
pub struct WebSite {
    /// The domain name for the website
    pub host_name: String,
    /// The file system path to the website's root directory
    pub server_root_path: String,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// List of websites to serve
    pub websites: Vec<WebSite>,
    /// Server address to bind to
    pub server_address: String,
    /// Number of threads in the thread pool
    pub num_threads: usize,
}

impl Config {
    /// Create a new default configuration
    pub fn default() -> Self {
        // Default to number of CPU cores or 4 if that can't be determined
        let default_threads = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);

        Config {
            websites: Vec::new(),
            server_address: "127.0.0.1:7878".to_string(),
            num_threads: default_threads,
        }
    }

    /// Load configuration from a YAML file
    pub fn from_file(path: &str) -> Result<Self> {
        let config_str = fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;

        Self::from_yaml_str(&config_str)
    }

    /// Load configuration from a YAML string
    pub fn from_yaml_str(yaml_str: &str) -> Result<Self> {
        let docs = YamlLoader::load_from_str(yaml_str)
            .map_err(|e| Error::YamlParse(format!("Failed to parse YAML: {}", e)))?;

        if docs.is_empty() {
            return Ok(Config::default());
        }

        let mut config = Config::default();

        // Parse websites
        if let Some(websites) = docs[0]["web_site"].as_vec() {
            for site in websites {
                let host_name = site["host_name"].as_str()
                    .ok_or_else(|| Error::Config("Missing host_name in website config".to_string()))?
                    .to_string();

                let server_root_path = site["server_root_path"].as_str()
                    .ok_or_else(|| Error::Config("Missing server_root_path in website config".to_string()))?
                    .to_string();

                config.websites.push(WebSite {
                    host_name,
                    server_root_path,
                });
            }
        }

        // Parse num_threads if present
        if let Some(num_threads) = docs[0]["num_threads"].as_i64() {
            if num_threads > 0 {
                config.num_threads = num_threads as usize;
            } else {
                return Err(Error::Config("num_threads must be greater than 0".to_string()));
            }
        }

        Ok(config)
    }
}
