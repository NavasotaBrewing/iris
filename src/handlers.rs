//! Basic Route handlers

use uuid::Uuid;
use warp::hyper::StatusCode;
use std::convert::Infallible;
use warp::reply::{json, Reply};
use warp::{Rejection, Filter};
use serde::Serialize;

use crate::ws::{self, Client, Clients};
use log::*;

type Result<T> = std::result::Result<T, Rejection>;

// The response returned from a register request
#[derive(Debug, Serialize)]
pub(crate) struct RegisterResponse {
    url: String,
}

// Register a new client and return the ws address with the client id in it
pub async fn register_handler(clients: Clients) -> Result<impl Reply> {
    let uuid = Uuid::new_v4().simple().to_string();
    register_client(uuid.clone(), clients.clone()).await;
    info!("Just registered a client with id: {}", uuid);
    info!("All clients: {:#?}", clients);
    Ok(json(&RegisterResponse {
        url: format!("ws://127.0.0.1:8000/ws/{}", uuid),
    }))
}

// Registers a client, adding them to the client list
pub async fn register_client(uuid: String, clients: Clients) {
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
