//! Execute the Iris server

use std::path::Path;

use log::{info, error};
use env_logger::Env;
use nbc_iris::model::RTU;

pub mod model;

/// Same as in lib.rs
pub const CONFIG_FILE: &'static str = "/etc/NavasotaBrewing/rtu_conf.yaml";

// Only compile that module if we want the web server
// This is good for the CLI because we can use the RTU generation
// code without compiling warp and tokio
// TODO: check that the CLI is actually using non-standard features
#[cfg(feature = "web")]
pub mod server;

#[cfg(feature = "web")]
#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // If they pass --validate-config as an arg, validate the default config file
    if let Some(arg1) = args.get(1) {
        if arg1 == "--validate-config" {
            validate_config();
            return;
        }
    }

    info!("Starting IRIS web server");
    server::run().await;
}

// Calls RTU::generate on the given path
pub fn validate_config() {
    let path = Path::new(CONFIG_FILE);
    if !path.exists() {
        error!("Config file does not exist at `{:?}`", path);
        return;
    }

    match RTU::generate(None) {
        Ok(rtu) => info!("RTU `{}` generated successfully and it passed all validators!", rtu.name),
        Err(e) => {
            error!("Couldn't generate rtu: {}", e);
            return;
        }
    }

    info!("Ok!");
}