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
pub(crate) struct Event {
    pub event_type: EventType,
    pub device: Device,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct EventList {
    /// List of events
    pub events: Vec<Event>,
    /// Time (in ms) to delay between each event.
    /// This delay is not added before the first event or after the last.
    #[serde(default = "default_time_between")]
    pub time_between: usize,
    // If any event encounters an error, stop and don't process the following events
    #[serde(default = "halt_events_if_error")]
    pub halt_if_error: bool,
}
