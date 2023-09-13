//! Responses through the websocket

use serde::Serialize;

use brewdrivers::{
    drivers::InstrumentError,
    model::{Device, RTU},
};
use warp::ws::Message;

/// The valid types of a response
#[derive(Debug, Serialize)]
pub enum OutgoingEventType {
    Error,
    Lock,
    Unlock,
    DeviceUpdateResult,
    DeviceEnactResult,
    RTUUpdateResult,
}

/// A wrapper around the types of data we may want to attach to a response payload
#[derive(Debug, Serialize)]
pub enum OutgoingData<'a> {
    // Rename to 'devices' for consistency
    #[serde(rename = "devices")]
    Devices(Vec<Device>),
    RTU(&'a RTU),
    None,
}

/// A response payload
#[derive(Debug, Serialize)]
pub struct OutgoingEvent<'a> {
    pub response_type: OutgoingEventType,
    pub message: Option<String>,
    pub data: OutgoingData<'a>,
}

impl<'a> OutgoingEvent<'a> {
    /// Creates a new event response
    pub(crate) fn new(
        response_type: OutgoingEventType,
        message: Option<String>,
        data: OutgoingData<'a>,
    ) -> Self {
        Self {
            response_type,
            message,
            data,
        }
    }

    /// Creates a new event response, marked as an error
    pub(crate) fn error(message: String, data: OutgoingData<'a>) -> Self {
        Self {
            response_type: OutgoingEventType::Error,
            message: Some(message),
            data,
        }
    }

    /// Creates a new event response holding an RTU
    pub(crate) fn rtu(rtu: &'a RTU) -> Self {
        Self {
            response_type: OutgoingEventType::RTUUpdateResult,
            message: None,
            data: OutgoingData::RTU(rtu),
        }
    }

    pub(crate) fn lock() -> Self {
        Self {
            response_type: OutgoingEventType::Lock,
            message: None,
            data: OutgoingData::None,
        }
    }

    pub(crate) fn unlock() -> Self {
        Self {
            response_type: OutgoingEventType::Unlock,
            message: None,
            data: OutgoingData::None,
        }
    }

    pub(crate) fn to_msg(&self) -> Message {
        Message::text(serde_json::to_string(self).unwrap())
    }
}

impl<'a> From<InstrumentError> for OutgoingEvent<'a> {
    fn from(e: InstrumentError) -> Self {
        Self::error(format!("Instrument error: {e}"), OutgoingData::None)
    }
}
