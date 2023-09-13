use std::{collections::HashMap, sync::Arc};

use brewdrivers::model::RTU;
use env_logger::Env;
use log::*;
use tokio::sync::Mutex;
use warp::Filter;

use crate::{clients::Clients, outgoing_event::OutgoingEvent};

mod clients;
pub mod defaults;
mod http_handlers;
mod incoming_event;
mod outgoing_event;
mod ws;
mod ws_handlers;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting IRIS WebSocket server");

    let ws_clients: Clients = Clients(Arc::new(Mutex::new(HashMap::new())));
    let mut rtu = RTU::generate(None).unwrap();

    let register = warp::path("register");
    let register_routes = register
        .and(warp::get())
        .and(http_handlers::with_clients(ws_clients.clone()))
        .and_then(http_handlers::register_handler)
        .or(register
            .and(warp::delete())
            .and(warp::path::param())
            .and(http_handlers::with_clients(ws_clients.clone()))
            .and_then(http_handlers::unregister_handler));

    let ws_routes = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(http_handlers::with_clients(ws_clients.clone()))
        .and_then(http_handlers::ws_handler);

    // At a regular interval, update all clients with this RTUs state
    tokio::task::spawn(async move {
        // no panics in this thread!
        loop {
            tokio::time::sleep(defaults::client_update_interval()).await;

            // Send the lock event so the UI can lock itself and prevent
            // actions briefly while we're updating
            ws_clients.send_to_all(OutgoingEvent::lock()).await;

            if let Err(e) = rtu.update().await {
                // Print, but don't panic
                // We don't want to kill this thread or else clients stop getting updated
                error!("{}", e);
            };

            ws_clients.send_to_all(OutgoingEvent::unlock()).await;

            for (_, client) in ws_clients.0.lock().await.iter() {
                if let Some(sender) = &client.sender {
                    info!("updating client with RTU state: {}", client.client_id);
                    if let Err(e) = sender.send(Ok(OutgoingEvent::rtu(&rtu).to_msg())) {
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
