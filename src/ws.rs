//! WebSocket details

use brewdrivers::model::RTU;
use log::*;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use futures::{FutureExt, StreamExt};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

use crate::event::{Event, EventType};
use crate::response::{EventResponse, EventResponseType, ResponseData};

pub(crate) type Clients = Arc<Mutex<HashMap<String, Client>>>;

pub(crate) const CLIENT_UPDATE_INTERVAL: Duration = Duration::from_secs(25);

// Represents a websocket client
#[derive(Clone, Debug)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
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
    clients.lock().await.insert(id.clone(), client.clone());

    info!("{} connected", id);
    // Once they're connected, go ahead and send them the RTU state
    // so they don't have to wait for the next pass
    let mut rtu = RTU::generate(None).unwrap();
    rtu.update().await.unwrap();
    client
        .sender
        .unwrap()
        .send(Ok(EventResponse::rtu(&rtu).to_msg()))
        .unwrap();

    // For each message
    while let Some(result) = client_ws_rcv.next().await {
        // Retrieve the message
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                error!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };

        // Call the handler and get the response
        // Note that errors that happen in brewdrivers are returned
        // as an Ok(Message) with an error payload inside, not an Err()
        let response = client_msg(msg).await;

        // If the client is still connected, send the response
        let c = clients.lock().await;
        match c.get(&id) {
            Some(client) => {
                if let Some(sender) = &client.sender {
                    sender.send(response).unwrap();
                }
            }
            None => {
                error!("Couldn't find client registered with that id: {}", id);
                return;
            }
        };
    }

    clients.lock().await.remove(&id);
    info!("{} disconnected", id);
}

// Handler for when a client sends a message
// Parse the event, handle it, and return a response event (or error)
async fn client_msg(msg: Message) -> Result<Message, warp::Error> {
    // retrieve the message
    let message = match msg.to_str() {
        Ok(m) => m,
        Err(e) => {
            error!("Couldn't convert ws message to string: {:?}", msg);
            return Ok(EventResponse::error(format!("{:?}", e), ResponseData::None).to_msg());
        }
    };

    // Deserialize the Event and handle it
    match serde_json::from_str(&message) {
        // Get the returned EventResponse and convert it to a Result<Message>
        Ok(event) => return Ok(handle_event(event).await.to_msg()),
        Err(e) => {
            error!("Couldn't deserialize event: {}", e);
            return Ok(EventResponse::error(format!("{:?}", e), ResponseData::None).to_msg());
        }
    };
}

async fn handle_event<'a>(mut event: Event) -> EventResponse<'a> {
    // update/enact the device based on event type
    let res: EventResponse = match event.event_type {
        EventType::DeviceEnact => {
            info!("Enacting device: `{}`", event.device.id);
            match event.device.enact().await {
                Ok(_) => EventResponse::new(
                    EventResponseType::DeviceEnactResult,
                    Some(format!("Device enacted")),
                    ResponseData::Device(event.device.clone()),
                ),
                Err(e) => EventResponse::from(e),
            }
        }
        EventType::DeviceUpdate => {
            info!("Enacting device: `{}`", event.device.id);
            match event.device.update().await {
                Ok(_) => EventResponse::new(
                    EventResponseType::DeviceUpdateResult,
                    Some(format!("Device updated")),
                    ResponseData::Device(event.device.clone()),
                ),
                Err(e) => EventResponse::from(e),
            }
        }
    };

    res
}
