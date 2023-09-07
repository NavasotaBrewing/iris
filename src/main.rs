use std::{collections::HashMap, sync::Arc};

use brewdrivers::model::RTU;
use env_logger::Env;
use log::*;
use tokio::sync::Mutex;
use warp::Filter;

use crate::response::EventResponse;

mod event;
mod handlers;
mod response;
mod ws;

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
        // no panics in this thread!
        loop {
            tokio::time::sleep(ws::CLIENT_UPDATE_INTERVAL).await;

            if let Err(e) = rtu.update().await {
                // Print, but don't panic
                // We don't want to kill this thread or else clients stop getting updated
                error!("{}", e);
            };

            for (_, client) in ws_clients.lock().await.iter() {
                if let Some(sender) = &client.sender {
                    info!("updating client with RTU state: {}", client.client_id);
                    if let Err(e) = sender.send(Ok(EventResponse::rtu(&rtu).to_msg())) {
                        // Same as above
                        error!("{}", e);
                    }
                }
            }
        }
    });

    let routes = register_routes
        .or(ws_routes)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 3012)).await;
}
