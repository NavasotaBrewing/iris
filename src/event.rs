//! Events are incoming messages through the websocket
//! The opposite of EventResponse

use serde::{Deserialize, Serialize};

use brewdrivers::model::Device;

use crate::defaults::{default_time_between, halt_events_if_error};

#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    DeviceEnact,
    DeviceUpdate,
}

/// An incoming websocket event
#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub event_type: EventType,
    // Usually this will just be one device, but it's a list to give
    // the opportunity to operate on multi devices at once
    pub devices: Vec<Device>,
    /// Time (in ms) to delay between each event.
    /// This delay is not added before the first event or after the last.
    #[serde(default = "default_time_between")]
    pub time_between: u64,
    // If any event encounters an error, stop and don't process the following events
    #[serde(default = "halt_events_if_error")]
    pub halt_if_error: bool,
}
