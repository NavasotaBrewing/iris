pub const CONFIG_FILE: &'static str = "/etc/NavasotaBrewing/rtu_conf.yaml";

use std::{sync::Arc, collections::HashMap};

use brewdrivers::model::RTU;
use env_logger::Env;
use log::*;
use tokio::sync::Mutex;
use warp::{ws::Message, Filter};

mod ws;
mod handlers;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting IRIS WebSocket server");

    let ws_clients: ws::Clients = Arc::new(Mutex::new(HashMap::new()));
    let mut rtu = RTU::generate(None).unwrap();

    let register = warp::path("register");
    let register_routes = register
        .and(warp::get())
        .and(handlers::with_clients(ws_clients.clone()))
        .and_then(handlers::register_handler)
        .or(register
            .and(warp::delete())
            .and(warp::path::param())
            .and(handlers::with_clients(ws_clients.clone()))
            .and_then(handlers::unregister_handler));

    let ws_routes = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(handlers::with_clients(ws_clients.clone()))
        .and_then(handlers::ws_handler);

    // At a regular interval, update all clients with their state
    tokio::task::spawn(async move {
        loop {
            tokio::time::sleep(ws::CLIENT_UPDATE_INTERVAL).await;
            rtu.update().await.unwrap();
            for (_, client) in ws_clients.lock().await.iter() {
                if let Some(sender) = &client.sender {
                    info!("updating client with RTU state: {}", client.client_id);
                    sender
                        .send(Ok(Message::text(serde_json::to_string(&rtu).unwrap())))
                        .unwrap();
                }
            }
        }
    });

    let routes = register_routes
        .or(ws_routes)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
