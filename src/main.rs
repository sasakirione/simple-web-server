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

fn get_routing_file(buffer: &mut [u8; 1024]) -> (&str, &str) {
    let get = b"GET / HTTP/1.1\r\n";

    if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "static/hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "static/404.html")
    }
}

