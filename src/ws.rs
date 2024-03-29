//! WebSocket infrastructure
//!
//! This handles the creation/connection of websockets. See [ws_handlers](crate::ws_handlers) for
//! details on what happens to incoming/outgoing events

use brewdrivers::model::RTU;
use log::*;

use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

use crate::clients::{Client, Clients};
use crate::outgoing_event::{OutgoingData, OutgoingEvent};
use crate::ws_handlers;

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

    clients.send_to_all(OutgoingEvent::lock()).await;
    match rtu.update().await {
        Ok(_) => clients.send_event_to(OutgoingEvent::rtu(&rtu), &id).await,
        Err(e) => {
            clients
                .send_event_to(
                    OutgoingEvent::error(
                        format!("Couldn't update RTU when initializing: {e}"),
                        OutgoingData::RTU(&rtu),
                    ),
                    &id,
                )
                .await
        }
    }
    clients.send_to_all(OutgoingEvent::unlock()).await;

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
