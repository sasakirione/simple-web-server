#[macro_use]
extern crate log;
extern crate env_logger as logger;

use std::{env, fs};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::Mutex;

use yaml_rust2::{Yaml, YamlLoader};
use log::Level;

static SETTING: Mutex<Vec<Yaml>> = Mutex::new(Vec::new());

fn main() {
    env::set_var("ECHIZEN_S", "trace");
    logger::init();

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
    info!("start poling at 127.0.0.1:7878");

    // 接続の受け入れと処理
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

/// HTTPリクエストに対して具体的な処理を行う
///
/// # Arguments
/// * `stream` - TCPのストリーム
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

/// HTTPリクエストからレスポンスを生成する
///
/// # Arguments
/// * `buffer` - HTTPリクエストのByte配列
///
/// # Returns
/// * `status_line` - レスポンスの1行目
/// * `filename` - 本文として返答するファイルのパス
fn get_routing_file(buffer: &mut [u8; 1024]) -> (&str, String) {
    let request_str = std::str::from_utf8(buffer).expect("Invalid UTF-8 sequence");
    println!("{}", request_str);
    let parts: Vec<&str> = request_str.split_whitespace().collect();
    if !is_valid_request(&parts) {
        debug!("Response: 400");
        return (BAD_REQUEST, "static/400.html".to_string())
    }
    let host = parts[4];
    let setting = SETTING.lock().expect("設定ファイルの読み込みに失敗しました");
    // よくわからない！
    let server_path: &str = setting.first()
        .and_then(|setting| setting["web_site"].as_vec())
        .and_then(|hosts| hosts.iter().find(|&x| x["host_name"].as_str() == Option::from(host)))
        .map(|hosts| hosts["server_root_path"].as_str().unwrap()).unwrap();

    let has_end_slash = parts[1].ends_with("/");
    let path = if has_end_slash
        { format!("{}{}index.html", server_path, parts[1]) } else
        { format!("{}{}/index.html", server_path, parts[1]) };

    if Path::new(&path).is_file(){
        debug!("Method: GET, Routing Path: {}, Response: 200", path);
        return (OK, path);
    }
    debug!("Method: GET, Routing Path: {}, Response: 404", path);
    return (NOT_FOUND, "static/404.html".to_string())
}

/// パースされたリクエストが正しい形式かチェックする
///
/// # Arguments
/// * `parts` - HTTPリクエストの文字列を半角スペースで分割したもの
///
/// # Returns
/// * `res` - HTTPリクエストとしてバリデートされていればtrueを返す
fn is_valid_request(parts: &Vec<&str>) -> bool {
    if parts[2] != "HTTP/1.1" {
        return false
    }
    if parts[3] != "Host:" {
        return false
    }
    if parts[0] != "GET" {
        return false
    }
    return true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_get_routing_file_ok() {
        let mut buffer = [0; 1024];
        let request = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        buffer[..request.len()].copy_from_slice(request);

        set_setting();

        let (status_line, filename) = get_routing_file(&mut buffer);
        assert_eq!(status_line, OK);
        assert_eq!(filename, "/test/index.html");
    }

    #[test]
    #[ignore]
    fn test_get_routing_nest_file_ok() {
        let mut buffer = [0; 1024];
        let request = b"GET /otaku/ HTTP/1.1\r\nHost: localhost\r\n\r\n";
        buffer[..request.len()].copy_from_slice(request);

        set_setting();

        let (status_line, filename) = get_routing_file(&mut buffer);
        assert_eq!(status_line, OK);
        assert_eq!(filename, "/test/otaku/index.html");
    }

    #[test]
    fn test_get_routing_file_not_found() {
        let mut buffer = [0; 1024];
        let request = b"GET /unknown HTTP/1.1\r\nHost: localhost\r\n\r\n";
        buffer[..request.len()].copy_from_slice(request);

        set_setting();

        let (status_line, filename) = get_routing_file(&mut buffer);
        assert_eq!(status_line, NOT_FOUND);
        assert_eq!(filename, "static/404.html");
    }

    #[test]
    fn test_get_routing_file_bad_request() {
        let mut buffer = [0; 1024];
        let request = b"INVALID_REQUEST / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        buffer[..request.len()].copy_from_slice(request);

        set_setting();

        let (status_line, filename) = get_routing_file(&mut buffer);
        assert_eq!(status_line, BAD_REQUEST);
        assert_eq!(filename, "static/400.html");
    }

    fn set_setting() {
        let setting = YamlLoader::load_from_str(r#"
            web_site:
              - host_name: localhost
                server_root_path: "/test_site/site1"
        "#).unwrap();

        {
            let mut global_setting = SETTING.lock().unwrap();
            global_setting.clear();
            global_setting.extend(setting);
        }
    }
}

