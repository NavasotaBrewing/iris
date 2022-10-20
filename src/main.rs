pub mod model;

pub const CONFIG_FILE: &'static str = "/etc/NavasotaBrewing/rtu_conf.yaml";

use log::info;
use env_logger::Env;

// Only compile that module if we want the web server
// This is good for the CLI because we can use the RTU generation
// code without compiling warp and tokio
#[cfg(feature = "web")]
pub mod server;

#[cfg(feature = "web")]
#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting IRIS web server");
    server::run().await;
}
