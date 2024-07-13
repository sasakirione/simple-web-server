use std::net::TcpListener;
use std::io::{Read, Write};
use std::{env, fs};
use std::sync::Mutex;
use yaml_rust2::{Yaml, YamlLoader};

static SETTING: Mutex<Vec<Yaml>> = Mutex::new(Vec::new());

fn main() {
    let args: Vec<String> = env::args().collect();
    // 設定ファイルを読み込む
    let mut docs = if args.iter().any(|x| x.contains("yaml")) {
        println!("設定ファイルを読み込みます");
        let config_file = fs::read_to_string(args.iter().find(|x| x.contains("yaml")).unwrap()).expect("設定ファイルの読み込みに失敗しました");
        let config_file2 = config_file.as_str();
        YamlLoader::load_from_str(&config_file2).expect("設定ファイルの読み込みに失敗しました")
    } else {
        println!("デフォルト設定を読み込みます");
        YamlLoader::load_from_str("").expect("設定ファイルの読み込みに失敗しました")
    };

    {
        let mut setting = SETTING.lock().expect("設定ファイルの読み込みに失敗しました");
        setting.append(&mut docs);
    }

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

    let response = format!("{}\r\n\r\n{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
static BAD_REQUEST: &str = "HTTP/1.1 400 BAD REQUEST";
static NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND";
static OK: &str = "HTTP/1.1 200 OK";

fn get_routing_file(buffer: &mut [u8; 1024]) -> (&str, String) {
    let request_str = std::str::from_utf8(buffer).expect("Invalid UTF-8 sequence");
    println!("{}", request_str);
    let parts: Vec<&str> = request_str.split_whitespace().collect();
    if parts[2] != "HTTP/1.1" {
        return (BAD_REQUEST, "static/400.html".to_string())
    }
    if parts[3] != "Host:" {
        return (BAD_REQUEST, "static/400.html".to_string());
    }
    let host = parts[4];
    let setting = SETTING.lock().expect("設定ファイルの読み込みに失敗しました");
    // よくわからない！
    let server_path: &str = setting.first()
        .and_then(|setting| setting["web_site"].as_vec())
        .and_then(|hosts| hosts.iter().find(|&x| x["host_name"].as_str() == Option::from(host)))
        .map(|hosts| hosts["server_root_path"].as_str().unwrap()).unwrap();
    println!("{}", server_path);

    match (parts[0], parts[1]) {
        ("GET", "/") => (OK, server_path.to_string() + "/hello.html"),
        _ => (NOT_FOUND, "static/404.html".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_get_routing_file_ok() {
        let mut buffer = [0; 1024];
        let request = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        buffer[..request.len()].copy_from_slice(request);

        let setting = YamlLoader::load_from_str(r#"
            web_site:
              - host_name: localhost
                server_root_path: "/test"
        "#).unwrap();

        {
            let mut global_setting = SETTING.lock().unwrap();
            global_setting.clear();
            global_setting.extend(setting);
        }

        let (status_line, filename) = get_routing_file(&mut buffer);
        assert_eq!(status_line, OK);
        assert_eq!(filename, "/test/hello.html");
    }

    #[test]
    fn test_get_routing_file_not_found() {
        let mut buffer = [0; 1024];
        let request = b"GET /unknown HTTP/1.1\r\nHost: localhost\r\n\r\n";
        buffer[..request.len()].copy_from_slice(request);

        let setting = YamlLoader::load_from_str(r#"
            web_site:
              - host_name: localhost
                server_root_path: "/test"
        "#).unwrap();

        {
            let mut global_setting = SETTING.lock().unwrap();
            global_setting.clear();
            global_setting.extend(setting);
        }

        let (status_line, filename) = get_routing_file(&mut buffer);
        assert_eq!(status_line, NOT_FOUND);
        assert_eq!(filename, "static/404.html");
    }

    #[test]
    fn test_get_routing_file_bad_request() {
        let mut buffer = [0; 1024];
        let request = b"INVALID_REQUEST / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        buffer[..request.len()].copy_from_slice(request);

        let (status_line, filename) = get_routing_file(&mut buffer);
        assert_eq!(status_line, BAD_REQUEST);
        assert_eq!(filename, "static/400.html");
    }

    #[test]
    fn test_handle_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                let stream = stream.unwrap();
                handle_connection(stream);
            }
        });

        thread::sleep(Duration::from_millis(100));

        let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        let request = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        stream.write_all(request).unwrap();

        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();

        let response = std::str::from_utf8(&buffer).unwrap();
        assert!(response.contains("HTTP/1.1 200 OK"));

        handle.join().unwrap();
    }
}

