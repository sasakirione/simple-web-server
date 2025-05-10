use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

use log::{debug, error, info};

use crate::config::Config;
use crate::error::{Error, Result};
use crate::routing::{Request, Router};
use crate::thread_pool::ThreadPool;

/// HTTP server
pub struct Server {
    config: Config,
    router: Router,
    thread_pool: ThreadPool,
}

impl Server {
    /// Create a new server with the given configuration
    pub fn new(config: Config) -> Self {
        let router = Router::new(config.clone());
        let thread_pool = ThreadPool::new(config.num_threads);
        Server { config, router, thread_pool }
    }

    /// Determine content type based on file extension
    fn get_content_type(file_path: &str) -> &'static str {
        let path = Path::new(file_path);
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("html") => "text/html",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            Some("pdf") => "application/pdf",
            Some("txt") => "text/plain",
            _ => "application/octet-stream", // Default binary type
        }
    }

    /// Start the server
    pub fn start(&self) -> Result<()> {
        // Bind to the address
        let listener = TcpListener::bind(&self.config.server_address)
            .map_err(|e| Error::Io(e))?;

        info!("Server started at {} with {} threads", self.config.server_address, self.config.num_threads);

        // Accept connections
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let router = self.router.clone();

                    self.thread_pool.execute(move || {
                        if let Err(e) = Self::handle_connection_static(stream, &router) {
                            error!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Static method to handle a client connection
    fn handle_connection_static(mut stream: TcpStream, router: &Router) -> Result<()> {
        // Read the request
        let mut buffer = [0; 1024];
        stream.read(&mut buffer)
            .map_err(|e| Error::Io(e))?;

        // Parse the request
        let request = match Request::parse(&buffer) {
            Ok(req) => req,
            Err(e) => {
                debug!("Invalid request: {}", e);
                let response = format!("{}\r\n\r\n", "HTTP/1.1 400 BAD REQUEST");
                stream.write_all(response.as_bytes())
                    .map_err(|e| Error::Io(e))?;
                stream.flush()
                    .map_err(|e| Error::Io(e))?;
                return Ok(());
            }
        };

        // Route the request
        let routing_result = router.route(&request);

        // Determine content type based on file extension
        let content_type = Self::get_content_type(&routing_result.file_path);

        // Read the file as binary
        let contents = match fs::read(&routing_result.file_path) {
            Ok(contents) => contents,
            Err(e) => {
                error!("Error reading file {}: {}", routing_result.file_path, e);
                let response = format!("{}\r\n\r\n", "HTTP/1.1 500 INTERNAL SERVER ERROR");
                stream.write_all(response.as_bytes())
                    .map_err(|e| Error::Io(e))?;
                stream.flush()
                    .map_err(|e| Error::Io(e))?;
                return Ok(());
            }
        };

        // Build the response with Content-Type header
        let response_header = format!("{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n", 
            routing_result.status_line, 
            content_type,
            contents.len());

        // Send the response header
        stream.write_all(response_header.as_bytes())
            .map_err(|e| Error::Io(e))?;

        // Send the file content
        stream.write_all(&contents)
            .map_err(|e| Error::Io(e))?;

        stream.flush()
            .map_err(|e| Error::Io(e))?;

        debug!("Response: {}, File: {}", routing_result.status_line, routing_result.file_path);

        Ok(())
    }
}
