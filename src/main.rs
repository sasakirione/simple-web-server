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

