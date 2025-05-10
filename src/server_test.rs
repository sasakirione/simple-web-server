#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll};

    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

    use crate::config::{Config, WebSite};
    use crate::error::{Error, Result};
    use crate::routing::{HttpStatus, Request, Router};
    use crate::server::Server;

    // Mock TcpStream for testing
    struct MockTcpStream {
        read_data: Vec<u8>,
        read_pos: usize,
        write_data: Arc<Mutex<Vec<u8>>>,
    }

    impl MockTcpStream {
        fn new(read_data: Vec<u8>) -> Self {
            MockTcpStream {
                read_data,
                read_pos: 0,
                write_data: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_written_data(&self) -> Vec<u8> {
            self.write_data.lock().unwrap().clone()
        }
    }

    impl AsyncRead for MockTcpStream {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            let remaining = &self.read_data[self.read_pos..];
            let to_read = std::cmp::min(remaining.len(), buf.remaining());

            if to_read > 0 {
                buf.put_slice(&remaining[..to_read]);
                self.read_pos += to_read;
            }

            Poll::Ready(Ok(()))
        }
    }

    impl AsyncWrite for MockTcpStream {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            self.write_data.lock().unwrap().extend_from_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    // Test helper function that mimics the server's handle_connection function
    async fn test_handle_connection(stream: &mut MockTcpStream, router: &Router) -> Result<()> {
        // Read the request
        let mut buffer = [0; 1024];
        let mut read_buf = ReadBuf::new(&mut buffer);
        Pin::new(&mut *stream).poll_read(&mut Context::from_waker(futures::task::noop_waker_ref()), &mut read_buf).map_err(|e| Error::Io(e))?;
        let n = read_buf.filled().len();

        // Parse the request
        let request = match Request::parse(&buffer[..n]) {
            Ok(req) => req,
            Err(_) => {
                let response = format!("{}\r\n\r\n", "HTTP/1.1 400 BAD REQUEST");
                Pin::new(&mut *stream).poll_write(&mut Context::from_waker(futures::task::noop_waker_ref()), response.as_bytes()).map_err(|e| Error::Io(e))?;
                Pin::new(&mut *stream).poll_flush(&mut Context::from_waker(futures::task::noop_waker_ref())).map_err(|e| Error::Io(e))?;
                return Ok(());
            }
        };

        // Route the request
        let routing_result = router.route(&request);

        // For testing, we'll just return the status line without reading a file
        let contents = "Test content";

        // Build the response
        let response = format!("{}\r\n\r\n{}", routing_result.status_line, contents);

        // Send the response
        Pin::new(&mut *stream).poll_write(&mut Context::from_waker(futures::task::noop_waker_ref()), response.as_bytes()).map_err(|e| Error::Io(e))?;
        Pin::new(&mut *stream).poll_flush(&mut Context::from_waker(futures::task::noop_waker_ref())).map_err(|e| Error::Io(e))?;

        Ok(())
    }

    #[tokio::test]
    async fn test_handle_connection_valid_request() {
        // Create a mock request
        let request = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let mut stream = MockTcpStream::new(request.to_vec());

        // Create a test configuration
        let mut config = Config::default();
        config.websites.push(WebSite {
            host_name: "localhost".to_string(),
            server_root_path: "test_site/site1".to_string(),
        });

        // Create a router
        let router = Router::new(config);

        // Handle the connection
        let result = test_handle_connection(&mut stream, &router).await;

        // Check that the connection was handled successfully
        assert!(result.is_ok());

        // Check the response
        let written_data = stream.get_written_data();
        let response = String::from_utf8_lossy(&written_data);

        // The response should contain the status line
        assert!(response.contains(HttpStatus::OK) || response.contains(HttpStatus::NOT_FOUND));
    }

    #[tokio::test]
    async fn test_handle_connection_invalid_request() {
        // Create an invalid request (missing Host header)
        let request = b"GET / HTTP/1.1\r\n\r\n";
        let mut stream = MockTcpStream::new(request.to_vec());

        // Create a test configuration
        let config = Config::default();

        // Create a router
        let router = Router::new(config);

        // Handle the connection
        let result = test_handle_connection(&mut stream, &router).await;

        // Check that the connection was handled successfully (even though the request was invalid)
        assert!(result.is_ok());

        // Check the response
        let written_data = stream.get_written_data();
        let response = String::from_utf8_lossy(&written_data);

        // The response should contain a 400 Bad Request status
        assert!(response.contains("400 BAD REQUEST"));
    }

    #[test]
    fn test_server_creation() {
        // Create a test configuration
        let config = Config::default();

        // Create a server
        let _server = Server::new(config);

        // Just testing that it doesn't panic
        assert!(true);
    }
}
