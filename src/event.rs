//! Events are incoming messages through the websocket
//! The opposite of EventResponse

use brewdrivers::model::Device;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    DeviceEnact,
    DeviceUpdate,
}

/// An incoming websocket event
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Event {
    pub event_type: EventType,
    pub device: Device,
}
