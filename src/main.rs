use std::env;
use std::process;

use env_logger;
use log::info;

// Include test modules
#[cfg(test)]
mod routing_test;
#[cfg(test)]
mod config_test;
#[cfg(test)]
mod thread_pool_test;
#[cfg(test)]
mod server_test;

// Import modules
mod config;
mod error;
mod routing;
mod server;
mod thread_pool;

use config::Config;
use server::Server;

/// Main entry point for the application
fn main() {
    // Initialize logging
    env::set_var("RUST_LOG", "trace");
    env_logger::init();

    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Load configuration
    let config = if let Some(config_path) = args.iter().find(|x| x.contains("yaml")) {
        info!("Loading configuration from file: {}", config_path);
        match Config::from_file(config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error loading configuration: {}", e);
                process::exit(1);
            }
        }
    } else {
        info!("Loading default configuration");
        Config::default()
    };

    // Create and start the server
    let server = Server::new(config);
    if let Err(e) = server.start() {
        eprintln!("Server error: {}", e);
        process::exit(1);
    }
}
