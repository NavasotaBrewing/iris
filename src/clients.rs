use log::*;
use std::{collections::HashMap, sync::Arc};

use tokio::sync::{mpsc, Mutex};
use warp::ws::Message;

use crate::outgoing_event::OutgoingEvent;

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
    pub async fn send_event_to<'a>(&self, outgoing_event: OutgoingEvent<'a>, client_id: &str) {
        match self.0.lock().await.get(client_id) {
            Some(client) => {
                if let Some(sender) = &client.sender {
                    if let Err(e) = sender.send(Ok(outgoing_event.to_msg())) {
                        error!("Error sending message to client {client_id}: {e}");
                    }
                }
            }
            None => {
                error!("couldn't find client registered with that id: {client_id}");
            }
        }
    }

    pub async fn send_to_all<'a>(&self, outgoing_event: OutgoingEvent<'a>) {
        let client_list = self.0.lock().await;
        for (id, client) in client_list.iter() {
            if let Some(sender) = &client.sender {
                if let Err(e) = sender.send(Ok(outgoing_event.to_msg())) {
                    error!("Error sending message to client {id}: {e}");
                }
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
