//! WebSocket details

use brewdrivers::model::{Device, RTU};
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
    // If this fails, return an error
    match serde_json::from_str(&message) {
        // Get the returned EventResponse and convert it to a Result<Message>
        Ok(event) => return Ok(handle_event(event).await.to_msg()),
        Err(e) => {
            error!("Couldn't deserialize event: {}", e);
            return Ok(EventResponse::error(format!("{:?}", e), ResponseData::None).to_msg());
        }
    };
}

async fn handle_event<'a>(event: Event) -> EventResponse<'a> {
    // Log the incoming event
    info!(
        "Received event type {:?} with {} devices attached",
        event.event_type,
        event.devices.len()
    );
    for device in &event.devices {
        trace!("\tAttached device: {} ({})", device.id, device.name);
    }

    match event.event_type {
        EventType::DeviceEnact => handle_device_enact(event).await,
        EventType::DeviceUpdate => handle_device_update(event).await,
    }
}

async fn handle_device_update<'a>(event: Event) -> EventResponse<'a> {
    // Note: this method and handle_device_enact are very similar
    // I'm keeping them as separate functions because they might
    // diverge later.

    // If they don't provide at least one device, return an error
    if event.devices.len() < 1 {
        let error_msg = format!(
            "Got DeviceUpdate event with {} devices",
            event.devices.len()
        );
        error!("{}", error_msg);
        return EventResponse::error(error_msg, ResponseData::Devices(event.devices));
    }

    // The devices to include in the response
    let mut response_devices: Vec<Device> = Vec::new();
    // Devices that errored. If there's any of these, we'll return an error response
    let mut error_devices: Vec<Device> = Vec::new();

    for mut device in event.devices {
        info!("{}: Updating device", device.id);
        match device.update().await {
            Ok(_) => {
                info!("{}: success", device.id);
                response_devices.push(device);
            }
            Err(e) => {
                error!("{}: failed", device.id);
                error!("{e}");
                error_devices.push(device);
            }
        }
    }

    if error_devices.len() > 0 {
        // Return an error here
        let error_msg = format!(
            "{} devices encountered errors. See logs for more details.",
            error_devices.len()
        );
        return EventResponse::error(error_msg, ResponseData::Devices(error_devices));
    } else {
        // Return a success
        let success_message = format!("{} devices updated successfully", response_devices.len());
        info!("{success_message}");
        return EventResponse::new(
            EventResponseType::DeviceUpdateResult,
            Some(success_message),
            ResponseData::Devices(response_devices),
        );
    }
}
async fn handle_device_enact<'a>(event: Event) -> EventResponse<'a> {
    // If they don't provide at least one device, return an error
    if event.devices.len() < 1 {
        let error_msg = format!("Got DeviceEnact event with {} devices", event.devices.len());
        error!("{}", error_msg);
        return EventResponse::error(error_msg, ResponseData::Devices(event.devices));
    }

    // The devices to include in the response
    let mut response_devices: Vec<Device> = Vec::new();
    // Devices that errored. If there's any of these, we'll return an error response
    let mut error_devices: Vec<Device> = Vec::new();

    let delay = Duration::from_millis(event.time_between);

    for mut device in event.devices {
        info!("{}: Enacting device", device.id);
        match device.enact().await {
            Ok(_) => {
                info!("{}: success", device.id);
                response_devices.push(device);
            }
            Err(e) => {
                error!("{}: failed", device.id);
                error!("{e}");
                error_devices.push(device);

                // If they want to halt after the first error,
                // break from the loop and let the error response go
                if event.halt_if_error {
                    error!("Because of the above error, and halt_if_error = true, halting DeviceEnact event");
                    break;
                }
            }
        }
        // TODO: this will sleep after the last enact, which will delay the whole
        // response by `delay` milliseconds. Not very efficient.
        tokio::time::sleep(delay).await;
    }

    if error_devices.len() > 0 {
        // Return an error here
        let error_msg = format!(
            "{} devices encountered errors. See logs for more details.",
            error_devices.len()
        );
        return EventResponse::error(error_msg, ResponseData::Devices(error_devices));
    } else {
        // Return a success
        let success_message = format!("{} devices enacted successfully", response_devices.len());
        info!("{success_message}");
        return EventResponse::new(
            EventResponseType::DeviceEnactResult,
            Some(success_message),
            ResponseData::Devices(response_devices),
        );
    }
}
