//! Execute the Iris server

use log::info;
use env_logger::Env;

/// Same as in lib.rs
pub const CONFIG_FILE: &'static str = "/etc/NavasotaBrewing/rtu_conf.yaml";

pub mod server;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting IRIS web server");
    server::run().await;
}
