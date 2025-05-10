# Rust Webサーバー

## 概要

このプロジェクトは、Rustで作成されたシンプルなWebサーバーです。

## 機能

- HTTP GETリクエストの処理
- 静的ファイルの配信
- 基本的なルーティング
- マルチスレッド処理

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
num_threads: 8  # スレッドプールのスレッド数（省略可、デフォルトはCPUコア数）
```

- `web_site`: ホスト名とそのルートディレクトリのマッピング
- `num_threads`: サーバーが使用するスレッド数。省略した場合はCPUコア数が使用されます。

## 注意
 - こちらのプロジェクトは私の自己学習を目的としています

## License
This project is licensed under the MIT License, see the LICENSE.txt file for details
