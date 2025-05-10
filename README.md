# Rust Webサーバー

## 概要

このプロジェクトは、Rustで作成されたシンプルなWebサーバーです。

## 機能

- HTTP GETリクエストの処理
- 静的ファイルの配信（HTML, CSS, JavaScript, 画像など）
- 基本的なルーティング
- 非同期処理（async/await with tokio）
- Content-Typeヘッダーの自動設定

## 前提条件

このプロジェクトをビルドして実行するには、以下が必要です：

- [Rust](https://www.rust-lang.org/tools/install)（最新の安定版）
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)（Rustに同梱）

## 始め方

### リポジトリをクローン

```sh
git clone https://github.com/sasakirione/simple-web-server.git
cd rust-web-server
```

## 設定

設定ファイルはYAML形式で、以下のオプションがあります：

```yaml
web_site:
  - host_name: example.com
    server_root_path: /path/to/site1
  - host_name: another-example.com
    server_root_path: /path/to/site2
num_threads: 8  # tokioランタイムのワーカースレッド数（省略可、デフォルトはCPUコア数）
```

- `web_site`: ホスト名とそのルートディレクトリのマッピング
- `num_threads`: tokioランタイムが使用するワーカースレッド数。省略した場合はCPUコア数が使用されます。

## 静的ファイルの配信

サーバーは以下のような静的ファイルを配信できます：

- HTML (.html) - text/html
- CSS (.css) - text/css
- JavaScript (.js) - application/javascript
- JSON (.json) - application/json
- 画像ファイル:
  - PNG (.png) - image/png
  - JPEG (.jpg, .jpeg) - image/jpeg
  - GIF (.gif) - image/gif
  - SVG (.svg) - image/svg+xml
  - アイコン (.ico) - image/x-icon
- PDF (.pdf) - application/pdf
- テキストファイル (.txt) - text/plain
- その他のファイル - application/octet-stream

ファイル拡張子を持つパス（例：`/styles.css`、`/images/logo.png`）にアクセスすると、サーバーは対応するファイルを直接提供します。
拡張子のないパス（例：`/about`、`/products`）にアクセスすると、サーバーはそのディレクトリの `index.html` ファイルを提供します。

## 注意
 - こちらのプロジェクトは私の自己学習を目的としています

## License
This project is licensed under the MIT License, see the LICENSE.txt file for details
