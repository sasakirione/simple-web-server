# Rust Web Server Development Guidelines

This document provides guidelines and information for developing and maintaining the Rust Web Server project.

## Build/Configuration Instructions

### Prerequisites
- Rust (latest stable version)
- Cargo (included with Rust)

### Building the Project
1. Clone the repository
2. Build the project:
   ```bash
   cargo build
   ```
3. Run the project:
   ```bash
   cargo run
   ```
   
   Or with a specific configuration file:
   ```bash
   cargo run -- test_setting.yaml
   ```

### Configuration
The server uses YAML configuration files to define website settings. The configuration format is:

```yaml
web_site:
  - host_name: example.com
    server_root_path: /path/to/site1
  - host_name: another-example.com
    server_root_path: /path/to/site2
```

Each entry in the `web_site` array defines:
- `host_name`: The domain name for the website
- `server_root_path`: The file system path to the website's root directory

The server will serve files from the specified root directory based on the host name in the HTTP request.

## Testing Information

### Running Tests
To run all tests:
```bash
cargo test
```

To run ignored tests:
```bash
cargo test -- --ignored
```

To run a specific test:
```bash
cargo test test_name
```

### Adding Tests
Tests are written using Rust's built-in testing framework. There are two ways to add tests:

1. **Unit tests in the same file as the code**:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_function_name() {
           // Test code here
           assert_eq!(expected, actual);
       }
   }
   ```

2. **Tests in separate files**:
   - Create a new file in the `src` directory (e.g., `test_module.rs`)
   - Add the test module with the `#[cfg(test)]` attribute
   - Include the module in `main.rs` with `#[cfg(test)] mod test_module;`

### Test Example
Here's a simple test example that demonstrates testing YAML parsing:

```rust
#[cfg(test)]
mod test_example {
    use yaml_rust2::YamlLoader;
    
    #[test]
    fn test_yaml_parsing() {
        let yaml_str = r#"
        key1: value1
        key2: 42
        "#;
        
        let docs = YamlLoader::load_from_str(yaml_str).expect("Failed to parse YAML");
        
        assert_eq!(docs[0]["key1"].as_str().unwrap(), "value1");
        assert_eq!(docs[0]["key2"].as_i64().unwrap(), 42);
    }
}
```

## Additional Development Information

### Project Structure
- `src/main.rs`: Main application code
- `static/`: Static HTML files for error responses (400.html, 404.html)
- `test_site/`: Example website directories for testing

### Error Handling
The server currently uses `unwrap()` in many places, which will panic on errors. For production code, consider using proper error handling with `Result` types and error propagation.

### Logging
The project uses the `log` and `env_logger` crates for logging. Log levels can be controlled through the `ECHIZEN_S` environment variable, which is set to "trace" by default in the code.

### HTTP Request Handling
The server only supports HTTP GET requests. The request validation is performed in the `is_valid_request` function, which checks:
1. HTTP version is 1.1
2. Request includes a Host header
3. Method is GET

### Routing
The server routes requests based on:
1. The host name in the request
2. The path in the request

Files are served from the configured server root path for the matching host name.

### Future Improvements
1. Add support for other HTTP methods (POST, PUT, DELETE)
2. Implement proper error handling instead of using `unwrap()`
3. Add support for HTTPS
4. Implement request logging
5. Add support for dynamic content (e.g., server-side rendering)