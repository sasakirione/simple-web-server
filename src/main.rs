use std::net::TcpListener;
use std::io::{Read, Write};
use std::fs;

fn main() {
    // ソケットをバインドして待ち受け
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server is running on port 7878");

    // 接続の受け入れと処理
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: std::net::TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // レスポンスの準備
    let (status_line, filename) = get_routing_file(&mut buffer);

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
static BAD_REQUEST: &str = "HTTP/1.1 400 BAD REQUEST\r\n\r\n";
static NOT_FOUND: &str = "HTTP/1.1 200 OK\r\n\r\n";
static OK: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";

fn get_routing_file(buffer: &mut [u8; 1024]) -> (&str, &str) {
    let request_str = std::str::from_utf8(buffer).expect("Invalid UTF-8 sequence");
    let parts: Vec<&str> = request_str.split_whitespace().collect();
    if parts?[2] != "HTTP/1.1" {
        return (BAD_REQUEST, "static/400.html")
    }
    match (parts?[0], parts?[1]) {
        ("GET", "/") => (OK, "static/hello.html"),
        _ => (NOT_FOUND, "static/404.html")
    }
}

