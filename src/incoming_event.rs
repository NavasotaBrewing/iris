//! Events are incoming messages through the websocket
//! The opposite of EventResponse

use serde::{Deserialize, Serialize};

use brewdrivers::model::{Device, RTU};

use crate::defaults::{default_time_between, halt_events_if_error};

#[derive(Debug, Serialize, Deserialize)]
pub enum IncomingEventType {
    /// Enacts all the devices sent
    DeviceEnact,
    /// Enacts all devices on the attached RTU
    RTUEnact,
    /// Updates all the devices sent
    DeviceUpdate,
    /// Requests that the RTU is reset to it's base configuration
    RTUReset,
}

/// An incoming websocket event
#[derive(Debug, Serialize, Deserialize)]
pub struct IncomingEvent {
    pub event_type: IncomingEventType,
    // Usually this will just be one device, but it's a list to give
    // the opportunity to operate on multi devices at once
    pub devices: Vec<Device>,
    // An entire RTU. Used for setting scenes
    #[allow(non_snake_case)]
    pub RTU: Option<RTU>,
    /// Time (in ms) to delay between each event.
    /// This delay is not added before the first event or after the last.
    #[serde(default = "default_time_between")]
    pub time_between: u64,
    // If any event encounters an error, stop and don't process the following events
    #[serde(default = "halt_events_if_error")]
    pub halt_if_error: bool,
}
