//! Basic Route handlers
//!
//! These are for the routes + connecting websockets, not the actual websocket events themselves.

use serde::Serialize;
use std::convert::Infallible;
use uuid::Uuid;
use warp::hyper::StatusCode;
use warp::reply::{json, Reply};
use warp::{Filter, Rejection};

use crate::ws::{self, Client, Clients};
use log::*;

type Result<T> = std::result::Result<T, Rejection>;

// The response returned from a register request
#[derive(Debug, Serialize)]
pub(crate) struct RegisterResponse {
    url: String,
}

// Register a new client and return the ws address with the client id in it
// This generates a UUID for a client, add that UUID to the client list pointing
// at an empty client (no socket connected). Later, we'll add a websocket connection
// to that client.
// TODO: Also send the RTU id for better event reporting on the interface
pub async fn register_handler(clients: Clients) -> Result<impl Reply> {
    let uuid = Uuid::new_v4().simple().to_string();
    // TODO: Can we avoid this clone?
    add_client(uuid.clone(), clients.clone()).await;
    info!("Just registered a client with id: {}", uuid);
    info!("All clients: {:#?}", clients);
    Ok(json(&RegisterResponse {
        url: format!("ws://127.0.0.1:3012/ws/{}", uuid),
    }))
}

// Registers a client, adding them to the client list
pub async fn add_client(uuid: String, clients: Clients) {
    clients.lock().await.insert(
        uuid.clone(),
        Client {
            client_id: uuid,
            sender: None,
        },
    );
}

// Delete a client from the client list
pub async fn unregister_handler(id: String, clients: Clients) -> Result<impl Reply> {
    clients.lock().await.remove(&id);
    info!("Client disconnected: {}", id);
    Ok(StatusCode::OK)
}

// Attempt to find a client from the list, and if so connect a websocket
pub async fn ws_handler(ws: warp::ws::Ws, id: String, clients: Clients) -> Result<impl Reply> {
    let client = clients.lock().await.get(&id).cloned();
    match client {
        Some(c) => Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, id, clients, c))),
        None => Err(warp::reject::not_found()),
    }
}

// Attaches Clients to a warp route
pub(crate) fn with_clients(
    clients: Clients,
) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}
