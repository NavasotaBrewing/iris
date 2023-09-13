//! WebSocket infrastructure
//!
//! This handles the creation/connection of websockets. See [ws_handlers](crate::ws_handlers) for
//! details on what happens to incoming/outgoing events

use brewdrivers::model::RTU;
use log::*;
use std::{collections::HashMap, sync::Arc};

use futures::{FutureExt, StreamExt};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

use crate::response::{EventResponse, ResponseData};
use crate::ws_handlers;

// Represents a websocket client
#[derive(Clone, Debug)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

pub type ClientList = Arc<Mutex<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub struct Clients(pub ClientList);

impl Clients {
    pub async fn send_event_to<'a>(&self, outgoing_event: EventResponse<'a>, client_id: &str) {
        match self.0.lock().await.get(client_id) {
            Some(client) => {
                if let Some(sender) = &client.sender {
                    sender.send(Ok(outgoing_event.to_msg())).unwrap();
                }
            }
            None => {
                error!("couldn't find client registered with that id: {client_id}");
            }
        }
    }

    pub async fn send_to_all<'a>(&self, outgoing_event: EventResponse<'a>) {
        let client_list = self.0.lock().await;
        for (_, client) in client_list.iter() {
            if let Some(sender) = &client.sender {
                sender.send(Ok(outgoing_event.to_msg())).unwrap();
            }
        }
    }

    pub async fn add_client(&self, id: String, client: Client) {
        self.0.lock().await.insert(id, client);
    }

    pub async fn remove_client(&self, id: &str) {
        self.0.lock().await.remove(id);
    }
}

// Connects a client to a websocket
pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    // Reading this code seems like it shouldn't work... but it does. ?
    // see here: https://stackoverflow.com/questions/67602278/rust-tokio-trait-bounds-were-not-satisfied-on-forward-method
    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            error!("error sending websocket msg: {}", e);
        }
    }));

    client.sender = Some(client_sender);

    // Add the client
    clients.add_client(id.clone(), client.clone()).await;

    info!("{} connected", id);

    // Once they're connected, go ahead and send them the RTU state
    // so they don't have to wait for the next pass
    let mut rtu = RTU::generate(None).unwrap();
    match rtu.update().await {
        Ok(_) => clients.send_event_to(EventResponse::rtu(&rtu), &id).await,
        Err(e) => {
            clients
                .send_event_to(
                    EventResponse::error(
                        format!("Couldn't update RTU when initializing: {e}"),
                        ResponseData::RTU(&rtu),
                    ),
                    &id,
                )
                .await
        }
    }

    // Then loop until they disconnect, sending messages to the handler
    while let Some(result) = client_ws_rcv.next().await {
        match result {
            Ok(msg) => handle_incoming_msg(msg, &clients, &id).await,
            Err(e) => error!("error receiving ws message for client {}: {}", id, e),
        }
    }

    clients.remove_client(&id).await;
    info!("{} disconnected", id);
}

// Handler for when a client sends a message
async fn handle_incoming_msg(msg: Message, clients: &Clients, client_id: &str) {
    // Get the msg content.
    // If this fails, log an error and return
    let message = match msg.to_str() {
        Ok(m) => m,
        Err(e) => {
            error!("Couldn't convert ws message to string: {:?}", e);
            return;
        }
    };

    // Pass the event and the clients to the ws handler.
    // We pass the clients so that the handler can decide who gets updated
    match serde_json::from_str(&message) {
        Ok(event) => ws_handlers::handle_event(event, clients, client_id).await,
        Err(e) => error!("Couldn't deserialize event: {}", e),
    };
}
