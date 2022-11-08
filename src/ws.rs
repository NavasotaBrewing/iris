//! WebSocket details

use brewdrivers::drivers::InstrumentError;
use log::*;
use warp::Rejection;
use warp::reject::Reject;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};


use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

use brewdrivers::model::Device;

pub(crate) type Clients = Arc<Mutex<HashMap<String, Client>>>;

pub(crate) const CLIENT_UPDATE_INTERVAL: Duration = Duration::from_secs(10);

// Represents a websocket client
#[derive(Clone, Debug)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    Enact,
    Update,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorType {
    BrewdriversError,
    SerializationError,
}

/// A websocket event
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Event {
    pub event_type: EventType,
    pub device: Device,
}

pub(crate) struct EventError {
    error_type: ErrorType,
    error: String
}

// Connects a client to a websocket
pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    // see here: https://stackoverflow.com/questions/67602278/rust-tokio-trait-bounds-were-not-satisfied-on-forward-method
    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            error!("error sending websocket msg: {}", e);
        }
    }));

    client.sender = Some(client_sender);
    clients.lock().await.insert(id.clone(), client);

    info!("{} connected", id);

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                error!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        client_msg(&id, msg, &clients).await;
    }

    clients.lock().await.remove(&id);
    info!("{} disconnected", id);
}

// Handler for when a client sends a message
async fn client_msg(id: &str, msg: Message, clients: &Clients) {
    // retrieve the message
    let message = match msg.to_str() {
        Ok(m) => m,
        Err(_) => {
            error!("Couldn't convert ws message to string: {:?}", msg);
            return;
        }
    };

    // Deserialize the event
    let event: Event = match serde_json::from_str(&message) {
        Ok(event) => event,
        Err(e) => {
            error!("Couldn't deserialize event: {}", e);
            return;
        }
    };

    // call the handler and get the response
    let response = handle_event(event).await;

    // If the client is still connected, send the response
    let c = clients.lock().await;
    let client = match c.get(id) {
        Some(client) => client,
        None => {
            error!("Couldn't find client registered with that id: {}", id);
            return;
        }
    };

    if let Some(sender) = &client.sender {
        sender.send(response).unwrap();
    }
}

async fn handle_event(mut event: Event) -> std::result::Result<Message, warp::Error> {
    // update/enact the device based on event type
    let res = match event.event_type {
        EventType::Enact => event.device.enact().await.map_err(|e| BrewdriversError::from(e) ),
        EventType::Update => event.device.update().await.map_err(|e| BrewdriversError::from(e) ),
    };

    match res {
        Ok(_) => Ok(Message::text(
            serde_json::to_string(&event.device).unwrap(),
        )),
        Err(e) => Err(warp::reject::custom(e))
    }
    
}
